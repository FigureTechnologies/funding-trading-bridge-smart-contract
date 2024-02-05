use crate::types::error::ContractError;

/// A trait that defines a struct that validates its own fields.
pub trait SelfValidating {
    /// Validates all fields by self-reference, where necessary.
    fn self_validate(&self) -> Result<(), ContractError>;
}
