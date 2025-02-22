# Solana Token Distribution Program

A Solana program for automated token distribution to holders based on their token balance. The program implements proportional reward distribution where each eligible holder (holding >1000 tokens) receives rewards according to the formula:

```
Ri = (Ti/Ttotal) × X

Where:
- Ri: Rewards for account i
- Ti: Number of tokens held by account i
- Ttotal: Total tokens held by all eligible accounts (>1K tokens)
- X: Total rewards to distribute
```

## Features

- Automated 10-minute distribution intervals
- Minimum token threshold (1000 tokens)
- Batch processing for large numbers of holders
- Secure vault authority using PDAs
- Complete distribution lifecycle management
- Client SDK for easy integration

## Prerequisites

- Rust 1.69.0 or later
- Solana CLI tools 1.16.0 or later
- Node.js 16.0.0 or later
- Anchor Framework 0.28.0 or later

## Installation

1. Clone the repository:
```bash
git clone https://github.com/abhirupinspace/solana-token-distribution
cd solana-token-distribution
```

2. Install dependencies:
```bash
npm install
```

3. Build the program:
```bash
anchor build
```

## Program Deployment

1. Set up Solana config for devnet:
```bash
solana config set --url devnet
```

2. Create a new keypair for deployment:
```bash
solana-keygen new -o deploy-keypair.json
```

3. Get devnet SOL:
```bash
solana airdrop 2 $(solana-keygen pubkey deploy-keypair.json) --url devnet
```

4. Deploy the program:
```bash
anchor deploy --program-id D3LMDue6hQpkjM5SUFcFnc5i2GH9Qk2FjNwngGG5Zhfe --provider.wallet deploy-keypair.json
```

## Token Setup

1. Create reward token:
```bash
spl-token create-token --decimals 6
export REWARD_TOKEN=<token_address>
```

2. Create token account:
```bash
spl-token create-account $REWARD_TOKEN
```

3. Mint initial supply:
```bash
spl-token mint $REWARD_TOKEN 1000000000
```

## Program Initialization

```typescript
import { Connection, Keypair } from '@solana/web3.js';
import { TokenDistributorClient } from './client';

const connection = new Connection('https://api.devnet.solana.com');
const wallet = new Wallet(Keypair.fromSecretKey(/* your keypair */));
const client = new TokenDistributorClient(connection, wallet);

// Initialize program
const { stateKey, vaultAuthority, vaultAddress } = await client.initialize(
  new PublicKey(process.env.REWARD_TOKEN)
);
```

## Running Distributions

```typescript
// Start a distribution cycle
await client.executeDistribution(
  stateKey,
  new PublicKey(process.env.REWARD_TOKEN),
  vaultAddress,
  vaultAuthority
);
```

## Client SDK Usage

The client SDK provides a complete interface for interacting with the program:

```typescript
// Find eligible token accounts
const eligibleAccounts = await client.findEligibleTokenAccounts(tokenMint);

// Calculate total eligible tokens
await client.calculateTotalEligibleTokens(state, eligibleAccounts);

// Distribute rewards
await client.distributeRewards(
  state,
  tokenMint,
  holderTokenAccount,
  rewardVault,
  vaultAuthority
);

// Monitor distribution
await client.monitorDistribution(state);
```

## Program Architecture

### State Account
- Authority
- Token mint
- Distribution interval
- Minimum token threshold
- Total eligible tokens
- Distribution status

### Instructions
- `initialize`: Set up the distributor
- `startDistribution`: Begin distribution cycle
- `calculateTotalEligibleTokens`: Calculate total eligible supply
- `distributeRewards`: Send rewards to holders
- `endDistribution`: Complete distribution cycle

### Security Features
- PDA for vault authority
- Minimum token threshold enforcement
- Distribution interval checks
- Overflow protection in calculations

## Testing

1. Run test suite:
```bash
anchor test
```

2. Run specific tests:
```bash
npm run test tests/distributor.test.ts
```

## Development

1. Modify program code:
```bash
cd programs/token_distributor/src
```

2. Build changes:
```bash
anchor build
```

3. Update program ID:
```bash
solana address -k target/deploy/token_distributor-keypair.json
```
