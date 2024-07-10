use crate::types::denom::Denom;
use crate::types::error::ContractError;
use crate::util::self_validating::SelfValidating;
use crate::util::validation_utils::validate_attribute_name;
use cosmwasm_std::Uint128;
use result_extensions::ResultExtensions;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The msg that is sent to the chain in order to instantiate a new instance of this contract's
/// stored code.  Used in the functionality described in [instantiate_contract](crate::instantiate::instantiate_contract::instantiate_contract).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    /// A free-form name defining this particular contract instance.  Used for identification on
    /// query purposes only.
    pub contract_name: String,
    /// Defines the marker denom that is deposited to this contract in exchange for [trading_marker](crate::store::contract_state::ContractStateV1#trading_marker)
    /// denom.
    pub deposit_marker: Denom,
    /// Defines the marker denom that is sent to accounts from this contract in exchange for
    /// [deposit_marker](crate::store::contract_state::ContractStateV1#deposit_marker).
    pub trading_marker: Denom,
    /// Defines any blockchain attributes required on accounts in order to execute the [fund_trading](crate::execute::fund_trading::fund_trading)
    /// execution route.
    pub required_deposit_attributes: Vec<String>,
    /// Defines any blockchain attributes required on accounts in order to execute the
    /// [withdraw_trading](crate::execute::withdraw_trading::withdraw_trading) execution route.
    pub required_withdraw_attributes: Vec<String>,
    /// If provided, this value must be a valid provenance name module name that can be bound to an
    /// unrestricted parent name.  This will cause the contract to bind the provided name to itself.
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
            .any(|attr| validate_attribute_name(attr).is_err())
        {
            return ContractError::ValidationError {
                message: "all required deposit attributes must be valid".to_string(),
            }
            .to_err();
        }
        if self
            .required_withdraw_attributes
            .iter()
            .any(|attr| validate_attribute_name(attr).is_err())
        {
            return ContractError::ValidationError {
                message: "all required withdraw attributes must be valid".to_string(),
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

/// All defined paylods to be used when executing routes on this contract instance.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// A route that swaps the current value in the [contract state](crate::store::contract_state::ContractStateV1)
    /// for the admin to the provided value.
    AdminUpdateAdmin {
        /// A bech32 address to use as the new administrator of the contract.
        new_admin_address: String,
    },
    /// A route that sets a new collection of attribute names required when an account deposits their
    /// deposit denom into the contract via the [fund_trading](crate::execute::fund_trading::fund_trading)
    /// execution route.
    AdminUpdateDepositRequiredAttributes {
        /// The new attributes that will be set in the contract state's [required_deposit_attributes](crate::store::contract_state::ContractStateV1#required_deposit_attributes)
        /// property upon successful execution.
        attributes: Vec<String>,
    },
    /// A route that sets a new collection of attribute names required when an account withdraws
    /// their deposit denom from the contract via the [withdraw_trading](crate::execute::withdraw_trading::withdraw_trading)
    /// execution route.
    AdminUpdateWithdrawRequiredAttributes {
        /// The new attributes that will be set in the contract state's [required_withdraw_attributes](crate::store::contract_state::ContractStateV1#required_withdraw_attributes)
        /// property upon successful execution.
        attributes: Vec<String>,
    },
    /// A route that will attempt to pull the trade amount of the deposit marker's denom from the
    /// sender's account with a marker transfer, discern how much of the trading denom to which the
    /// submitted amount is equivalent, and then mint and withdraw the equivalent amount into the
    /// sender's account.
    FundTrading {
        /// The amount of the deposit marker to pull from the sender's account in exchange for
        /// trading denom.
        trade_amount: Uint128,
    },
    /// A route that will attempt to pull the trade amount of the trading marker's denom from the
    /// sender's account with a marker transfer, discern how much of the deposit denom to which the
    /// submitted amount is equivalent, transfer that amount to the sender, and then burn the
    /// exchanged trading marker denom.
    WithdrawTrading {
        /// The amount of the trading marker to pull from the sender's account in exchange for
        /// deposit denom.
        trade_amount: Uint128,
    },
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
                if attributes
                    .iter()
                    .any(|attr| validate_attribute_name(attr).is_err())
                {
                    return ContractError::ValidationError {
                        message: "all specified attributes must be valid".to_string(),
                    }
                    .to_err();
                }
            }
            ExecuteMsg::AdminUpdateWithdrawRequiredAttributes { attributes } => {
                if attributes
                    .iter()
                    .any(|attr| validate_attribute_name(attr).is_err())
                {
                    return ContractError::ValidationError {
                        message: "all specified attributes must be valid".to_string(),
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

/// All defined payloads to be used when querying routes on this contract instance.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// A route that returns the current [contract state](crate::store::contract_state::ContractStateV1)
    /// value stored in state.  Invokes the functionality defined in [query_contract_state](crate::query::query_contract_state).
    QueryContractState {},
}
impl SelfValidating for QueryMsg {
    fn self_validate(&self) -> Result<(), ContractError> {
        match self {
            QueryMsg::QueryContractState {} => ().to_ok(),
        }
    }
}

/// All defined payloads to be used when migrating to a new instance of this contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {
    /// The standard migration route that modifies the [contract state](crate::store::contract_state::ContractStateV1)
    /// to include the new values defined in a target code instance.  Invokes the functionality
    /// defined in [migrate_contract](crate::migrate::migrate_contract::migrate_contract).
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
                required_deposit_attributes: vec!["a.aa.b".to_string()],
                ..InstantiateMsg::default()
            }
            .self_validate()
            .expect_err("expected invalid required deposit attributes to fail"),
            "all required deposit attributes must be valid",
        );
        assert_validation_err(
            &InstantiateMsg {
                required_withdraw_attributes: vec!["normal.stillnormal.andthen😏".to_string()],
                ..InstantiateMsg::default()
            }
            .self_validate()
            .expect_err("expected invalid required withdraw attributes to fail"),
            "all required withdraw attributes must be valid",
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
                attributes: vec![
                    "verylongstringintheattributeshouldberejected.thiswouldbeokthough".to_string(),
                ],
            }
            .self_validate()
            .expect_err("expected invalid attributes to fail"),
            "all specified attributes must be valid",
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
                attributes: vec!["not a.validattribute".to_string()],
            }
            .self_validate()
            .expect_err("expected invalid attributes to fail"),
            "all specified attributes must be valid",
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
