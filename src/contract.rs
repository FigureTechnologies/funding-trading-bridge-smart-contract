use crate::execute::fund_trading::fund_trading;
use crate::execute::withdraw_trading::withdraw_trading;
use crate::instantiate::instantiate_contract::instantiate_contract;
use crate::migrate::migrate_contract::migrate_contract;
use crate::query::query_contract_state::query_contract_state;
use crate::types::error::ContractError;
use crate::types::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::util::self_validating::SelfValidating;
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    msg.self_validate()?;
    instantiate_contract(deps, info, msg)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    msg.self_validate()?;
    match msg {
        ExecuteMsg::FundTrading {} => fund_trading(deps, env, info),
        ExecuteMsg::WithdrawTrading {} => withdraw_trading(deps, env, info),
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    msg.self_validate()?;
    match msg {
        QueryMsg::QueryContractState {} => query_contract_state(deps),
    }
}

pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    msg.self_validate()?;
    match msg {
        MigrateMsg::ContractUpgrade {} => migrate_contract(deps),
    }
}
