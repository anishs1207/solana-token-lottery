use anchor_lang::prelude::*;
use instructions::*;

mod constants;
mod error;
mod instructions;
mod state;

declare_id!("2RTh2Y4e2N421EbSnUYTKdGqDHJH7etxZb3VrWDMpNMY");

#[program]
pub mod token_lottery {
    use super::*;

    pub fn initialize_config(
        ctx: Context<InitializeConifg>,
        start: u64,
        end: u64,
        price: u64,
    ) -> Result<()> {
        process_initialize_config(ctx, start, end, price)
    }

    pub fn initialize_lottery(ctx: Context<InitializeLottery>) -> Result<()> {
        process_initialize_lottery(ctx)
    }

    pub fn buy_ticket(ctx: Context<BuyTicket>) -> Result<()> {
        process_buy_ticket(ctx)
    }

    pub fn commit_a_winner(ctx: Context<CommitWinner>) -> Result<()> {
        process_commit_a_winner(ctx)
    }

    pub fn choose_a_winner(ctx: Context<ChooseWinner>) -> Result<()> {
        process_choose_a_winner(ctx)
    }

    pub fn claim_prize(ctx: Context<ClaimPrize>) -> Result<()> {
        process_claim_prize(ctx)
    }
}
