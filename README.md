Solana Staking Program

This program allows users to stake SOL tokens, accumulate reward points over time based on the staked amount, and claim those points.

Features

- Create a PDA account to store user staking data.
- Stake SOL tokens to start earning points.
- Unstake SOL tokens to withdraw your stake.
- Claim accumulated reward points.
- Check current points without claiming.

How it works

- When you stake SOL, the program tracks your staked amount and the time.
- Points are calculated based on the amount staked and the time elapsed.
- You can unstake any amount up to your staked balance.
- Points can be claimed separately.
- The program uses PDAs to securely manage each user’s staking data.

Program Instructions

- create_pda_account: Creates a user-specific PDA account.
- stake: Stake a specified amount of lamports.
- unstake: Withdraw a specified amount from your stake.
- claim_points: Claim your accumulated points.
- get_points: View your current points and stake status.

Accounts

- StakeAccount: Stores owner public key, staked amount, total points, last update time, and bump seed.

Errors

- Invalid amounts, insufficient stake, unauthorized access, arithmetic overflow/underflow, and invalid timestamps are handled gracefully.
