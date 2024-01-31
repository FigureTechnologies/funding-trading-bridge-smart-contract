use crate::store::contract_state::{set_contract_state_v1, ContractStateV1};
use crate::types::error::ContractError;
use crate::types::msg::InstantiateMsg;
use crate::util::provenance_utils::msg_bind_name;
use crate::util::validation_utils::check_funds_are_empty;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use result_extensions::ResultExtensions;

// TODO: Validate that both deposit / trading denoms are tied to restricted markers (and that they exist at all lol)
pub fn instantiate_contract(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    check_funds_are_empty(&info)?;
    let contract_state = ContractStateV1::new(
        info.sender,
        &msg.contract_name,
        &msg.deposit_marker,
        &msg.trading_marker,
        &msg.required_deposit_attributes,
        &msg.required_deposit_attributes,
    );
    set_contract_state_v1(deps.storage, &contract_state)?;
    let mut response = Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", &msg.contract_name)
        .add_attribute("deposit_marker_name", &msg.deposit_marker.name)
        .add_attribute("trading_marker_name", &msg.trading_marker.name);
    if let Some(name) = msg.name_to_bind {
        response = response
            .add_message(msg_bind_name(&name, &env.contract.address, true)?)
            .add_attribute("contract_bound_with_name", name)
    }
    response.to_ok()
}
