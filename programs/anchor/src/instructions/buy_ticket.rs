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

/// Accounts required to buy a lottery ticket.
/// Handles:
/// - Payment transfer
/// - Ticket minting
/// - Metadata creation
/// - Master edition creation
/// - Collection verification
#[derive(Accounts)]
pub struct BuyTicket<'info> {
    /// The account paying for the ticket.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// TokenLottery state account tracking the current lottery.
    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    /// Mint for the specific ticket being purchased.
    #[account(
        init,
        payer = payer,
        seeds = [token_lottery.ticket_num.to_le_bytes().as_ref()],
        bump,
        mint::decimals = 0,
        mint::authority = collection_mint,
        mint::freeze_authority = collection_mint,
        mint::token_program = token_program
    )]
    pub ticket_mint: InterfaceAccount<'info, Mint>,

    /// Destination token account to receive the minted ticket.
    #[account(
        init,
        payer = payer,
        associated_token::mint = ticket_mint,
        associated_token::authority = payer,
        associated_token::token_program = token_program,
    )]
    pub destination: InterfaceAccount<'info, TokenAccount>,

    /// Metadata account for the minted ticket.
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), 
        ticket_mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata: UncheckedAccount<'info>,

    /// Master edition account for the ticket NFT.
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), 
            ticket_mint.key().as_ref(), b"edition"],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub master_edition: UncheckedAccount<'info>,

    /// Metadata account of the collection the ticket belongs to.
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), collection_mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub collection_metadata: UncheckedAccount<'info>,

    /// Master edition account of the collection.
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), 
            collection_mint.key().as_ref(), b"edition"],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub collection_master_edition: UncheckedAccount<'info>,

    /// Mint account of the collection.
    #[account(
        mut,
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,

    /// Associated token program
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Token program interface
    pub token_program: Interface<'info, TokenInterface>,

    /// System program interface
    pub system_program: Program<'info, System>,

    /// Metaplex token metadata program
    pub token_metadata_program: Program<'info, Metadata>,

    /// Rent sysvar for account creation
    pub rent: Sysvar<'info, Rent>,
}

/// Buys a lottery ticket for the caller.
///
/// Steps performed:
/// 1. Check if the lottery is currently open.
/// 2. Transfer SOL from payer to the lottery pot.
/// 3. Mint the NFT ticket.
/// 4. Create metadata for the ticket.
/// 5. Create the master edition.
/// 6. Verify the NFT as part of the collection.
/// 7. Increment the lottery ticket counter.
///
/// # Arguments
/// * `ctx` - Context containing BuyTicket accounts
pub fn process_buy_ticket(ctx: Context<BuyTicket>) -> Result<()> {
    let clock = Clock::get()?;
    let ticket_name = NAME.to_owned() + ctx.accounts.token_lottery.ticket_num.to_string().as_str();

    if clock.slot < ctx.accounts.token_lottery.lottery_start
        || clock.slot > ctx.accounts.token_lottery.lottery_end
    {
        return Err(ErrorCode::LotteryNotOpen.into());
    }

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.token_lottery.to_account_info(),
            },
        ),
        ctx.accounts.token_lottery.price,
    )?;

    ctx.accounts.token_lottery.lottery_pot_amount += ctx.accounts.token_lottery.price;

    let signer_seeds: &[&[&[u8]]] = &[&[b"collection_mint".as_ref(), &[ctx.bumps.collection_mint]]];

    // Mint Ticket
    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.ticket_mint.to_account_info(),
                to: ctx.accounts.destination.to_account_info(),
                authority: ctx.accounts.collection_mint.to_account_info(),
            },
            signer_seeds,
        ),
        1,
    )?;

    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata.to_account_info(),
                mint: ctx.accounts.ticket_mint.to_account_info(),
                mint_authority: ctx.accounts.collection_mint.to_account_info(),
                update_authority: ctx.accounts.collection_mint.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &signer_seeds,
        ),
        DataV2 {
            name: ticket_name,
            symbol: SYMBOL.to_string(),
            uri: URI.to_string(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        true,
        true,
        None,
    )?;

    create_master_edition_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                payer: ctx.accounts.payer.to_account_info(),
                mint: ctx.accounts.ticket_mint.to_account_info(),
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

    // verify nft as part of collection
    set_and_verify_sized_collection_item(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            SetAndVerifySizedCollectionItem {
                metadata: ctx.accounts.metadata.to_account_info(),
                collection_authority: ctx.accounts.collection_mint.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.collection_mint.to_account_info(),
                collection_mint: ctx.accounts.collection_mint.to_account_info(),
                collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
                collection_master_edition: ctx.accounts.collection_master_edition.to_account_info(),
            },
            &signer_seeds,
        ),
        None,
    )?;

    ctx.accounts.token_lottery.ticket_num += 1;

    Ok(())
}
