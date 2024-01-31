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
    pub name_to_bind: Option<String>,
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
        if let Some(name) = self.name_to_bind.to_owned() {
            if name.is_empty() {
                return ContractError::ValidationError {
                    message: "contract name cannot be specified as empty string".to_string(),
                }
                .to_err();
            }
        }
        ().to_ok()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum ExecuteMsg {
    FundTrading { trade_amount: Uint128 },
    WithdrawTrading { trade_amount: Uint128 },
}
impl SelfValidating for ExecuteMsg {
    fn self_validate(&self) -> Result<(), ContractError> {
        match self {
            ExecuteMsg::FundTrading { trade_amount } => {
                if trade_amount.u128() == 0 {
                    return ContractError::ValidationError {
                        message: "trade amount must be greater than zero".to_string(),
                    }
                    .to_err();
                }
            }
            ExecuteMsg::WithdrawTrading { trade_amount } => {
                if trade_amount.u128() == 0 {
                    return ContractError::ValidationError {
                        message: "trade amount must be greater than zero".to_string(),
                    }
                    .to_err();
                }
            }
        }
        ().to_ok()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum QueryMsg {
    QueryContractState {},
}
impl SelfValidating for QueryMsg {
    fn self_validate(&self) -> Result<(), ContractError> {
        match self {
            QueryMsg::QueryContractState {} => ().to_ok(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum MigrateMsg {
    ContractUpgrade {},
}
impl SelfValidating for MigrateMsg {
    fn self_validate(&self) -> Result<(), ContractError> {
        match self {
            MigrateMsg::ContractUpgrade {} => ().to_ok(),
        }
    }
}
