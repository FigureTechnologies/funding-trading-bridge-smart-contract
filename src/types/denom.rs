use crate::types::error::ContractError;
use crate::util::self_validating::SelfValidating;
use cosmwasm_std::Uint64;
use result_extensions::ResultExtensions;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Defines a blockchain denom associated with a marker in reference to the contract's usages.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Denom {
    /// The name of the marker on-chain that manages this denom.
    pub name: String,
    /// The amount of decimal places represented in coin by this denom.
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
    /// Constructs a new instance of this struct.
    ///
    /// # Parameters
    /// * `name` The name of the marker on-chain that manages this denom.
    /// * `precision` The amount decimal places represented in coin by this denom.
    pub fn new<S: Into<String>>(name: S, precision: u64) -> Self {
        Self {
            name: name.into(),
            precision: Uint64::new(precision),
        }
    }
}

/// Defines a conversion between one denom and another.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct DenomConversion {
    /// The amount of the first denom used in the conversion.
    pub source_amount: u128,
    /// The amount of the second denom to which the first denom is equivalent after conversion.
    pub target_amount: u128,
    /// Any amount of the [source amount](DenomConversion#source_amount) that cannot be converted to
    /// the second denom due to values that do not fit into the second denom's precision.
    pub remainder: u128,
}
