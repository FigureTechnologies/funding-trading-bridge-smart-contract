use crate::store::contract_state::{set_contract_state_v1, ContractStateV1};
use crate::types::error::ContractError;
use crate::types::msg::InstantiateMsg;
use crate::util::self_validating::SelfValidating;
use crate::util::validation_utils::check_funds_are_empty;
use cosmwasm_std::{DepsMut, MessageInfo, Response};
use result_extensions::ResultExtensions;

pub fn instantiate_contract(
    deps: DepsMut,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    check_funds_are_empty(&info)?;
    msg.self_validate()?;
    let contract_state = ContractStateV1::new(
        info.sender,
        &msg.contract_name,
        &msg.deposit_marker,
        &msg.trading_marker,
    );
    set_contract_state_v1(deps.storage, &contract_state)?;
    Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", &msg.contract_name)
        .add_attribute("deposit_marker_name", &msg.deposit_marker.name)
        .add_attribute("trading_marker_name", &msg.trading_marker.name)
        .to_ok()
}
