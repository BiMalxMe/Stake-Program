use anchor_lang::prelude::*;

declare_id!("ELvizMGwAUf3qrBkFQ5HKMmTvPzee5thNRpJ5ko1Ai9J");

const  SOLANA_PER_DAY : u64 = 1_000_000;
const SEONDS_PER_DAY :u64 = 86_400;
const LAMPORTS_PER_SOL :u64 = 1_000_000_000;

#[program]
pub mod stake_program {
    use super::*;
    
    // pub fn create_pda_account(ctx : Context<CreatePdaAcc>) -> Result<()>{

    // }
}

//creatina a pda that stores the users data
#[account]
pub struct StakeAccount {
    pub owner : Pubkey,
    pub staked_amount : u64,
    pub total_points : u64,
    pub last_updata_time : u64,
    pub bump : u8
}

pub struct CreatePdaAcc<'info>{
    //user paying for the accouunt creation
    #[account(mut)]
    pub payer : Signer<'info>,
    
    //Initialize the pda account
    #[account(
        init,
        payer = payer ,
        space : 8 + 32 + 8 + 8 + 8 + 1,
        seeds = [b"user",payer.key().as_ref()],
        bump
    )]
    pub pda_account : Account<'info,StakeAccount>,
    pub system_program : Program<'info,System>
}