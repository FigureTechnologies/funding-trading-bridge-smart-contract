use crate::store::contract_state::{get_contract_state_v1, CONTRACT_TYPE};
use crate::types::error::ContractError;
use crate::util::conversion_utils::convert_denom;
use crate::util::provenance_utils::{
    check_account_has_all_attributes, check_account_has_enough_denom,
};
use crate::util::validation_utils::check_funds_are_empty;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use provwasm_std::types::cosmos::base::v1beta1::Coin;
use provwasm_std::types::provenance::marker::v1::{
    MsgMintRequest, MsgTransferRequest, MsgWithdrawRequest,
};
use result_extensions::ResultExtensions;

pub fn fund_trading(
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
        &contract_state.deposit_marker.name,
        trade_amount,
    )?;
    check_account_has_all_attributes(
        &deps,
        &info.sender,
        &contract_state.required_deposit_attributes,
    )?;
    let conversion = convert_denom(
        trade_amount,
        &contract_state.deposit_marker,
        &contract_state.trading_marker,
    )?;
    if conversion.target_amount == 0 {
        return ContractError::InvalidFundsError {
            message: format!(
                "sent [{}{}], but that is not enough to convert to at least one [{}]",
                trade_amount,
                &contract_state.deposit_marker.name,
                &contract_state.trading_marker.name,
            ),
        }
        .to_err();
    }
    // Transfer the necessary amount from the sender (total amount requested - remainder that cannot be converted)
    let transferred_amount = trade_amount - conversion.remainder;
    let transfer_msg = MsgTransferRequest {
        administrator: env.contract.address.to_string(),
        amount: Some(Coin {
            denom: contract_state.deposit_marker.name.to_owned(),
            amount: transferred_amount.to_string(),
        }),
        from_address: info.sender.to_string(),
        to_address: env.contract.address.to_string(),
    };
    // Mint the amount of coin to which the conversion equates
    let minted_coin = Coin {
        denom: contract_state.trading_marker.name.to_owned(),
        amount: conversion.target_amount.to_string(),
    };
    let mint_msg = MsgMintRequest {
        administrator: env.contract.address.to_string(),
        amount: Some(minted_coin.to_owned()),
    };
    // Withdraw the newly-minted coin to the sender, effectively making the trade
    let withdraw_msg = MsgWithdrawRequest {
        denom: contract_state.trading_marker.name.to_owned(),
        administrator: env.contract.address.to_string(),
        to_address: info.sender.to_string(),
        amount: vec![minted_coin.to_owned()],
    };
    let mut response = Response::new()
        .add_message(transfer_msg)
        .add_message(mint_msg)
        .add_message(withdraw_msg)
        .add_attribute("action", "fund_trading")
        .add_attribute("contract_address", env.contract.address.to_string())
        .add_attribute("contract_type", CONTRACT_TYPE)
        .add_attribute("contract_name", &contract_state.contract_name)
        .add_attribute("deposit_input_denom", &contract_state.deposit_marker.name)
        .add_attribute("deposit_requested_amount", trade_amount.to_string())
        .add_attribute("deposit_actual_amount", transferred_amount.to_string())
        .add_attribute("received_denom", minted_coin.denom)
        .add_attribute("received_amount", minted_coin.amount);
    response.to_ok()
}
