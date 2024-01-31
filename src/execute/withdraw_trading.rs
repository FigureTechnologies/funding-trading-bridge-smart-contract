use crate::store::contract_state::{get_contract_state_v1, CONTRACT_TYPE};
use crate::types::error::ContractError;
use crate::util::conversion_utils::convert_denom;
use crate::util::provenance_utils::check_account_has_all_attributes;
use crate::util::validation_utils::get_single_coin_input;
use cosmwasm_std::{BankMsg, DepsMut, Env, MessageInfo, Response, Uint128};
use provwasm_std::types::cosmos::base::v1beta1::Coin;
use provwasm_std::types::provenance::marker::v1::MsgBurnRequest;
use result_extensions::ResultExtensions;

pub fn withdraw_trading(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let contract_state = get_contract_state_v1(deps.storage)?;
    let input_coin = get_single_coin_input(&info, &contract_state.trading_marker.name)?;
    check_account_has_all_attributes(
        &deps,
        &info.sender,
        &contract_state.required_withdraw_attributes,
    )?;
    let conversion = convert_denom(
        input_coin.amount.u128(),
        &contract_state.trading_marker,
        &contract_state.deposit_marker,
    )?;
    if conversion.target_amount == 0 {
        return ContractError::InvalidFundsError {
            message: format!(
                "sent [{}{}], but that is not enough to convert to at least one [{}]",
                input_coin.amount.u128(),
                &contract_state.trading_marker.name,
                &contract_state.deposit_marker.name,
            ),
        }
        .to_err();
    }
    // Release the total converted amount of funds back to the user
    let release_funds_msg = BankMsg::Send {
        amount: vec![cosmwasm_std::Coin {
            amount: Uint128::new(conversion.target_amount),
            denom: contract_state.deposit_marker.name.to_owned(),
        }],
        to_address: info.sender.to_string(),
    };
    // Burn all coins that were received except those that could not be converted, these will be
    // refunded
    let burn_amount = input_coin.amount.u128() - conversion.remainder;
    let burn_msg = MsgBurnRequest {
        administrator: env.contract.address.to_string(),
        amount: Some(Coin {
            amount: burn_amount.to_string(),
            denom: contract_state.trading_marker.name.to_owned(),
        }),
    };
    // Refund any amount of input coins that could not be successfully converted
    let refund_msg = if conversion.remainder > 0 {
        Some(BankMsg::Send {
            amount: vec![cosmwasm_std::Coin {
                amount: Uint128::new(conversion.remainder),
                denom: contract_state.trading_marker.name.to_owned(),
            }],
            to_address: info.sender.to_string(),
        })
    } else {
        None
    };
    let mut response = Response::new()
        .add_message(release_funds_msg)
        .add_message(burn_msg)
        .add_attribute("action", "withdraw_trading")
        .add_attribute("contract_address", env.contract.address.to_string())
        .add_attribute("contract_type", CONTRACT_TYPE)
        .add_attribute("contract_name", &contract_state.contract_name)
        .add_attribute("withdraw_input_denom", &contract_state.trading_marker.name)
        .add_attribute(
            "withdraw_input_amount",
            input_coin.amount.u128().to_string(),
        )
        .add_attribute("received_denom", &contract_state.deposit_marker.name)
        .add_attribute("received_amount", conversion.target_amount.to_string());
    if let Some(refund) = refund_msg {
        response = response
            .add_message(refund)
            .add_attribute("refund_denom", &contract_state.trading_marker.name)
            .add_attribute("refund_amount", conversion.remainder.to_string())
    }
    response.to_ok()
}
