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

/// The entry point used when an account instantiates a stored code wasm payload of this contract on
/// the Provenance Blockchain.
///
/// # Parameters
///
/// * `deps` A dependencies object provided by the cosmwasm framework.  Allows access to useful
/// resources like contract internal storage and a querier to retrieve blockchain objects.
/// * `env` An environment object provided by the cosmwasm framework.  Describes the contract's
/// details, as well as blockchain information at the time of the transaction.
/// * `info` A message information object provided by the cosmwasm framework.  Describes the sender
/// of the instantiation message, as well as the funds provided as an amount during the transaction.
/// * `msg` A custom instantiation message defined by this contract for creating the initial
/// configuration used by the contract.
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

/// The entry point used when an account initiates an execution process defined in the contract.
/// This defines the primary purposes of the contract.
///
/// # Parameters
///
/// * `deps` A dependencies object provided by the cosmwasm framework.  Allows access to useful
/// resources like contract internal storage and a querier to retrieve blockchain objects.
/// * `env` An environment object provided by the cosmwasm framework.  Describes the contract's
/// details, as well as blockchain information at the time of the transaction.
/// * `info` A message information object provided by the cosmwasm framework.  Describes the sender
/// of the instantiation message, as well as the funds provided as an amount during the transaction.
/// * `msg` A custom execution message enum defined by this contract to allow multiple different
/// processes to be defined for the singular execution route entry point allowed by the
/// cosmwasm framework.
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

/// The entry point used when an account invokes the contract to retrieve information.  Allows
/// access to the internal storage information in an immutable manner.
///
/// # Parameters
///
/// * `deps` A dependencies object provided by the cosmwasm framework.  Allows access to useful
/// resources like contract internal storage and a querier to retrieve blockchain objects.
/// * `_env` An environment object provided by the cosmwasm framework.  Describes the contract's
/// details, as well as blockchain information at the time of the transaction.  Unused by this
/// function, but required by cosmwasm for successfully defined query entrypoint.
/// * `msg` A custom query message enum defined by this contract to allow multiple different results
/// to be determined for this route.
#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    msg.self_validate()?;
    match msg {
        QueryMsg::QueryContractState {} => query_contract_state(deps),
    }
}

/// The entry point used when the contract admin migrates an existing instance of this contract to
/// a new stored code instance on chain.
///
/// # Parameters
///
/// * `deps` A dependencies object provided by the cosmwasm framework.  Allows access to useful
/// resources like contract internal storage and a querier to retrieve blockchain objects.
/// * `_env` An environment object provided by the cosmwasm framework.  Describes the contract's
/// details, as well as blockchain information at the time of the transaction.  Unused by this
/// function, but required by cosmwasm for successfully defined migration entrypoint.
/// * msg` A custom migrate message enum defined by this contract to allow multiple different
/// results of invoking the migrate endpoint.
#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    msg.self_validate()?;
    match msg {
        MigrateMsg::ContractUpgrade {} => migrate_contract(deps),
    }
}
