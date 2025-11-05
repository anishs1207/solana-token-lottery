use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct TokenLottery {
    /// The bump seed used for deriving the PDA address of this account.
    pub bump: u8,

    /// The index or identifier of the winning ticket.
    /// Defaults to `0` until a winner is selected.
    pub winner: u64,

    /// A flag indicating whether the winner has been chosen.
    /// `true` once the random draw has been completed.
    pub winner_chosen: bool,

    /// The UNIX timestamp marking when the lottery started.
    pub lottery_start: u64,

    /// The UNIX timestamp marking when the lottery ends.
    pub lottery_end: u64,

    /// The total amount of SOL (in lamports) accumulated in the lottery pot.
    /// This field stores metadata only — the actual SOL should be held
    /// in a separate escrow or vault account for safety.
    pub lottery_pot_amount: u64,

    /// The total number of tickets issued for this lottery.
    pub ticket_num: u64,

    /// The price (in lamports) required to purchase a single ticket.
    pub price: u64,

    /// The public key of the randomness oracle or account
    /// used to generate verifiable random numbers
    pub randomness_account: Pubkey,

    /// The authority or admin responsible for managing this lottery.
    pub authority: Pubkey,
}

// @self-notes: defining all the state programs/accounst here (#[account] is used for it)
// it defined the state programs here
