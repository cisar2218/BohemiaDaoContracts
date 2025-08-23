# Bohemia DAO Contracts

A decentralized autonomous organization (DAO) smart contract built from scratch using Ink! framework for Polkadot/Substrate ecosystem. This project was developed as part of the Synergy Hackathon 2025 (Berlin Web3 Summit).

## Overview

Bohemia DAO is a governance-focused smart contract that enables decentralized decision-making through a token-based voting system. The contract supports multiple proposal types and provides a robust framework for community governance and fund management.

## Key Features

### Governance & Voting
- Multiple Proposal Types: Support for general voting (multiple choice) and funding requests
- Token-Based Voting: Members vote with proportional weight based on token holdings
- Time-Limited Proposals: Configurable voting periods with automatic expiration
- Vote Tracking

### Member Management
- Founding Members: Initial token distribution among founding members
- Member Verification: Built-in membership validation system
- Token Distribution: Equal initial allocation with capability for additional distributions

### Treasury Management
- Money Request Proposals: Members can propose funding for projects
- Transparent Voting: All funding decisions require community approval
- Balance Tracking: Monitor individual and total token supplies

## Screenshots

The following screenshots demonstrate the contract's functionality through the frontend interface:

### Proposal Preview (Paset Hub)
<img width="1997" height="1183" alt="image" src="https://github.com/user-attachments/assets/29c6ab89-e387-4489-af6c-72a5b8fbb100" />

### Voting
<img width="1997" height="1183" alt="image" src="https://github.com/user-attachments/assets/ae951c12-93a0-4ac6-95fa-01d0864073da" />

### Member Verification System
<img width="1997" height="1183" alt="image" src="https://github.com/user-attachments/assets/787e79f0-ff26-4fb9-9ae7-e37b25103561" />

<img width="1997" height="1183" alt="image" src="https://github.com/user-attachments/assets/b4a3942f-1949-4c21-83e2-f7419748a56a" />

## Technical Implementation

### Smart Contract Architecture
- **Language**: Rust
- **Framework**: Ink! (Polkadot/Substrate smart contracts)
- **Storage**: Efficient mapping-based storage for members, proposals, and votes
- **Events**: Comprehensive event emission for frontend integration

### Core Functions
- `new()` - Initialize DAO with founding members and parameters
- `create_proposal()` - Submit new proposals for community voting
- `vote()` - Cast votes on active proposals
- `get_proposal()` - Retrieve proposal details and current vote counts
- `distribute_tokens()` - Manage token distribution to members

### Testing
- Unit tests with small coverage

## Deployment Information

**Testnet Contract Address**: `0x170eAb77A1911b43cE5B9E09fd610861026AD42e`

## Future Roadmap

### Post-Hackathon Development
- Mobile Integration
- Enhanced Security
- Mainnet Deployment
- Additional Features


## Contributing

This project was developed for the Synergy Hackathon 2025. Contributions and feedback are welcome as we continue development toward a production-ready DAO solution.