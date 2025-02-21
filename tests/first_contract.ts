// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { TokenDistributor } from "../target/types/token_distributor";
// import { BN } from "bn.js";
// import {
//   PublicKey,
//   Keypair,
//   SystemProgram,
//   LAMPORTS_PER_SOL,
// } from "@solana/web3.js";
// import {
//   TOKEN_PROGRAM_ID,
//   MINT_SIZE,
//   createMint,
//   createAccount,
//   mintTo,
//   getAccount,
//   getMint,
// } from "@solana/spl-token";
// import { assert } from "chai";

// describe("token_distributor", () => {
//   // Configure the client to use the local cluster
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   const program = anchor.workspace.TokenDistributor as Program<TokenDistributor>;
  
//   // Generate keypairs for our test accounts
//   const mintAuthority = Keypair.generate();
//   const tokenHolder1 = Keypair.generate();
//   const tokenHolder2 = Keypair.generate();
  
//   // We'll set these in the beforeEach hook
//   let mintAddress: PublicKey;
//   let rewardVault: PublicKey;
//   let holder1TokenAccount: PublicKey;
//   let holder2TokenAccount: PublicKey;
//   let distributorPDA: PublicKey;
//   let vaultAuthority: PublicKey;
//   let vaultBump: number;

//   beforeEach(async () => {
//     // Airdrop SOL to mint authority for creating token mint
//     const airdropSig = await provider.connection.requestAirdrop(
//       mintAuthority.publicKey,
//       2 * LAMPORTS_PER_SOL
//     );
//     await provider.connection.confirmTransaction(airdropSig);

//     // Create token mint
//     mintAddress = await createMint(
//       provider.connection,
//       mintAuthority,
//       mintAuthority.publicKey,
//       null,
//       9,
//       undefined,
//       undefined,
//       TOKEN_PROGRAM_ID
//     );

//     // Create token accounts
//     holder1TokenAccount = await createAccount(
//       provider.connection,
//       tokenHolder1,
//       mintAddress,
//       tokenHolder1.publicKey
//     );

//     holder2TokenAccount = await createAccount(
//       provider.connection,
//       tokenHolder2,
//       mintAddress,
//       tokenHolder2.publicKey
//     );

//     rewardVault = await createAccount(
//       provider.connection,
//       mintAuthority,
//       mintAddress,
//       mintAuthority.publicKey
//     );

//     // Find PDA for distributor
//     [distributorPDA] = await PublicKey.findProgramAddress(
//       [Buffer.from("distributor"), mintAddress.toBuffer()],
//       program.programId
//     );

//     // Find PDA for vault authority
//     [vaultAuthority, vaultBump] = await PublicKey.findProgramAddress(
//       [Buffer.from("vault")],
//       program.programId
//     );

//     // Mint some tokens to holders and vault
//     await mintTo(
//       provider.connection,
//       mintAuthority,
//       mintAddress,
//       holder1TokenAccount,
//       mintAuthority,
//       1000
//     );

//     await mintTo(
//       provider.connection,
//       mintAuthority,
//       mintAddress,
//       holder2TokenAccount,
//       mintAuthority,
//       2000
//     );

//     await mintTo(
//       provider.connection,
//       mintAuthority,
//       mintAddress,
//       rewardVault,
//       mintAuthority,
//       3000
//     );
//   });

//   it("Initializes the distributor", async () => {
//     try {
//       const tx = await program.methods
//         .initialize()
//         .accounts({
//           distributor: distributorPDA,
//           xyzMint: mintAddress,
//           authority: provider.wallet.publicKey,
//           systemProgram: SystemProgram.programId,
//         })
//         .signers([])
//         .rpc();

//       // Fetch the created distributor account
//       const distributorAccount = await program.account.distributor.fetch(
//         distributorPDA
//       );

//       // Verify the initialization
//       assert.ok(distributorAccount.authority.equals(provider.wallet.publicKey));
//       assert.ok(distributorAccount.xyzMint.equals(mintAddress));
//       assert.equal(distributorAccount.distributionInterval.toNumber(), new BN(600).toNumber());
      
//       console.log("Initialization transaction:", tx);
//     } catch (err) {
//       console.error("Error:", err);
//       throw err;
//     }
//   });

//   it("Distributes rewards correctly", async () => {
//     // First initialize
//     await program.methods
//       .initialize()
//       .accounts({
//         distributor: distributorPDA,
//         xyzMint: mintAddress,
//         authority: provider.wallet.publicKey,
//         systemProgram: SystemProgram.programId,
//       })
//       .signers([])
//       .rpc();

