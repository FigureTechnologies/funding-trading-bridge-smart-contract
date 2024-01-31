use crate::execute::fund_trading::fund_trading;
use crate::instantiate::instantiate_contract::instantiate_contract;
use crate::migrate::migrate_contract::migrate_contract;
use crate::types::error::ContractError;
use crate::types::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
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
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::FundTrading { deposit_amount } => {
            fund_trading(deps, env, info, deposit_amount.u128())
        }
    }
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

pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {
        MigrateMsg::ContractUpgrade {} => migrate_contract(deps),
    }
}
