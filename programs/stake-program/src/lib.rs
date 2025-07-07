use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("ELvizMGwAUf3qrBkFQ5HKMmTvPzee5thNRpJ5ko1Ai9J");

const SOLANA_PER_DAY: u64 = 1_000_000; // micropoiints
const SECONDS_PER_DAY: u64 = 86_400;
const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

#[program]
pub mod stake_program {

    use super::*;

    pub fn create_pda_account(ctx: Context<CreatePdaAcc>) -> Result<()> {
        // getting the mutable reference to the pda
        let pda_account = &mut ctx.accounts.pda_account;

        // gets current blockchain timestamp
        let clock = Clock::get()?;

        // initialize the pda fields
        pda_account.owner = ctx.accounts.payer.key();
        pda_account.staked_amount = 0;
        pda_account.total_points = 0;
        // sets last update time to now (convert i64 to u64)
        pda_account.last_updated_time = clock.unix_timestamp as u64;
        // after the creation the bumps are returned here
        pda_account.bump = ctx.bumps.pda_account;

        msg!("Pda account created successfully");
        Ok(())
    }
    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        // validate the input
        require!(amount > 0, StakeError::InvalidAmount);

        let clock = Clock::get()?;

        // scope mutable borrow of pda_account
        {
            let pda_account = &mut ctx.accounts.pda_account;

            // update points earned so far before changing staked amount
            update_points(pda_account, clock.unix_timestamp)?;

            // update pda staked amount (optional: move this inside if needed)
            pda_account.staked_amount = pda_account
                .staked_amount
                .checked_add(amount)
                .ok_or(StakeError::Overflow)?;
        } // mutable borrow ends here

        // transfer sol from the user to the pda
        let cpi = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.pda_account.to_account_info(),
            },
        );
        transfer(cpi, amount)?;

        msg!(
            "Staked {} lamports. Total staked: {} lamports",
            amount,
            ctx.accounts.pda_account.staked_amount
        );

        Ok(())
    }
    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        // unstake amount should be greater than 0
        require!(amount > 0, StakeError::InvalidAmount);

        let clock = Clock::get()?;

        {
            // pda mut ref scoped
            let pda_account = &mut ctx.accounts.pda_account;

            // staked should be greater than withdrawal
            require!(
                pda_account.staked_amount >= amount,
                StakeError::InsufficientStake
            );

            // update points earned so far before changing staked amount
            update_points(pda_account, clock.unix_timestamp)?;
        } // mutable borrow ends here

        let binding = ctx.accounts.user.key();
        let seeds = &[b"user1", binding.as_ref(), &[ctx.accounts.pda_account.bump]];
        let signer = &[&seeds[..]];

        // new transaction from pda to user
        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.pda_account.to_account_info(),
                to: ctx.accounts.user.to_account_info(),
            },
            signer,
        );
        transfer(cpi_context, amount)?;

        // update pda staked amount
        ctx.accounts.pda_account.staked_amount = ctx
            .accounts
            .pda_account
            .staked_amount
            .checked_sub(amount)
            .ok_or(StakeError::Underflow)?;

        msg!(
            "Unstaked {} lamports, remaining is {} lamports",
            amount,
            ctx.accounts.pda_account.staked_amount
        );

        Ok(())
    }

    pub fn claim_points(ctx: Context<ClaimPoints>) -> Result<()> {
        let pda_account = &mut ctx.accounts.pda_account;
        let clock = Clock::get()?;

        // update points to current time
        update_points(pda_account, clock.unix_timestamp)?;

        // convert micro-points to actual points
        let claimable_points = pda_account.total_points / 1_000_000;

        msg!("User has {} claimable points", claimable_points);

        // reset total_points after claiming
        pda_account.total_points = 0;

        Ok(())
    }

    pub fn get_points(ctx: Context<GetPoints>) -> Result<()> {
        let pda_account = &ctx.accounts.pda_account;
        let clock = Clock::get()?;

        // calculate pending points without updating account
        let time_elapsed = clock
            .unix_timestamp
            .checked_sub(pda_account.last_updated_time as i64)
            .ok_or(StakeError::InvalidTimestamp)? as u64;

        let new_points = calculate_points_earned(pda_account.staked_amount, time_elapsed)?;
        let current_total_points = pda_account
            .total_points
            .checked_add(new_points)
            .ok_or(StakeError::Overflow)?;

        // convert micro-points to actual points
        msg!(
            "Current points: {}, Staked amount: {} SOL",
            current_total_points / 1_000_000,
            pda_account.staked_amount / LAMPORTS_PER_SOL
        );

        Ok(())
    }
}

