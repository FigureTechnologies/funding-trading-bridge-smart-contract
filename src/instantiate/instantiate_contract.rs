use crate::store::contract_state::{set_contract_state_v1, ContractStateV1};
use crate::types::error::ContractError;
use crate::types::msg::InstantiateMsg;
use crate::util::provenance_utils::msg_bind_name;
use crate::util::validation_utils::check_funds_are_empty;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use result_extensions::ResultExtensions;

pub fn instantiate_contract(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    check_funds_are_empty(&info)?;
    let contract_state = ContractStateV1::new(
        info.sender,
        &msg.contract_name,
        &msg.deposit_marker,
        &msg.trading_marker,
        &msg.required_deposit_attributes,
        &msg.required_withdraw_attributes,
    );
    set_contract_state_v1(deps.storage, &contract_state)?;
    let mut response = Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", &msg.contract_name)
        .add_attribute("deposit_marker_name", &msg.deposit_marker.name)
        .add_attribute("trading_marker_name", &msg.trading_marker.name);
    if let Some(name) = msg.name_to_bind {
        response = response
            .add_message(msg_bind_name(&name, env.contract.address, true)?)
            .add_attribute("contract_bound_with_name", name)
    }
    response.to_ok()
}

#[cfg(test)]
mod tests {
    use crate::instantiate::instantiate_contract::instantiate_contract;
    use crate::test::attribute_extractor::AttributeExtractor;
    use crate::types::denom::Denom;
    use crate::types::error::ContractError;
    use crate::types::msg::InstantiateMsg;
    use crate::util::provenance_utils::msg_bind_name;
    use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{coins, CosmosMsg, Uint64};
    use provwasm_mocks::mock_provenance_dependencies;
    use provwasm_std::types::provenance::name::v1::MsgBindNameRequest;

    #[test]
    fn test_rejection_for_included_funds() {
        let mut deps = mock_provenance_dependencies();
        let error = instantiate_contract(
            deps.as_mut(),
            mock_env(),
            mock_info("test-sender", &coins(10, "nhash")),
            default_instantiate(),
        )
        .expect_err("an error should occur when providing funds");
        let _expected_err_msg = "funds provided but empty funds required".to_string();
        assert!(
            matches!(
                error,
                ContractError::InvalidFundsError {
                    message: _expected_err_msg
                },
            ),
            "unexpected error emitted when no funds provided",
        );
    }

    #[test]
    fn test_successful_instantiate_without_name_bind() {
        let mut deps = mock_provenance_dependencies();
        let instantiate_msg = default_instantiate();
        let response = instantiate_contract(
            deps.as_mut(),
            mock_env(),
            mock_info("test-sender", &[]),
            instantiate_msg.clone(),
        )
        .expect("proper params should cause a successful instantiation");
        assert!(
            response.messages.is_empty(),
            "no messages should be emitted when a name isn't bound",
        );
        assert_eq!(
            4,
            response.attributes.len(),
            "expected four attributes to be emitted when no name is bound",
        );
        response.assert_attribute("action", "instantiate");
        response.assert_attribute("contract_name", instantiate_msg.contract_name);
        response.assert_attribute("deposit_marker_name", instantiate_msg.deposit_marker.name);
        response.assert_attribute("trading_marker_name", instantiate_msg.trading_marker.name);
    }

    #[test]
    fn test_successful_instantiate_with_name_bind() {
        let mut deps = mock_provenance_dependencies();
        let mut instantiate_msg = default_instantiate();
        instantiate_msg.name_to_bind = Some("name".to_string());
        let response = instantiate_contract(
            deps.as_mut(),
            mock_env(),
            mock_info("test-sender", &[]),
            instantiate_msg.clone(),
        )
        .expect("proper params should cause a successful instantiation");
        assert_eq!(
            1,
            response.messages.len(),
            "expected a single message to be emitted when a name is bound",
        );
        let message = response.messages.first().unwrap();
        println!("{message:?}");
        match &message.msg {
            CosmosMsg::Stargate { type_url: _, value } => {
                let expected_name_bind = msg_bind_name("name", MOCK_CONTRACT_ADDR, true)
                    .expect("failed to generate expected msg format");
                let name_bind = MsgBindNameRequest::try_from(value.to_owned())
                    .expect("expected the name msg binary to deserialize correctly");
                assert_eq!(
                    expected_name_bind, name_bind,
                    "expected the correct name msg to be deserialized",
                );
            }
            msg => panic!("unexpected msg format for bind name: {msg:?}"),
        }
        assert_eq!(
            5,
            response.attributes.len(),
            "expected five attributes to be emitted when a name is bound",
        );
        response.assert_attribute("action", "instantiate");
        response.assert_attribute("contract_name", instantiate_msg.contract_name);
        response.assert_attribute("deposit_marker_name", instantiate_msg.deposit_marker.name);
        response.assert_attribute("trading_marker_name", instantiate_msg.trading_marker.name);
        response.assert_attribute("contract_bound_with_name", "name");
    }

    fn default_instantiate() -> InstantiateMsg {
        InstantiateMsg {
            contract_name: "test-contract".to_string(),
            deposit_marker: Denom {
                name: "deposit".to_string(),
                precision: Uint64::new(2),
            },
            trading_marker: Denom {
                name: "trading".to_string(),
                precision: Uint64::new(6),
            },
            required_deposit_attributes: vec![],
            required_withdraw_attributes: vec![],
            name_to_bind: None,
        }
    }
}
