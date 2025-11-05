/// Module containing program-wide constants such as token names, URIs,
/// symbols, and lottery configuration parameters.
pub mod constants;

/// Module defining custom error types used throughout the program.
/// Errors are returned via the Anchor framework when instructions fail.
pub mod error;

/// Module containing all instruction handlers for the program,
/// such as initializing a lottery, buying tickets, and choosing a winner.
pub mod instructions;

/// Module defining the on-chain state structures for the program,
/// including the `TokenLottery` account and any related PDAs.s
pub mod state;
