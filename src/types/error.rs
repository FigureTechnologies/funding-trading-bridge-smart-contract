use cosmwasm_std::StdError;
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("conversion failure: {message}")]
    ConversionError { message: String },

    #[error("instantiation error occurred: {message}")]
    InstantiationError { message: String },

    #[error("invalid account: {message}")]
    InvalidAccountError { message: String },

    #[error("invalid format: {message}")]
    InvalidFormatError { message: String },

    #[error("invalid funds: {message}")]
    InvalidFundsError { message: String },

    #[error("migration error occurred: {message}")]
    MigrationError { message: String },

    #[error("not authorized: {message}")]
    NotAuthorizedError { message: String },

    #[error("not found: {message}")]
    NotFoundError { message: String },

    #[error("{0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("{0}")]
    SemVerError(#[from] semver::Error),

    #[error("{0}")]
    Std(#[from] StdError),

    #[error("storage error occurred: {message}")]
    StorageError { message: String },

    #[error("validation failed: {message}")]
    ValidationError { message: String },
}
