use crate::instantiate::instantiate_contract::instantiate_contract;
use crate::test::test_constants::DEFAULT_ADMIN;
use crate::types::msg::InstantiateMsg;
use cosmwasm_std::testing::{message_info, mock_env};
use cosmwasm_std::{Addr, DepsMut};

pub fn test_instantiate(deps: DepsMut) {
    test_instantiate_with_msg(deps, InstantiateMsg::default());
}

pub fn test_instantiate_with_msg(deps: DepsMut, msg: InstantiateMsg) {
    instantiate_contract(
        deps,
        mock_env(),
        message_info(&Addr::unchecked(DEFAULT_ADMIN), &[]),
        msg,
    )
    .expect("expected default instantiation to succeed");
}
