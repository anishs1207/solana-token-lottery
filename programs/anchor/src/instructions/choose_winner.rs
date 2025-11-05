use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::metadata::{
    create_master_edition_v3, create_metadata_accounts_v3,
    mpl_token_metadata::types::{CollectionDetails, Creator, DataV2},
    set_and_verify_sized_collection_item, sign_metadata, CreateMasterEditionV3,
    CreateMetadataAccountsV3, Metadata, MetadataAccount, SetAndVerifySizedCollectionItem,
    SignMetadata,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface},
};
use switchboard_on_demand::accounts::RandomnessAccountData;

/// Accounts required to choose a lottery winner.
///
/// This ensures that:
/// 1. Only the authorized lottery authority can pick a winner.
/// 2. The randomness account provided matches the lottery.
/// 3. The lottery period has ended.
/// 4. A winner hasn't already been chosen.
#[derive(Accounts)]
pub struct ChooseWinner<'info> {
    /// Account paying for any transaction fees.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The main lottery state account.
    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    /// The randomness oracle account providing verifiable randomness.
    /// CHECK: The account's data is validated manually within the handler.
    pub randomness_account_data: UncheckedAccount<'info>,

    /// System program for account operations.
    pub system_program: Program<'info, System>,
}

pub fn process_choose_a_winner(ctx: Context<ChooseWinner>) -> Result<()> {
    let clock = Clock::get()?;
    let token_lottery = &mut ctx.accounts.token_lottery;

    if ctx.accounts.randomness_account_data.key() != token_lottery.randomness_account {
        return Err(ErrorCode::IncorrectRandomnessAccount.into());
    }
    if ctx.accounts.payer.key() != token_lottery.authority {
        return Err(ErrorCode::NotAuthorized.into());
    }
    if clock.slot < token_lottery.lottery_end {
        msg!("Current slot: {}", clock.slot);
        msg!("End slot: {}", token_lottery.lottery_end);
        return Err(ErrorCode::LotteryNotCompleted.into());
    }
    require!(
        token_lottery.winner_chosen == false,
        ErrorCode::WinnerChosen
    );

    let randomness_data =
        RandomnessAccountData::parse(ctx.accounts.randomness_account_data.data.borrow()).unwrap();
    let revealed_random_value = randomness_data
        .get_value(&clock)
        .map_err(|_| ErrorCode::RandomnessNotResolved)?;

    msg!("Randomness result: {}", revealed_random_value[0]);
    msg!("Ticket num: {}", token_lottery.ticket_num);

    let randomness_result = revealed_random_value[0] as u64 % token_lottery.ticket_num;

    msg!("Winner: {}", randomness_result);

    token_lottery.winner = randomness_result;
    token_lottery.winner_chosen = true;

    Ok(())
}
