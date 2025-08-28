# Solana Staking dApp (Devnet)

A beginner-friendly staking dApp built on **Solana Devnet**, allowing users to stake SOL, earn time-based reward points, view their points, unstake, and claim rewards.  

This project was built as a submission for the **Superteam Nepal Solana Mini-Hack Bounty**, with the goal of showcasing how Nepali developers can build useful on-chain applications and contribute to the Solana ecosystem. 🚀

---

## 🎥 Demo Video
👉 [Watch on YouTube](https://youtu.be/iWgUivVBK4M)


---

## 🚀 Live Demo
👉 [Try the Staking dApp](https://bimalxstake.vercel.app)



## 📝 Problem Statement
Staking on Solana often lacks beginner-friendly interfaces and clarity around reward mechanisms.  

This dApp solves that by offering a **simple, intuitive interface** where users can stake, track, and claim rewards easily. It’s designed as an educational prototype to show how staking logic works on-chain while being extendable for real DeFi applications.

---

## 🛠️ Built With
- **Rust / Anchor** — smart contract framework  
- **Solana Web3.js** & **@coral-xyz/anchor** — blockchain client libraries  
- **TypeScript / React / Next.js** — frontend  
- **Phantom / Solflare Wallets** — wallet integration  

---

## ✨ Features
- **PDA-based staking account** (per user)  
- **Stake SOL** to start earning points (calculated based on staked amount × time)  
- **Claim reward points** independently  
- **Unstake SOL** (partial or full)  
- **Check points anytime** without claiming  

---

## ⚙️ Architecture (How It Works)
1. **create_pda_account** → Initializes a PDA for each user’s staking data (owner pubkey, staked amount, points, last update time).  
2. **stake** → Adds lamports to PDA, updates timestamp.  
3. **unstake** → Withdraws user’s SOL, updates balance.  
4. **claim_points** → Calculates and distributes accumulated reward points.  
5. **get_points** → Lets user check balance & rewards without claiming.  

---

## 🖥️ Setup Instructions

### Prerequisites
- Node.js & npm (or Yarn/PNPM)  
- Rust & Anchor CLI  
- Solana CLI (configured to **Devnet**)  
- Phantom/compatible wallet  

### Clone & Install
```bash
git clone https://github.com/BiMalxMe/Stake-Program.git
cd Stake-Program

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