// helper: update accumulated points
fn update_points(pda_account: &mut StakeAccount, current_time: i64) -> Result<()> {
    let time_elapsed = current_time
        .checked_sub(pda_account.last_updated_time as i64)
        .ok_or(StakeError::InvalidTimestamp)? as u64;

    if time_elapsed > 0 && pda_account.staked_amount > 0 {
        let new_points = calculate_points_earned(pda_account.staked_amount, time_elapsed)?;
        pda_account.total_points = pda_account
            .total_points
            .checked_add(new_points)
            .ok_or(StakeError::Overflow)?;
    }

    pda_account.last_updated_time = current_time as u64;
    Ok(())
}

//  helper: core reward calculation
fn calculate_points_earned(staked_amount: u64, time_elapsed_seconds: u64) -> Result<u64> {
    let points = (staked_amount as u128)
        .checked_mul(time_elapsed_seconds as u128)
        .ok_or(StakeError::Overflow)?
        .checked_mul(SOLANA_PER_DAY as u128)
        .ok_or(StakeError::Overflow)?
        .checked_div(LAMPORTS_PER_SOL as u128)
        .ok_or(StakeError::Overflow)?
        .checked_div(SECONDS_PER_DAY as u128)
        .ok_or(StakeError::Overflow)?;

    Ok(points as u64)
}

// creating a pda that stores the users data
#[account]
pub struct StakeAccount {
    pub owner: Pubkey,
    pub staked_amount: u64,
    pub total_points: u64,
    pub last_updated_time: u64,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct CreatePdaAcc<'info> {
    // user paying for the account creation
    #[account(mut)]
    pub payer: Signer<'info>,

    // initialize the pda account
    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 8 + 8 + 8 + 1,
        seeds = [b"user1", payer.key().as_ref()],
        bump
    )]
    pub pda_account: Account<'info, StakeAccount>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct Stake<'info> {
    //user who is staking
    #[account(mut)]
    pub user: Signer<'info>,

    //users pda accoutn
    #[account(
        mut,
        seeds = [b"user1", user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key() @ StakeError::Unauthorized
    )]
    pub pda_account: Account<'info, StakeAccount>,

    //for transferring sol
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    // user who is unstaking
    #[account(mut)]
    pub user: Signer<'info>,

    //users pda accoutn
    #[account(
        mut,
        seeds = [b"user1", user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key() @ StakeError::Unauthorized
    )]
    pub pda_account: Account<'info, StakeAccount>,

    //for transferring sol
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct ClaimPoints<'info> {
    // user claiming points
    #[account(mut)]
    pub user: Signer<'info>,

    // user's pda account
    #[account(
        mut,
        seeds = [b"user1", user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key() @ StakeError::Unauthorized
    )]
    pub pda_account: Account<'info, StakeAccount>,
}
#[derive(Accounts)]
pub struct GetPoints<'info> {
    pub user: Signer<'info>,

    #[account(
        seeds = [b"user1", user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key() @ StakeError::Unauthorized
    )]
    pub pda_account: Account<'info, StakeAccount>,
}
#[error_code]
pub enum StakeError {
    #[msg("Amount must be greater than 0")]
    InvalidAmount,
    #[msg("Insufficient staked amount")]
    InsufficientStake,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Arithmetic underflow")]
    Underflow,
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
}
