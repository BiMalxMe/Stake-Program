use anchor_lang::prelude::*;
use anchor_lang::system_program::{Transfer,transfer};

declare_id!("ELvizMGwAUf3qrBkFQ5HKMmTvPzee5thNRpJ5ko1Ai9J");

const SOLANA_PER_DAY: u64 = 1_000_000;
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

        // Initialize the pda fields
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

    //staking
    pub fn stake(ctx : Context<Stake>,amount : u64) -> Result<()>{

        //validate the input
        require!(amount > 0 ,StakeError::InvalidAmount);

        //get mutable reference of the pda account
        let pda_account = &mut ctx.accounts.pda_account;
        let clock = Clock::get()?;

        //  Update points earned so far before changing staked amount
        update_points(pda_account, clock.unix_timestamp)?;

        //transfer sol from the user to the pda
        let cpi = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from : ctx.accounts.user.to_account_info(),
                to: ctx.accounts.pda_account.to_account_info(),
            }
            );
            transfer(cpi, amount)?;

             //  Update PDA staked amount
            pda_account.staked_amount = pda_account.staked_amount
            .checked_add(amount)
            .ok_or(StakeError::Overflow)?;
            msg!(
              "Staked {} lamports. Total staked: {} lamports",
                 amount,
                pda_account.staked_amount
            );

            Ok(())

    }
    pub fn unstake(ctx : Context<Unstake> , amount : u64) -> Result<()>{

        // unstake amount should be gerater then 0
        require!(amount > 0, StakeError::InvalidAmount);

        //Pda mut ref
        let pda_account = &mut ctx.accounts.pda_account;
        let clock = Clock::get()?;

        // staked should be grater than withdrawll
        require!(
          pda_account.staked_amount >= amount,
            StakeError::InsufficientStake
         );

        //Update points earned so far before changing staked amount
        update_points(pda_account, clock.unix_timestamp)?;

        let seeds = &[
            b"user1",
            ctx.accounts.user.key().as_ref(),
            &[pda_account.bump],
        ];
        //converts to (&[&[u8]]),
        let signer = &[&seeds[..]];

        //new transaction from pda to user
        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.pda_account.to_account_info(),
                to: ctx.accounts.user.to_account_info(),
            },
            signer,
        );
        transfer(cpi_context, amount)?;

        // update PDA staked amount
        pda_account.staked_amount = pda_account.staked_amount
        //subtract
         .checked_sub(amount)
           .ok_or(StakeError::Underflow)?;

        msg!(
            "Unstaked {} lamports reaminaing is {} lamports",
            amount,
            pda_account.staked_amount
        );

        Ok(())
    }
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

    // Initialize the pda account
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
pub struct Stake<'info>{
    //user who is staking
    #[account(mut)]
    pub user : Signer<'info>,

    //users pda accoutn
    #[account(
        mut,
        seeds = [b"user1", user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key() @ StakeError::Unauthorized
    )]
    pub pda_account : Account<'info,StakeAccount>,

    //for transferring sol
    pub system_program : Program<'info,System>,

}

#[derive(Accounts)]
pub struct Unstake<'info>{
    // User who is unstaking
    #[account(mut)]
    pub user : Signer<'info>,

    //users pda accoutn
    #[account(
        mut,
        seeds = [b"user1", user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key() @ StakeError::Unauthorized
    )]
    pub pda_account : Account<'info,StakeAccount>,

    //for transferring sol
    pub system_program : Program<'info,System>,

}