//     // Get initial balances
//     const initialHolder1Balance = (
//       await getAccount(provider.connection, holder1TokenAccount)
//     ).amount;
    
//     // Fast forward time (since we have a 10-minute interval)
//     await new Promise((resolve) => setTimeout(resolve, 1000));

//     try {
//       // Distribute rewards to holder1
//       const tx = await program.methods
//         .distributeRewards()
//         .accounts({
//           distributor: distributorPDA,
//           xyzMint: mintAddress,
//           holderTokenAccount: holder1TokenAccount,
//           rewardVault: rewardVault,
//           vaultAuthority: vaultAuthority,
//           holder: tokenHolder1.publicKey,
//           tokenProgram: TOKEN_PROGRAM_ID,
//         })
//         .signers([])
//         .rpc();

//       // Get final balances
//       const finalHolder1Balance = (
//         await getAccount(provider.connection, holder1TokenAccount)
//       ).amount;

//       // Verify the distribution
//       assert.isTrue(finalHolder1Balance > initialHolder1Balance);
      
//       console.log("Distribution transaction:", tx);
//       console.log("Rewards distributed:", finalHolder1Balance - initialHolder1Balance);
//     } catch (err) {
//       console.error("Error:", err);
//       throw err;
//     }
//   });

//   it("Fails to distribute before interval has passed", async () => {
//     // First initialize
//     await program.methods
//       .initialize()
//       .accounts({
//         distributor: distributorPDA,
//         xyzMint: mintAddress,
//         authority: provider.wallet.publicKey,
//         systemProgram: SystemProgram.programId,
//       })
//       .signers([])
//       .rpc();

//     try {
//       // Try immediate distribution
//       await program.methods
//         .distributeRewards()
//         .accounts({
//           distributor: distributorPDA,
//           xyzMint: mintAddress,
//           holderTokenAccount: holder1TokenAccount,
//           rewardVault: rewardVault,
//           vaultAuthority: vaultAuthority,
//           holder: tokenHolder1.publicKey,
//           tokenProgram: TOKEN_PROGRAM_ID,
//         })
//         .signers([])
//         .rpc();
      
//       assert.fail("Should have failed due to time interval");
//     } catch (err) {
//       assert.include(
//         err.toString(),
//         "Not enough time has passed since last distribution"
//       );
//     }
//   });

//   it("Distributes rewards proportionally", async () => {
//     // First initialize
//     await program.methods
//       .initialize()
//       .accounts({
//         distributor: distributorPDA,
//         xyzMint: mintAddress,
//         authority: provider.wallet.publicKey,
//         systemProgram: SystemProgram.programId,
//       })
//       .signers([])
//       .rpc();

//     // Get initial balances
//     const initialHolder1Balance = (
//       await getAccount(provider.connection, holder1TokenAccount)
//     ).amount;
//     const initialHolder2Balance = (
//       await getAccount(provider.connection, holder2TokenAccount)
//     ).amount;

//     // Fast forward time
//     await new Promise((resolve) => setTimeout(resolve, 1000));

//     // Distribute to both holders
//     await program.methods
//       .distributeRewards()
//       .accounts({
//         distributor: distributorPDA,
//         xyzMint: mintAddress,
//         holderTokenAccount: holder1TokenAccount,
//         rewardVault: rewardVault,
//         vaultAuthority: vaultAuthority,
//         holder: tokenHolder1.publicKey,
//         tokenProgram: TOKEN_PROGRAM_ID,
//       })
//       .signers([])
//       .rpc();

//     await program.methods
//       .distributeRewards()
//       .accounts({
//         distributor: distributorPDA,
//         xyzMint: mintAddress,
//         holderTokenAccount: holder2TokenAccount,
//         rewardVault: rewardVault,
//         vaultAuthority: vaultAuthority,
//         holder: tokenHolder2.publicKey,
//         tokenProgram: TOKEN_PROGRAM_ID,
//       })
//       .signers([])
//       .rpc();

//     // Get final balances
//     const finalHolder1Balance = (
//       await getAccount(provider.connection, holder1TokenAccount)
//     ).amount;
//     const finalHolder2Balance = (
//       await getAccount(provider.connection, holder2TokenAccount)
//     ).amount;

//     // Calculate rewards
//     const holder1Rewards = finalHolder1Balance - initialHolder1Balance;
//     const holder2Rewards = finalHolder2Balance - initialHolder2Balance;

//     // Verify proportional distribution (holder2 should get ~2x rewards as holder1)
//     const ratio = Number(holder2Rewards) / Number(holder1Rewards);
//     assert.approximately(ratio, 2, 0.1);
//   });
// });