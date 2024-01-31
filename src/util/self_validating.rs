use crate::types::error::ContractError;

pub trait SelfValidating {
    fn self_validate(&self) -> Result<(), ContractError>;
}
