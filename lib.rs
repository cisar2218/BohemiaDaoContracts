#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod simple_dao {
    use ink::prelude::string::String;
    use ink::prelude::vec;
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use ink::H160;

    #[derive(Debug, PartialEq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum ProposalType {
        MultipleChoice,
        MoneyRequest,
    }

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum ProposalStatus {
        Active,
        Passed,
        Rejected,
        Expired,
    }

    #[derive(Debug)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Proposal {
        pub id: u32,
        pub name: String,
        pub description: String,
        pub author: H160,
        pub proposal_type: ProposalType,
        pub options: Vec<String>, // For multiple choice or single option for money request
        pub amount: Option<Balance>, // For money request proposals
        pub votes: Vec<u32>,      // Vote count for each option
        pub voted_members: Vec<H160>,
        pub status: ProposalStatus,
        pub created_at: u64,
        pub voting_deadline: u64,
    }

    #[ink(storage)]
    pub struct SimpleDao {
        members: Vec<H160>,
        member_tokens: Mapping<H160, Balance>,
        total_supply: Balance,

        proposals: Mapping<u32, Proposal>,
        next_proposal_id: u32,

        voting_period: u64, // in blocks
        min_votes_required: u32,
    }

    #[derive(Debug)]
    #[ink(event)]
    pub struct DaoInitiated {
        #[ink(topic)]
        members: Vec<H160>,
        total_supply: Balance,
    }

    #[derive(Debug)]
    #[ink(event)]
    pub struct ProposalCreated {
        #[ink(topic)]
        proposal_id: u32,
        #[ink(topic)]
        author: H160,
        name: String,
    }

    #[derive(Debug)]
    #[ink(event)]
    pub struct VoteCast {
        #[ink(topic)]
        proposal_id: u32,
        #[ink(topic)]
        voter: H160,
        option: u32,
    }

    #[derive(Debug)]
    #[ink(event)]
    pub struct TokensDistributed {
        #[ink(topic)]
        recipient: H160,
        amount: Balance,
    }

    // Custom errors
    #[derive(Debug, PartialEq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        NotMember,
        ProposalNotFound,
        ProposalExpired,
        AlreadyVoted,
        InvalidOption,
        InvalidProposalType,
        InsufficientBalance,
        EmptyMembers,
        InvalidVotingPeriod,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl SimpleDao {
        /// Initialize the DAO with founding members
        #[ink(constructor)]
        pub fn new(
            members: Vec<H160>,
            total_supply: Balance,
            voting_period: u64,
            min_votes_required: u32,
        ) -> Self {
            assert!(members.len() > 0, "Invalid number of members specified.");
            assert!(voting_period > 0, "Invalid voting period.");

            let mut dao = Self {
                members: members.clone(),
                member_tokens: Mapping::new(),
                total_supply,
                proposals: Mapping::new(),
                next_proposal_id: 1,
                voting_period,
                min_votes_required,
            };

            // Distribute initial tokens equally among founding members
            let tokens_per_member = total_supply / members.len() as Balance;
            for member in &members {
                dao.member_tokens.insert(member, &tokens_per_member);
            }

            Self::env().emit_event(DaoInitiated {
                members,
                total_supply,
            });

            dao
        }

        /// Distribute additional tokens to a member (only callable by contract)
        #[ink(message)]
        pub fn distribute_tokens(&mut self, recipient: H160, amount: Balance) -> Result<()> {
            // Only existing members can receive tokens
            if !self.members.contains(&recipient) {
                return Err(Error::NotMember);
            }

            let current_balance = self.member_tokens.get(&recipient).unwrap_or(0);
            self.member_tokens
                .insert(&recipient, &(current_balance + amount));
            self.total_supply += amount;

            Self::env().emit_event(TokensDistributed { recipient, amount });

            Ok(())
        }

        /// Create a new proposal
        #[ink(message)]
        pub fn create_proposal(
            &mut self,
            name: String,
            description: String,
            proposal_type: ProposalType,
            options: Vec<String>,
            amount: Option<Balance>,
        ) -> Result<u32> {
            let caller: H160 = self.env().caller();

            // Validate proposal based on type
            match proposal_type {
                ProposalType::MultipleChoice => {
                    if options.is_empty() {
                        return Err(Error::InvalidProposalType);
                    }
                }
                ProposalType::MoneyRequest => {
                    if amount.is_none() || options.len() != 1 {
                        return Err(Error::InvalidProposalType);
                    }
                }
            }

            let proposal_id = self.next_proposal_id;
            let current_block = self.env().block_number() as u64;

            let proposal = Proposal {
                id: proposal_id,
                name: name.clone(),
                description,
                author: caller,
                proposal_type,
                options: options.clone(),
                amount,
                votes: vec![0; options.len()],
                voted_members: Vec::new(),
                status: ProposalStatus::Active,
                created_at: current_block,
                voting_deadline: current_block as u64 + self.voting_period,
            };

            self.proposals.insert(&proposal_id, &proposal);
            self.next_proposal_id += 1;

            Self::env().emit_event(ProposalCreated {
                proposal_id,
                author: caller,
                name,
            });

            Ok(proposal_id)
        }

        /// Cast a vote on a proposal
        #[ink(message)]
        pub fn vote(&mut self, proposal_id: u32, option: u32) -> Result<()> {
            let caller: H160 = self.env().caller();

            // Check if caller is a member
            if !self.members.contains(&caller) {
                return Err(Error::NotMember);
            }

            let mut proposal = self
                .proposals
                .get(&proposal_id)
                .ok_or(Error::ProposalNotFound)?;

            // Check if member has altready voted
            if proposal.voted_members.contains(&caller) {
                return Err(Error::AlreadyVoted);
            }

            // Check if proposal is still active
            if proposal.status != ProposalStatus::Active {
                return Err(Error::ProposalExpired);
            }

            // Check if voting period has expired
            if self.env().block_number() as u64 > proposal.voting_deadline {
                proposal.status = ProposalStatus::Expired;
                self.proposals.insert(&proposal_id, &proposal);
                return Err(Error::ProposalExpired);
            }

            // Validate option
            if option as usize >= proposal.options.len() {
                return Err(Error::InvalidOption);
            }

            // Cast vote
            proposal.votes[option as usize] += 1;
            proposal.voted_members.push(caller);

            // Update proposal status if needed
            self.update_proposal_status(&mut proposal);

            self.proposals.insert(&proposal_id, &proposal);

            Self::env().emit_event(VoteCast {
                proposal_id,
                voter: caller,
                option,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn get_proposal(&self, proposal_id: u32) -> Result<Proposal> {
            let mut proposal = self
                .proposals
                .get(&proposal_id)
                .ok_or(Error::ProposalNotFound)?;

            // Update status if voting period expired
            if proposal.status == ProposalStatus::Active
                && self.env().block_number() as u64 > proposal.voting_deadline
            {
                proposal.status = ProposalStatus::Expired;
            }

            Ok(proposal)
        }

        #[ink(message)]
        pub fn get_active_proposals(&self) -> Vec<u32> {
            let mut active_proposals = Vec::new();

            for id in 1..self.next_proposal_id {
                if let Some(proposal) = self.proposals.get(&id) {
                    if proposal.status == ProposalStatus::Active
                        && self.env().block_number() as u64 <= proposal.voting_deadline
                    {
                        active_proposals.push(id);
                    }
                }
            }

            active_proposals
        }

        #[ink(message)]
        pub fn get_member_balance(&self, member: H160) -> Balance {
            self.member_tokens.get(&member).unwrap_or(0)
        }

        #[ink(message)]
        pub fn get_members(&self) -> Vec<H160> {
            self.members.clone()
        }

        #[ink(message)]
        pub fn get_total_supply(&self) -> Balance {
            self.total_supply
        }

        #[ink(message)]
        pub fn is_member(&self, account: H160) -> bool {
            self.members.contains(&account)
        }

        #[ink(message)]
        pub fn add_blocks(&self) -> u32 {
            self.env().block_number()
        }

        fn update_proposal_status(&self, proposal: &mut Proposal) {
            let total_votes: u32 = proposal.votes.iter().sum();

            if total_votes >= self.min_votes_required {
                match proposal.proposal_type {
                    ProposalType::MultipleChoice => {
                        // Find the option with most votes
                        let max_votes = proposal.votes.iter().max().unwrap_or(&0);
                        if *max_votes > total_votes / 2 {
                            proposal.status = ProposalStatus::Passed;
                        }
                    }
                    ProposalType::MoneyRequest => {
                        // Simple majority for money requests
                        if proposal.votes[0] > total_votes / 2 {
                            proposal.status = ProposalStatus::Passed;
                        } else {
                            proposal.status = ProposalStatus::Rejected;
                        }
                    }
                }
            }
        }
    }
}

pub use self::simple_dao::*;
