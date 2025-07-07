use anchor_lang::prelude::*;

declare_id!("ELvizMGwAUf3qrBkFQ5HKMmTvPzee5thNRpJ5ko1Ai9J");

#[program]
pub mod stake_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
