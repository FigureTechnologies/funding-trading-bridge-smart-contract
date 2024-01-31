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
        if self.name.is_empty() {
            return ContractError::ValidationError {
                message: "name cannot be empty".to_string(),
            }
            .to_err();
        }
        ().to_ok()
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct DenomConversion {
    pub source_amount: u128,
    pub target_amount: u128,
    pub remainder: u128,
}
