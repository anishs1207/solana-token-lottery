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

/// Accounts required to commit a randomness account for the lottery.
///
/// Ensures:
/// 1. Only the authority of the lottery can commit the randomness.
/// 2. The randomness account is valid and has not been revealed previously.
#[derive(Accounts)]
pub struct CommitWinner<'info> {
    /// The account paying transaction fees.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The main lottery state account.
    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    /// Randomness account from Switchboard.
    /// CHECK: The account's data is validated manually within the handler.
    pub randomness_account_data: UncheckedAccount<'info>,

    /// System program for lamports transfers.
    pub system_program: Program<'info, System>,
}

pub fn process_commit_a_winner(ctx: Context<CommitWinner>) -> Result<()> {
    let clock = Clock::get()?;
    let token_lottery = &mut ctx.accounts.token_lottery;
    if ctx.accounts.payer.key() != token_lottery.authority {
        return Err(ErrorCode::NotAuthorized.into());
    }

    let randomness_data =
        RandomnessAccountData::parse(ctx.accounts.randomness_account_data.data.borrow()).unwrap();

    if randomness_data.seed_slot != clock.slot - 1 {
        return Err(ErrorCode::RandomnessAlreadyRevealed.into());
    }

    token_lottery.randomness_account = ctx.accounts.randomness_account_data.key();

    Ok(())
}
