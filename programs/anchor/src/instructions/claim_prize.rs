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

/// Accounts required for claiming the lottery prize.
///
/// Ensures:
/// 1. Only the holder of the winning ticket can claim the prize.
/// 2. The ticket is verified as part of the correct NFT collection.
/// 3. The lottery winner has been selected.
/// 4. Lamports are correctly transferred to the winner.
#[derive(Accounts, Accounts)]
pub struct ClaimPrize<'info> {
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

    /// The collection mint used for lottery tickets.
    #[account(
        mut,
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,

    /// The NFT mint representing the winner's ticket.
    #[account(
        seeds = [token_lottery.winner.to_le_bytes().as_ref()],
        bump,
    )]
    pub ticket_mint: InterfaceAccount<'info, Mint>,

    /// Metadata account for the winner's ticket NFT.
    #[account(
        seeds = [b"metadata", token_metadata_program.key().as_ref(), ticket_mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata: Account<'info, MetadataAccount>,

    /// The token account of the winner that will receive the prize.
    #[account(
        associated_token::mint = ticket_mint,
        associated_token::authority = payer,
        associated_token::token_program = token_program,
    )]
    pub destination: InterfaceAccount<'info, TokenAccount>,

    /// Metadata account for the NFT collection.
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), collection_mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub collection_metadata: Account<'info, MetadataAccount>,

    /// Token program for transferring tokens.
    pub token_program: Interface<'info, TokenInterface>,

    /// System program for lamports transfer.
    pub system_program: Program<'info, System>,

    /// Metadata program for verifying NFTs.
    pub token_metadata_program: Program<'info, Metadata>,
}

/// Processes the prize claim for the winner.
///
/// Steps:
/// 1. Verify that a winner has been chosen.
/// 2. Validate that the ticket NFT belongs to the correct collection and matches the winning ticket.
/// 3. Ensure the caller owns the winning ticket.
/// 4. Transfer the lottery pot amount to the winner and reset the pot to zero.
///
/// # Arguments
/// * `ctx` - Context containing `ClaimPrize` accounts
pub fn process_claim_prize(ctx: Context<ClaimPrize>) -> Result<()> {
    // Check if winner has been chosen
    msg!(
        "Winner chosen: {}",
        ctx.accounts.token_lottery.winner_chosen
    );
    require!(
        ctx.accounts.token_lottery.winner_chosen,
        ErrorCode::WinnerNotChosen
    );

    // Check if token is a part of the collection
    require!(
        ctx.accounts.metadata.collection.as_ref().unwrap().verified,
        ErrorCode::NotVerifiedTicket
    );
    require!(
        ctx.accounts.metadata.collection.as_ref().unwrap().key
            == ctx.accounts.collection_mint.key(),
        ErrorCode::IncorrectTicket
    );

    let ticket_name = NAME.to_owned() + &ctx.accounts.token_lottery.winner.to_string();
    let metadata_name = ctx.accounts.metadata.name.replace("\u{0}", "");

    msg!("Ticket name: {}", ticket_name);
    msg!("Metdata name: {}", metadata_name);

    // Check if the winner has the winning ticket
    require!(metadata_name == ticket_name, ErrorCode::IncorrectTicket);
    require!(
        ctx.accounts.destination.amount > 0,
        ErrorCode::IncorrectTicket
    );

    **ctx
        .accounts
        .token_lottery
        .to_account_info()
        .try_borrow_mut_lamports()? -= ctx.accounts.token_lottery.lottery_pot_amount;
    **ctx.accounts.payer.try_borrow_mut_lamports()? +=
        ctx.accounts.token_lottery.lottery_pot_amount;

    ctx.accounts.token_lottery.lottery_pot_amount = 0;

    Ok(())
}
