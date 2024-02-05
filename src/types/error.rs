use cosmwasm_std::StdError;
use std::num::ParseIntError;
use thiserror::Error;

/// The base error enum that is used to wrap any errors that occur throughout contract execution.
#[derive(Error, Debug)]
pub enum ContractError {
    /// An error that occurs when a conversion between two denominations fails.
    #[error("conversion failure: {message}")]
    ConversionError {
        /// A free-form message describing the nature of the error.
        message: String,
    },

    /// An error that occurs when a blockchain account contains invalid information.
    #[error("invalid account: {message}")]
    InvalidAccountError {
        /// A free-form message describing the nature of the error.
        message: String,
    },

    /// An error that occurs when an invalid text format is detected.
    #[error("invalid format: {message}")]
    InvalidFormatError {
        /// A free-form message describing the nature of the error.
        message: String,
    },

    /// An error that occurs when invalid funds are sent to the contract.
    #[error("invalid funds: {message}")]
    InvalidFundsError {
        /// A free-form message describing the nature of the error.
        message: String,
    },

    /// An error that occurs when a migration fails.
    #[error("migration error occurred: {message}")]
    MigrationError {
        /// A free-form message describing the nature of the error.
        message: String,
    },

    /// An error that occurs when the executing sender is not authorized to take an action.
    #[error("not authorized: {message}")]
    NotAuthorizedError {
        /// A free-form message describing the nature of the error.
        message: String,
    },

    /// An error that occurs when a target resource cannot be found.
    #[error("not found: {message}")]
    NotFoundError {
        /// A free-form message describing the nature of the error.
        message: String,
    },

    /// A wrapper for a core library integer parsing error.
    #[error("{0}")]
    ParseIntError(#[from] ParseIntError),

    /// A wrapper for a semver library error.
    #[error("{0}")]
    SemVerError(#[from] semver::Error),

    /// A wrapper for a a core library std error.
    #[error("{0}")]
    Std(#[from] StdError),

    /// An error that occurs when smart contract storage fails.
    #[error("storage error occurred: {message}")]
    StorageError {
        /// A free-form message describing the nature of the error.
        message: String,
    },

    /// An error that occurs when type validation fails.
    #[error("validation failed: {message}")]
    ValidationError {
        /// A free-form message describing the nature of the error.
        message: String,
    },
}
