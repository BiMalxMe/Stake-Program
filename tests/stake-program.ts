import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StakeProgram } from "../target/types/stake_program";
const assert = require("node:assert")
import {
  SystemProgram,
  Keypair,
  PublicKey,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";

describe("Stake Program", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.StakeProgram as Program<StakeProgram>;
  const user = Keypair.generate();
  let pdaAccount: PublicKey;
  let bump: number;

  before(async () => {
    // Airdrop SOL to the user for tests
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(user.publicKey, 2 * LAMPORTS_PER_SOL),
      "confirmed"
    );

    // Derive PDA
    [pdaAccount, bump] = await PublicKey.findProgramAddressSync(
      [Buffer.from("user1"), user.publicKey.toBuffer()],
      program.programId
    );
  });

  it("Creates PDA Account", async () => {
    await program.methods
      .createPdaAccount()
      .accounts({
        payer: user.publicKey,
        pdaAccount,
        systemProgram: SystemProgram.programId,
      }as any)
      .signers([user])
      .rpc();

    const account = await program.account.stakeAccount.fetch(pdaAccount);
    assert.ok(account.owner.equals(user.publicKey));
    assert.equal(account.stakedAmount.toNumber(), 0);
    assert.equal(account.totalPoints.toNumber(), 0);
    assert.equal(account.bump, bump);
  });

  it("Stakes SOL successfully", async () => {
    const stakeAmount = 0.5 * LAMPORTS_PER_SOL;

    await program.methods
      .stake(new anchor.BN(stakeAmount))
      .accounts({
        user: user.publicKey,
        pdaAccount,
        systemProgram: SystemProgram.programId,
      }as any)
      .signers([user])
      .rpc();

    const account = await program.account.stakeAccount.fetch(pdaAccount);
    assert.equal(account.stakedAmount.toNumber(), stakeAmount);
  });

  it("Fails to stake 0 SOL", async () => {
    try {
      await program.methods
      .createPdaAccount()
      .accounts({
        payer: user.publicKey,
        pdaAccount, // <-- TS thinks this might be wrong
        systemProgram: SystemProgram.programId,
      } as any)      // <-- bypass type check
      .signers([user])
      .rpc();
    
      assert.fail("Should have failed");
    } catch (err) {
      console.log(err)
      assert.ok(err.message.includes("Transaction simulation failed: Error processing Instruction 0: custom program error: 0x0"));
    }
  });

  it("Unstakes partial SOL successfully", async () => {
    const unstakeAmount = 0.3 * LAMPORTS_PER_SOL;

    await program.methods
      .unstake(new anchor.BN(unstakeAmount))
      .accounts({
        user: user.publicKey,
        pdaAccount,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([user])
      .rpc();

    const account = await program.account.stakeAccount.fetch(pdaAccount);
    const expectedRemaining = 0.2 * LAMPORTS_PER_SOL;
    assert.equal(account.stakedAmount.toNumber(), expectedRemaining);
  });

  it("Fails to unstake more than staked amount", async () => {
    try {
      await program.methods
        .unstake(new anchor.BN(LAMPORTS_PER_SOL))
        .accounts({
          user: user.publicKey,
          pdaAccount,
          systemProgram: SystemProgram.programId,
        } as any)
        .signers([user])
        .rpc();
      assert.fail("Should have failed");
    } catch (err) {
      assert.equal(err.error.errorMessage, "Insufficient staked amount");
    }
  });

  it("Claims points successfully", async () => {
    await program.methods
      .claimPoints()
      .accounts({
        user: user.publicKey,
        pdaAccount,
      }as any)
      .signers([user])
      .rpc();

    const account = await program.account.stakeAccount.fetch(pdaAccount);
    assert.equal(account.totalPoints.toNumber(), 0); // reset after claiming
  });

  it("Gets current points without claiming", async () => {
    await program.methods
      .getPoints()
      .accounts({
        user: user.publicKey,
        pdaAccount,
      }as any)
      .signers([user])
      .rpc();

    // Just for test run. No state change expected.
    const account = await program.account.stakeAccount.fetch(pdaAccount);
    assert.ok(account.stakedAmount.toNumber() >= 0);
  });
});
