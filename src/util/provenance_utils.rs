use crate::types::error::ContractError;
use cosmwasm_std::{DepsMut, Uint128};
use provwasm_std::types::cosmos::bank::v1beta1::BankQuerier;
use provwasm_std::types::cosmos::base::query::v1beta1::PageRequest;
use provwasm_std::types::provenance::attribute::v1::AttributeQuerier;
use result_extensions::ResultExtensions;

pub fn check_account_has_all_attributes<S: Into<String>>(
    deps: &DepsMut,
    account: S,
    attributes: &[String],
) -> Result<(), ContractError> {
    if attributes.is_empty() {
        return ().to_ok();
    }
    let querier = AttributeQuerier::new(&deps.querier);
    let account_addr = account.into();
    let mut latest_response = querier.attributes(account_addr.to_owned(), None)?;
    let mut remaining_attributes = attributes.to_vec();
    while !remaining_attributes.is_empty() {
        for attr in latest_response.attributes.iter() {
            if remaining_attributes.contains(&attr.name) {
                remaining_attributes.retain(|name| name != &attr.name);
            }
        }
        if !remaining_attributes.is_empty()
            && latest_response.pagination.is_some()
            && !latest_response
                .pagination
                .clone()
                .unwrap()
                .next_key
                .is_empty()
        {
            latest_response = querier.attributes(
                account_addr.to_owned(),
                Some(PageRequest {
                    key: latest_response.pagination.unwrap().next_key.to_owned(),
                    offset: 0,
                    limit: 25,
                    count_total: false,
                    reverse: false,
                }),
            )?;
        } else {
            return ContractError::InvalidAccountError {
                message: "account does not have all required attributes".to_string(),
            }
            .to_err();
        }
    }
    ().to_ok()
}

pub fn check_account_has_enough_denom<S1: Into<String>, S2: Into<String>>(
    deps: &DepsMut,
    account: S1,
    denom: S2,
    required_amount: u128,
) -> Result<(), ContractError> {
    let querier = BankQuerier::new(&deps.querier);
    let account_address = account.into();
    let target_denom = denom.into();
    let balance_response = querier.balance(account_address.to_owned(), target_denom.to_owned())?;
    if let Some(coin) = balance_response.balance {
        let numeric_balance = coin.amount.parse::<u128>()?;
        if numeric_balance < required_amount {
            ContractError::InvalidAccountError {
                message: format!(
                    "required [{required_amount}], but account only holds [{numeric_balance}]"
                ),
            }
            .to_err()
        } else {
            ().to_ok()
        }
    } else {
        ContractError::InvalidFundsError {
            message: format!("account [{account_address}] has no [{target_denom}] balance"),
        }
        .to_err()
    }
}
