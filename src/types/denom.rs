use crate::types::error::ContractError;
use crate::util::self_validating::SelfValidating;
use cosmwasm_std::Uint64;
use result_extensions::ResultExtensions;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Denom {
    pub name: String,
    pub precision: Uint64,
}
impl SelfValidating for Denom {
    fn self_validate(&self) -> Result<(), ContractError> {
        let error_msg = if self.name.is_empty() {
            "denom name cannot be empty"
        } else if self.precision.u64() < 0 {
            "denom precision cannot be less than zero"
        } else {
            return ().to_ok();
        }
        .to_string();
        ContractError::ValidationError { message: error_msg }.to_err()
    }
}
impl Denom {
    pub fn new<S: Into<String>>(name: S, precision: u64) -> Self {
        Self {
            name: name.into(),
            precision: Uint64::new(precision),
        }
    }
}
