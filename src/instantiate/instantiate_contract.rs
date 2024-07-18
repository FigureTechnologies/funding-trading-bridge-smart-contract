use crate::store::contract_state::{set_contract_state_v1, ContractStateV1};
use crate::types::error::ContractError;
use crate::types::msg::InstantiateMsg;
use crate::util::provenance_utils::msg_bind_name;
use crate::util::validation_utils::check_funds_are_empty;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use result_extensions::ResultExtensions;

/// The core functionality that runs when the contract is first instantiated.  This creates the
/// singleton instance of the [ContractStateV1] used to denote the various configurations for the
/// contract, as well as optionally binding the contract's name if it does not need to be bound
/// after creation due to namespace restrictions.
///
/// # Parameters
/// * `deps` A dependencies object provided by the cosmwasm framework.  Allows access to useful
/// resources like contract internal storage and a querier to retrieve blockchain objects.
/// * `env` An environment object provided by the cosmwasm framework.  Describes the contract's
/// details, as well as blockchain information at the time of the transaction.
/// * `info` A message information object provided by the cosmwasm framework.  Describes the sender
/// of the instantiation message, as well as the funds provided as an amount during the transaction.
/// * `msg` A custom instantiation message defined by this contract for creating the initial
/// configuration used by the contract.
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
    use crate::types::error::ContractError;
    use crate::types::msg::InstantiateMsg;
    use crate::util::provenance_utils::msg_bind_name;
    use cosmwasm_std::testing::{message_info, mock_env, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{coins, Addr, CosmosMsg};
    use provwasm_mocks::mock_provenance_dependencies;
    use provwasm_std::types::provenance::name::v1::MsgBindNameRequest;

    #[test]
    fn test_rejection_for_included_funds() {
        let mut deps = mock_provenance_dependencies();
        let error = instantiate_contract(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked("test-sender"), &coins(10, "nhash")),
            InstantiateMsg::default(),
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
        let instantiate_msg = InstantiateMsg {
            name_to_bind: None,
            ..InstantiateMsg::default()
        };
        let response = instantiate_contract(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked("test-sender"), &[]),
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
        let mut instantiate_msg = InstantiateMsg {
            name_to_bind: Some("name".to_string()),
            ..InstantiateMsg::default()
        };
        instantiate_msg.name_to_bind = Some("name".to_string());
        let response = instantiate_contract(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked("test-sender"), &[]),
            instantiate_msg.clone(),
        )
        .expect("proper params should cause a successful instantiation");
        assert_eq!(
            1,
            response.messages.len(),
            "expected a single message to be emitted when a name is bound",
        );
        let message = response.messages.first().unwrap();
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
}
