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

/// Accounts required to initialize the Token Lottery configuration.
/// This sets up the main lottery account on-chain with initial parameters.
#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    /// The account paying for account creation and fees.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The TokenLottery state account that stores lottery information.
    #[account(
        init,
        payer = payer,
        space = 8 + TokenLottery::INIT_SPACE,
        seeds = [b"token_lottery".as_ref()],
        bump
    )]
    pub token_lottery: Box<Account<'info, TokenLottery>>,

    /// System program to create accounts.
    pub system_program: Program<'info, System>,
}

/// Accounts required to initialize a new lottery collection.
/// This includes the mint, token account, metadata, and master edition accounts.
#[derive(Accounts)]
pub struct InitializeLottery<'info> {
    /// The account paying for account creation and fees.
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = collection_mint,
        mint::freeze_authority = collection_mint,
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: Box<InterfaceAccount<'info, Mint>>,

    /// Metadata account for the collection (initialized by Metaplex program).
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        seeds = [b"collection_token_account".as_ref()],
        bump,
        token::mint = collection_mint,
        token::authority = collection_token_account
    )]
    pub collection_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Token program interface.
    pub token_program: Interface<'info, TokenInterface>,

    /// Associated token program interface.
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// System program interface.
    pub system_program: Program<'info, System>,

    /// Metaplex token metadata program.
    pub token_metadata_program: Program<'info, Metadata>,

    /// Rent sysvar for account creation.
    pub rent: Sysvar<'info, Rent>,
}

/// Initializes the main Token Lottery account with start/end times, ticket price,
/// and sets the authority.
///
/// # Arguments
/// * `ctx` - Context holding the InitializeConfig accounts
/// * `start` - UNIX timestamp for lottery start
/// * `end` - UNIX timestamp for lottery end
/// * `price` - Ticket price in lamports
pub fn process_initialize_config(
    ctx: Context<InitializeConifg>,
    start: u64,
    end: u64,
    price: u64,
) -> Result<()> {
    let token_lottery = &mut ctx.accounts.token_lottery;
    token_lottery.bump = ctx.bumps.token_lottery;
    token_lottery.lottery_start = start;
    token_lottery.lottery_end = end;
    token_lottery.price = price;
    token_lottery.authority = ctx.accounts.payer.key();
    token_lottery.randomness_account = Pubkey::default();
    token_lottery.ticket_num = 0;
    token_lottery.winner_chosen = false;
    Ok(())
}

/// Initializes a new lottery collection by creating:
/// - the mint account
/// - the collection token account
/// - the metadata account
/// - the master edition account
/// - signs the metadata for the collection
///
/// # Arguments
/// * `ctx` - Context holding the InitializeLottery accounts
pub fn process_initialize_lottery(ctx: Context<InitializeLottery>) -> Result<()> {
    let signer_seeds: &[&[&[u8]]] = &[&[b"collection_mint".as_ref(), &[ctx.bumps.collection_mint]]];

    msg!("Creating mint accounts");
    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.collection_mint.to_account_info(),
                to: ctx.accounts.collection_token_account.to_account_info(),
                authority: ctx.accounts.collection_mint.to_account_info(),
            },
            signer_seeds,
        ),
        1,
    )?;

    msg!("Creating metadata accounts");
    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata.to_account_info(),
                mint: ctx.accounts.collection_mint.to_account_info(),
                mint_authority: ctx.accounts.collection_mint.to_account_info(), // use pda mint address as mint authority
                update_authority: ctx.accounts.collection_mint.to_account_info(), // use pda mint as update authority
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &signer_seeds,
        ),
        DataV2 {
            name: NAME.to_string(),
            symbol: SYMBOL.to_string(),
            uri: URI.to_string(),
            seller_fee_basis_points: 0,
            creators: Some(vec![Creator {
                address: ctx.accounts.collection_mint.key(),
                verified: false,
                share: 100,
            }]),
            collection: None,
            uses: None,
        },
        true,
        true,
        Some(CollectionDetails::V1 { size: 0 }), // set as collection nft
    )?;

    msg!("Creating Master edition accounts");
    create_master_edition_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                payer: ctx.accounts.payer.to_account_info(),
                mint: ctx.accounts.collection_mint.to_account_info(),
                edition: ctx.accounts.master_edition.to_account_info(),
                mint_authority: ctx.accounts.collection_mint.to_account_info(),
                update_authority: ctx.accounts.collection_mint.to_account_info(),
                metadata: ctx.accounts.metadata.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &signer_seeds,
        ),
        Some(0),
    )?;

    msg!("verifying collection");
    sign_metadata(CpiContext::new_with_signer(
        ctx.accounts.token_metadata_program.to_account_info(),
        SignMetadata {
            creator: ctx.accounts.collection_mint.to_account_info(),
            metadata: ctx.accounts.metadata.to_account_info(),
        },
        &signer_seeds,
    ))?;

    Ok(())
}
