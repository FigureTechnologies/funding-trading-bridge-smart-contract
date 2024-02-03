use crate::instantiate::instantiate_contract::instantiate_contract;
use crate::test::test_constants::DEFAULT_ADMIN;
use crate::types::msg::InstantiateMsg;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::DepsMut;

pub fn test_instantiate(deps: DepsMut) {
    test_instantiate_with_msg(deps, InstantiateMsg::default());
}

pub fn test_instantiate_with_msg(deps: DepsMut, msg: InstantiateMsg) {
    instantiate_contract(deps, mock_env(), mock_info(DEFAULT_ADMIN, &[]), msg)
        .expect("expected default instantiation to succeed");
}
