use crate::types::denom::Denom;
use crate::types::error::ContractError;
use crate::util::self_validating::SelfValidating;
use cosmwasm_std::Uint128;
use result_extensions::ResultExtensions;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    pub contract_name: String,
    pub deposit_marker: Denom,
    pub trading_marker: Denom,
    pub required_deposit_attributes: Vec<String>,
    pub required_withdraw_attributes: Vec<String>,
}
impl SelfValidating for InstantiateMsg {
    fn self_validate(&self) -> Result<(), ContractError> {
        if self.contract_name.is_empty() {
            return ContractError::ValidationError {
                message: "contract name cannot be empty".to_string(),
            }
            .to_err();
        }
        self.deposit_marker
            .self_validate()
            .map_err(|e| ContractError::ValidationError {
                message: format!("deposito marker: {e:?}"),
            })?;
        self.trading_marker
            .self_validate()
            .map_err(|e| ContractError::ValidationError {
                message: format!("trading marker: {e:?}"),
            })?;
        if self
            .required_deposit_attributes
            .iter()
            .any(|attr| attr.is_empty())
        {
            return ContractError::ValidationError {
                message: "all required deposit attributes must be non-empty values".to_string(),
            }
            .to_err();
        }
        if self
            .required_withdraw_attributes
            .iter()
            .any(|attr| attr.is_empty())
        {
            return ContractError::ValidationError {
                message: "all required withdraw attributes must be non-empty values".to_string(),
            }
            .to_err();
        }
        ().to_ok()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum ExecuteMsg {
    FundTrading { deposit_amount: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum QueryMsg {}

pub enum MigrateMsg {
    ContractUpgrade {},
}
