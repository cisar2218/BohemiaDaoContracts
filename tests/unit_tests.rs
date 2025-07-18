#[cfg(test)]
mod tests {
    use ink::env;
    use ink::env::block_number;
    use ink::env::test;
    use ink::H160;
    use simple_dao::*;

    // Helper function to create test accounts
    fn create_accounts() -> (H160, H160, H160, H160) {
        let account1 = H160::from([1; 20]);
        let account2 = H160::from([2; 20]);
        let account3 = H160::from([3; 20]);
        let non_member = H160::from([4; 20]);
        (account1, account2, account3, non_member)
    }

    // Helper function to set caller
    fn set_caller(caller: H160) {
        test::set_caller(caller.into());
    }

    fn advance_block(blocks: u64) {}

    #[ink::test]
    fn test_dao_creation_with_1_member() {
        let (account1, _, _, _) = create_accounts();
        set_caller(account1);

        let dao = SimpleDao::new(
            vec![account1],
            1000,
            10, // voting period
            1,  // min votes required
        );

        // Test get_members works
        let members = dao.get_members();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0], account1);

        // Test token distribution
        assert_eq!(dao.get_member_balance(account1), 1000);
        assert_eq!(dao.get_total_supply(), 1000);

        // Test membership check
        assert!(dao.is_member(account1));
    }

    #[ink::test]
    fn test_dao_creation_with_2_members() {
        let (account1, account2, _, _) = create_accounts();
        set_caller(account1);

        let dao = SimpleDao::new(
            vec![account1, account2],
            1000,
            10, // voting period
            1,  // min votes required
        );

        // Test get_members works
        let members = dao.get_members();
        assert_eq!(members.len(), 2);
        assert!(members.contains(&account1));
        assert!(members.contains(&account2));

        // Test token distribution (should be equal)
        assert_eq!(dao.get_member_balance(account1), 500);
        assert_eq!(dao.get_member_balance(account2), 500);
        assert_eq!(dao.get_total_supply(), 1000);

        // Test membership check
        assert!(dao.is_member(account1));
        assert!(dao.is_member(account2));
    }

    #[ink::test]
    fn test_dao_creation_with_3_members() {
        let (account1, account2, account3, _) = create_accounts();
        set_caller(account1);

        let dao = SimpleDao::new(
            vec![account1, account2, account3],
            1000,
            10, // voting period
            1,  // min votes required
        );

        // Test get_members works
        let members = dao.get_members();
        assert_eq!(members.len(), 3);
        assert!(members.contains(&account1));
        assert!(members.contains(&account2));
        assert!(members.contains(&account3));

        // Test token distribution (should be equal, with integer division)
        assert_eq!(dao.get_member_balance(account1), 333);
        assert_eq!(dao.get_member_balance(account2), 333);
        assert_eq!(dao.get_member_balance(account3), 333);
        assert_eq!(dao.get_total_supply(), 1000);

        // Test membership check
        assert!(dao.is_member(account1));
        assert!(dao.is_member(account2));
        assert!(dao.is_member(account3));
    }

    #[ink::test]
    fn test_member_can_vote() {
        let (account1, _, _, _) = create_accounts();
        set_caller(account1);

        let mut dao = SimpleDao::new(
            vec![account1],
            1000,
            10, // voting period
            1,  // min votes required
        );

        // Create a proposal
        let proposal_id = dao
            .create_proposal(
                "Test Proposal".to_string(),
                "A test proposal".to_string(),
                ProposalType::MultipleChoice,
                vec!["Option A".to_string(), "Option B".to_string()],
                None,
            )
            .unwrap();

        // Member should be able to vote
        let result = dao.vote(proposal_id, 0);
        assert!(result.is_ok());

        // Check proposal was updated
        let proposal = dao.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.votes[0], 1);
        assert_eq!(proposal.votes[1], 0);
        assert!(proposal.voted_members.contains(&account1));
    }

    #[ink::test]
    fn test_non_member_cannot_vote() {
        let (account1, _, _, non_member) = create_accounts();
        set_caller(account1);

        let mut dao = SimpleDao::new(
            vec![account1],
            1000,
            10, // voting period
            1,  // min votes required
        );

        // Create a proposal
        let proposal_id = dao
            .create_proposal(
                "Test Proposal".to_string(),
                "A test proposal".to_string(),
                ProposalType::MultipleChoice,
                vec!["Option A".to_string(), "Option B".to_string()],
                None,
            )
            .unwrap();

        // Switch to non-member
        set_caller(non_member);

        // Non-member should not be able to vote
        let result = dao.vote(proposal_id, 0);
        assert_eq!(result, Err(Error::NotMember));

        // Check proposal was not updated
        let proposal = dao.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.votes[0], 0);
        assert_eq!(proposal.votes[1], 0);
        assert!(!proposal.voted_members.contains(&non_member));
    }

    #[ink::test]
    fn test_member_cannot_vote_twice() {
        let (account1, _, _, _) = create_accounts();
        set_caller(account1);

        let mut dao = SimpleDao::new(
            vec![account1],
            1000,
            1000, // voting period
            1,    // min votes required
        );

        // Create a proposal
        let proposal_id = dao
            .create_proposal(
                "Test Proposal".to_string(),
                "A test proposal".to_string(),
                ProposalType::MultipleChoice,
                vec!["Option A".to_string(), "Option B".to_string()],
                None,
            )
            .unwrap();

        // First vote should succeed
        let result1 = dao.vote(proposal_id, 0);
        assert!(result1.is_ok());

        // Second vote should fail
        let result2 = dao.vote(proposal_id, 1);
        assert_eq!(result2, Err(Error::AlreadyVoted));

        // Check only first vote was counted
        let proposal = dao.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.votes[0], 1);
        assert_eq!(proposal.votes[1], 0);
        assert_eq!(proposal.voted_members.len(), 1);
    }

    #[ink::test]
    fn test_invalid_vote_option() {
        let (account1, _, _, _) = create_accounts();
        set_caller(account1);

        let mut dao = SimpleDao::new(
            vec![account1],
            1000,
            10, // voting period
            1,  // min votes required
        );

        // Create a proposal with 2 options
        let proposal_id = dao
            .create_proposal(
                "Test Proposal".to_string(),
                "A test proposal".to_string(),
                ProposalType::MultipleChoice,
                vec!["Option A".to_string(), "Option B".to_string()],
                None,
            )
            .unwrap();

        // Vote for invalid option (index 2 doesn't exist)
        let result = dao.vote(proposal_id, 2);
        assert_eq!(result, Err(Error::InvalidOption));

        // Check no votes were recorded
        let proposal = dao.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.votes[0], 0);
        assert_eq!(proposal.votes[1], 0);
    }

    #[ink::test]
    fn test_money_request_proposal() {
        let (account1, _, _, _) = create_accounts();
        set_caller(account1);

        let mut dao = SimpleDao::new(
            vec![account1],
            1000,
            10, // voting period
            1,  // min votes required
        );

        // Create a money request proposal
        let proposal_id = dao
            .create_proposal(
                "Fund Project".to_string(),
                "Request funding for development".to_string(),
                ProposalType::MoneyRequest,
                vec!["Approve funding".to_string()],
                Some(500),
            )
            .unwrap();

        // Vote on money request
        let result = dao.vote(proposal_id, 0);
        assert!(result.is_ok());

        // Check proposal was updated
        let proposal = dao.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.votes[0], 1);
        assert_eq!(proposal.amount, Some(500));
        assert_eq!(proposal.proposal_type, ProposalType::MoneyRequest);
    }

    #[ink::test]
    fn test_proposal_status_update() {
        let (account1, account2, _, _) = create_accounts();
        set_caller(account1);

        let mut dao = SimpleDao::new(
            vec![account1, account2],
            1000,
            10, // voting period
            2,  // min votes required
        );

        // Create a money request proposal
        let proposal_id = dao
            .create_proposal(
                "Fund Project".to_string(),
                "Request funding".to_string(),
                ProposalType::MoneyRequest,
                vec!["Approve".to_string()],
                Some(500),
            )
            .unwrap();

        // First vote (should still be active)
        dao.vote(proposal_id, 0).unwrap();
        let proposal = dao.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Active);

        // Second vote (should pass with majority)
        set_caller(account2);
        dao.vote(proposal_id, 0).unwrap();
        let proposal = dao.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Passed);
    }
}
