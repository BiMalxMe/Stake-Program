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
        let pda_account = &mut ctx.accounts.pda_account;
        let clock = Clock::get()?;

        pda_account.owner = ctx.accounts.payer.key();
        pda_account.staked_amount = 0;
        pda_account.total_points = 0;
        pda_account.last_updated_time = clock.unix_timestamp as u64;
        pda_account.bump = ctx.bumps.pda_account;

        msg!("Pda account created successfully for owner: {}", pda_account.owner);
        msg!("Initial staked_amount: {}, total_points: {}, last_updated_time: {}, bump: {}", 
            pda_account.staked_amount, 
            pda_account.total_points, 
            pda_account.last_updated_time, 
            pda_account.bump
        );
        Ok(())
    }
    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        require!(amount > 0, StakeError::InvalidAmount);

        let clock = Clock::get()?;

        {
            let pda_account = &mut ctx.accounts.pda_account;

            msg!("Updating points before staking. Current total_points: {}, last_updated_time: {}", pda_account.total_points, pda_account.last_updated_time);
            update_points(pda_account, clock.unix_timestamp)?;

            let prev_staked = pda_account.staked_amount;
            pda_account.staked_amount = pda_account
                .staked_amount
                .checked_add(amount)
                .ok_or(StakeError::Overflow)?;
            msg!(
                "Staked amount updated from {} to {}",
                prev_staked,
                pda_account.staked_amount
            );
        }

        msg!(
            "Transferring {} lamports from user {} to PDA {}",
            amount,
            ctx.accounts.user.key(),
            ctx.accounts.pda_account.key()
        );
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
        require!(amount > 0, StakeError::InvalidAmount);

        let clock = Clock::get()?;

        {
            let pda_account = &mut ctx.accounts.pda_account;

            msg!(
                "Attempting to unstake {} lamports. Current staked_amount: {}",
                amount,
                pda_account.staked_amount
            );
            require!(
                pda_account.staked_amount >= amount,
                StakeError::InsufficientStake
            );

            msg!("Updating points before unstaking. Current total_points: {}, last_updated_time: {}", pda_account.total_points, pda_account.last_updated_time);
            update_points(pda_account, clock.unix_timestamp)?;
        }

        let binding = ctx.accounts.user.key();
        let seeds = &[b"user1", binding.as_ref(), &[ctx.accounts.pda_account.bump]];
        let signer = &[&seeds[..]];

        msg!(
            "Transferring {} lamports from PDA {} to user {}",
            amount,
            ctx.accounts.pda_account.key(),
            ctx.accounts.user.key()
        );
        **ctx.accounts.pda_account.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.user.try_borrow_mut_lamports()? += amount;

        let prev_staked = ctx.accounts.pda_account.staked_amount;
        ctx.accounts.pda_account.staked_amount = ctx
            .accounts
            .pda_account
            .staked_amount
            .checked_sub(amount)
            .ok_or(StakeError::Underflow)?;

        msg!(
            "Unstaked {} lamports, remaining is {} lamports (was {})",
            amount,
            ctx.accounts.pda_account.staked_amount,
            prev_staked
        );

        Ok(())
    }

    pub fn claim_points(ctx: Context<ClaimPoints>) -> Result<()> {
        let pda_account = &mut ctx.accounts.pda_account;
        let clock = Clock::get()?;

        msg!(
            "Claiming points for user {}. Current total_points: {}, last_updated_time: {}",
            ctx.accounts.user.key(),
            pda_account.total_points,
            pda_account.last_updated_time
        );
        update_points(pda_account, clock.unix_timestamp)?;

        let claimable_points = pda_account.total_points / 1_000_000;

        msg!("User {} has {} claimable points", ctx.accounts.user.key(), claimable_points);

        pda_account.total_points = 0;
        msg!("total_points reset to 0 after claim");

        Ok(())
    }

    pub fn get_points(ctx: Context<GetPoints>) -> Result<()> {
        let pda_account = &ctx.accounts.pda_account;
        let clock = Clock::get()?;

        msg!(
            "Getting points for user {}. Current total_points: {}, last_updated_time: {}, staked_amount: {}",
            ctx.accounts.user.key(),
            pda_account.total_points,
            pda_account.last_updated_time,
            pda_account.staked_amount
        );

        let time_elapsed = clock
            .unix_timestamp
            .checked_sub(pda_account.last_updated_time as i64)
            .ok_or(StakeError::InvalidTimestamp)? as u64;

        msg!("Time elapsed since last update: {} seconds", time_elapsed);

        let new_points = calculate_points_earned(pda_account.staked_amount, time_elapsed)?;
        msg!("New points earned since last update: {}", new_points);

        let current_total_points = pda_account
            .total_points
            .checked_add(new_points)
            .ok_or(StakeError::Overflow)?;

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

    msg!(
        "update_points called. time_elapsed: {}, staked_amount: {}, total_points before: {}",
        time_elapsed,
        pda_account.staked_amount,
        pda_account.total_points
    );

    if time_elapsed > 0 && pda_account.staked_amount > 0 {
        let new_points = calculate_points_earned(pda_account.staked_amount, time_elapsed)?;
        msg!("Calculated new_points: {}", new_points);
        pda_account.total_points = pda_account
            .total_points
            .checked_add(new_points)
            .ok_or(StakeError::Overflow)?;
        msg!("total_points after update: {}", pda_account.total_points);
    } else {
        msg!("No points updated (either no time elapsed or staked_amount is zero)");
    }

    pda_account.last_updated_time = current_time as u64;
    msg!("last_updated_time set to {}", pda_account.last_updated_time);
    Ok(())
}

//  helper: core reward calculation
fn calculate_points_earned(staked_amount: u64, time_elapsed_seconds: u64) -> Result<u64> {
    msg!(
        "Calculating points earned. staked_amount: {}, time_elapsed_seconds: {}",
        staked_amount,
        time_elapsed_seconds
    );
    let points = (staked_amount as u128)
        .checked_mul(time_elapsed_seconds as u128)
        .ok_or(StakeError::Overflow)?
        .checked_mul(SOLANA_PER_DAY as u128)
        .ok_or(StakeError::Overflow)?
        .checked_div(LAMPORTS_PER_SOL as u128)
        .ok_or(StakeError::Overflow)?
        .checked_div(SECONDS_PER_DAY as u128)
        .ok_or(StakeError::Overflow)?;

    msg!("Points earned (micropoints): {}", points);

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
    #[account(mut)]
    pub payer: Signer<'info>,

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
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user1", user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key() @ StakeError::Unauthorized
    )]
    pub pda_account: Account<'info, StakeAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user1", user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key() @ StakeError::Unauthorized
    )]
    pub pda_account: Account<'info, StakeAccount>,

    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct ClaimPoints<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

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
