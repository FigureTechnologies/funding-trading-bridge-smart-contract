use crate::execute::admin_update_admin::admin_update_admin;
use crate::execute::admin_update_deposit_required_attributes::admin_update_deposit_required_attributes;
use crate::execute::admin_update_withdraw_required_attributes::admin_update_withdraw_required_attributes;
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
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    msg.self_validate()?;
    instantiate_contract(deps, env, info, msg)
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
        ExecuteMsg::AdminUpdateAdmin { new_admin_address } => {
            admin_update_admin(deps, env, info, new_admin_address)
        }
        ExecuteMsg::AdminUpdateDepositRequiredAttributes { attributes } => {
            admin_update_deposit_required_attributes(deps, env, info, attributes)
        }
        ExecuteMsg::AdminUpdateWithdrawRequiredAttributes { attributes } => {
            admin_update_withdraw_required_attributes(deps, env, info, attributes)
        }
        ExecuteMsg::FundTrading { trade_amount } => {
            fund_trading(deps, env, info, trade_amount.u128())
        }
        ExecuteMsg::WithdrawTrading { trade_amount } => {
            withdraw_trading(deps, env, info, trade_amount.u128())
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    msg.self_validate()?;
    match msg {
        QueryMsg::QueryContractState {} => query_contract_state(deps),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    msg.self_validate()?;
    match msg {
        MigrateMsg::ContractUpgrade {} => migrate_contract(deps),
    }
}
