use crate::types::error::ContractError;
use cosmwasm_std::{Coin, MessageInfo};
use result_extensions::ResultExtensions;

pub fn check_funds_are_empty(info: &MessageInfo) -> Result<(), ContractError> {
    if !info.funds.is_empty() {
        ContractError::InvalidFundsError {
            message: "funds provided but empty funds required".to_string(),
        }
        .to_err()
    } else {
        ().to_ok()
    }
}

pub fn get_single_coin_input<S: Into<String>>(
    info: &MessageInfo,
    required_denom: S,
) -> Result<Coin, ContractError> {
    let denom = required_denom.into();
    if info.funds.len() != 1 {
        return ContractError::InvalidFundsError {
            message: format!("expected only a single coin of type [{denom}] to be provided"),
        }
        .to_err();
    }
    let coin = info.funds.first().unwrap();
    if coin.denom != denom {
        return ContractError::InvalidFundsError {
            message: format!(
                "expected provided coin to be of type [{denom}] but was [{}]",
                &coin.denom,
            ),
        }
        .to_err();
    }
    coin.to_owned().to_ok()
}
