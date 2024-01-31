use crate::store::contract_state::{get_contract_state_v1, CONTRACT_TYPE};
use crate::types::error::ContractError;
use crate::util::conversion_utils::convert_denom;
use crate::util::provenance_utils::{
    check_account_has_all_attributes, check_account_has_enough_denom, get_marker_address_for_denom,
};
use crate::util::validation_utils::check_funds_are_empty;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use provwasm_std::types::cosmos::base::v1beta1::Coin;
use provwasm_std::types::provenance::marker::v1::{MsgBurnRequest, MsgTransferRequest};
use result_extensions::ResultExtensions;

pub fn withdraw_trading(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    trade_amount: u128,
) -> Result<Response, ContractError> {
    check_funds_are_empty(&info)?;
    let contract_state = get_contract_state_v1(deps.storage)?;
    check_account_has_enough_denom(
        &deps,
        info.sender.as_str(),
        &contract_state.trading_marker.name,
        trade_amount,
    )?;
    check_account_has_all_attributes(
        &deps,
        &info.sender,
        &contract_state.required_withdraw_attributes,
    )?;
    let conversion = convert_denom(
        trade_amount,
        &contract_state.trading_marker,
        &contract_state.deposit_marker,
    )?;
    if conversion.target_amount == 0 {
        return ContractError::InvalidFundsError {
            message: format!(
                "sent [{}{}], but that is not enough to convert to at least one [{}]",
                trade_amount,
                &contract_state.trading_marker.name,
                &contract_state.deposit_marker.name,
            ),
        }
        .to_err();
    }
    let collected_amount = trade_amount - conversion.remainder;
    // Collect the amount to be traded to the contract from the sender and give it directly to the
    // marker in order to stage it for burning
    let collect_funds_msg = MsgTransferRequest {
        administrator: env.contract.address.to_string(),
        amount: Some(Coin {
            denom: contract_state.trading_marker.name.to_owned(),
            amount: collected_amount.to_string(),
        }),
        from_address: info.sender.to_string(),
        to_address: get_marker_address_for_denom(&deps, &contract_state.trading_marker.name)?,
    };
    // Release the total converted amount of funds back to the user
    let release_funds_msg = MsgTransferRequest {
        administrator: env.contract.address.to_string(),
        amount: Some(Coin {
            denom: contract_state.deposit_marker.name.to_owned(),
            amount: conversion.target_amount.to_string(),
        }),
        from_address: env.contract.address.to_string(),
        to_address: info.sender.to_string(),
    };
    // Burn all coins that were received except those that could not be converted, these will be
    // refunded
    let burn_msg = MsgBurnRequest {
        administrator: env.contract.address.to_string(),
        amount: Some(Coin {
            amount: collected_amount.to_string(),
            denom: contract_state.trading_marker.name.to_owned(),
        }),
    };
    Response::new()
        .add_message(collect_funds_msg)
        .add_message(release_funds_msg)
        .add_message(burn_msg)
        .add_attribute("action", "withdraw_trading")
        .add_attribute("contract_address", env.contract.address.to_string())
        .add_attribute("contract_type", CONTRACT_TYPE)
        .add_attribute("contract_name", &contract_state.contract_name)
        .add_attribute("withdraw_input_denom", &contract_state.trading_marker.name)
        .add_attribute("withdraw_input_amount", trade_amount.to_string())
        .add_attribute("withdraw_actual_amount", collected_amount.to_string())
        .add_attribute("received_denom", &contract_state.deposit_marker.name)
        .add_attribute("received_amount", conversion.target_amount.to_string())
        .to_ok()
}
