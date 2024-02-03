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
                message: format!("deposit marker: {e:?}"),
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
        if let Some(name) = &self.name_to_bind {
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
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AdminUpdateAdmin { new_admin_address: String },
    AdminUpdateDepositRequiredAttributes { attributes: Vec<String> },
    AdminUpdateWithdrawRequiredAttributes { attributes: Vec<String> },
    FundTrading { trade_amount: Uint128 },
    WithdrawTrading { trade_amount: Uint128 },
}
impl SelfValidating for ExecuteMsg {
    fn self_validate(&self) -> Result<(), ContractError> {
        match self {
            ExecuteMsg::AdminUpdateAdmin { new_admin_address } => {
                if new_admin_address.is_empty() {
                    return ContractError::ValidationError {
                        message: "new_admin_address param must be supplied".to_string(),
                    }
                    .to_err();
                }
            }
            ExecuteMsg::AdminUpdateDepositRequiredAttributes { attributes } => {
                if attributes.iter().any(|attr| attr.is_empty()) {
                    return ContractError::ValidationError {
                        message: "all specified attributes must be non-empty values".to_string(),
                    }
                    .to_err();
                }
            }
            ExecuteMsg::AdminUpdateWithdrawRequiredAttributes { attributes } => {
                if attributes.iter().any(|attr| attr.is_empty()) {
                    return ContractError::ValidationError {
                        message: "all specified attributes must be non-empty values".to_string(),
                    }
                    .to_err();
                }
            }
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
#[serde(rename_all = "snake_case")]
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
#[serde(rename_all = "snake_case")]
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

#[cfg(test)]
mod tests {
    use crate::types::denom::Denom;
    use crate::types::error::ContractError;
    use crate::types::msg::{ExecuteMsg, InstantiateMsg};
    use crate::util::self_validating::SelfValidating;
    use cosmwasm_std::{Uint128, Uint64};

    #[test]
    fn instantiate_msg_self_validation_should_function_properly() {
        assert_validation_err(
            &InstantiateMsg {
                contract_name: "".to_string(),
                ..InstantiateMsg::default()
            }
            .self_validate()
            .expect_err("expected missing contract name to fail"),
            "contract name cannot be empty",
        );
        assert_validation_err(
            &InstantiateMsg {
                deposit_marker: Denom {
                    name: "".to_string(),
                    precision: Uint64::new(10),
                },
                ..InstantiateMsg::default()
            }
            .self_validate()
            .expect_err("expected invalid deposit marker to fail"),
            "deposit marker: name cannot be empty",
        );
        assert_validation_err(
            &InstantiateMsg {
                trading_marker: Denom {
                    name: "".to_string(),
                    precision: Uint64::new(10),
                },
                ..InstantiateMsg::default()
            }
            .self_validate()
            .expect_err("expected invalid trading marker to fail"),
            "trading marker: name cannot be empty",
        );
        assert_validation_err(
            &InstantiateMsg {
                required_deposit_attributes: vec!["".to_string()],
                ..InstantiateMsg::default()
            }
            .self_validate()
            .expect_err("expected invalid required deposit attributes to fail"),
            "all required deposit attributes must be non-empty values",
        );
        assert_validation_err(
            &InstantiateMsg {
                required_withdraw_attributes: vec!["".to_string()],
                ..InstantiateMsg::default()
            }
            .self_validate()
            .expect_err("expected invalid required withdraw attributes to fail"),
            "all required withdraw attributes must be non-empty values",
        );
        assert_validation_err(
            &InstantiateMsg {
                name_to_bind: Some("".to_string()),
                ..InstantiateMsg::default()
            }
            .self_validate()
            .expect_err("expected invalid name to bind to fail"),
            "contract name cannot be specified as empty string",
        );
        InstantiateMsg::default()
            .self_validate()
            .expect("proper instantiate message values should pass validation");
    }

    #[test]
    fn admin_update_admin_execute_message_validation_should_function_properly() {
        assert_validation_err(
            &ExecuteMsg::AdminUpdateAdmin {
                new_admin_address: "".to_string(),
            }
            .self_validate()
            .expect_err("expected invalid new_admin_address to fail"),
            "new_admin_address param must be supplied",
        );
        ExecuteMsg::AdminUpdateAdmin {
            new_admin_address: "some-addr".to_string(),
        }
        .self_validate()
        .expect("non-empty input for new admin address should succeed");
    }

    #[test]
    fn admin_update_deposit_required_attributes_execute_message_validation_should_function_properly(
    ) {
        assert_validation_err(
            &ExecuteMsg::AdminUpdateDepositRequiredAttributes {
                attributes: vec!["".to_string()],
            }
            .self_validate()
            .expect_err("expected invalid attributes to fail"),
            "all specified attributes must be non-empty values",
        );
        ExecuteMsg::AdminUpdateDepositRequiredAttributes { attributes: vec![] }
            .self_validate()
            .expect("empty attributes should succeed");
        ExecuteMsg::AdminUpdateDepositRequiredAttributes {
            attributes: vec!["some-attribute".to_string()],
        }
        .self_validate()
        .expect("specified attributes should succeed");
    }

    #[test]
    fn admin_update_withdraw_required_attributes_execute_message_validation_should_function_properly(
    ) {
        assert_validation_err(
            &ExecuteMsg::AdminUpdateWithdrawRequiredAttributes {
                attributes: vec!["".to_string()],
            }
            .self_validate()
            .expect_err("expected invalid attributes to fail"),
            "all specified attributes must be non-empty values",
        );
        ExecuteMsg::AdminUpdateWithdrawRequiredAttributes { attributes: vec![] }
            .self_validate()
            .expect("empty attributes should succeed");
        ExecuteMsg::AdminUpdateWithdrawRequiredAttributes {
            attributes: vec!["some-attribute".to_string()],
        }
        .self_validate()
        .expect("specified attributes should succeed");
    }

    #[test]
    fn funding_trading_execute_message_validation_should_function_properly() {
        assert_validation_err(
            &ExecuteMsg::FundTrading {
                trade_amount: Uint128::new(0),
            }
            .self_validate()
            .expect_err("expected invalid trade amount to fail"),
            "trade amount must be greater than zero",
        );
        ExecuteMsg::FundTrading {
            trade_amount: Uint128::new(1),
        }
        .self_validate()
        .expect("a valid funding trading msg should pass validation");
    }

    #[test]
    fn withdraw_trading_execute_message_validation_should_function_properly() {
        assert_validation_err(
            &ExecuteMsg::WithdrawTrading {
                trade_amount: Uint128::new(0),
            }
            .self_validate()
            .expect_err("expected invalid trade amount to fail"),
            "trade amount must be greater than zero",
        );
        ExecuteMsg::WithdrawTrading {
            trade_amount: Uint128::new(1),
        }
        .self_validate()
        .expect("a valid withdraw trading msg should pass validation");
    }

    fn assert_validation_err<S: Into<String>>(error: &ContractError, expected_message: S) {
        let _message = expected_message.into();
        assert!(
            matches!(error, ContractError::ValidationError { message: _message },),
            "expected validation error with proper message {_message} but got: {error:?}",
        );
    }
}
