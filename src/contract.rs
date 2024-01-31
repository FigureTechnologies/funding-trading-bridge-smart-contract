use crate::instantiate::instantiate_contract::instantiate_contract;
use crate::types::error::ContractError;
use crate::types::msg::InstantiateMsg;
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use result_extensions::ResultExtensions;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    instantiate_contract(deps, info, msg)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: String, // Todo: Message
) -> Result<Response, ContractError> {
    ContractError::UnimplementedError {
        message: "one cannot simply execute into Mordor".to_string(),
    }
    .to_err()
}

pub fn query(
    deps: Deps,
    env: Env,
    msg: String, // Todo: Message
) -> Result<Binary, ContractError> {
    ContractError::UnimplementedError {
        message: "all your base are belong to query".to_string(),
    }
    .to_err()
}

pub fn migrate(
    deps: DepsMut,
    env: Env,
    msg: String, // Todo: Message
) -> Result<Response, ContractError> {
    ContractError::UnimplementedError {
        message: "you should be miGRATEFUL that you can even instantiate this contract".to_string(),
    }
    .to_err()
}
