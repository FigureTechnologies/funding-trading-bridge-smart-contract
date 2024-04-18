use crate::store::contract_state::{get_contract_state_v1, set_contract_state_v1, CONTRACT_TYPE};
use crate::types::error::ContractError;
use crate::util::validation_utils::check_funds_are_empty;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use result_extensions::ResultExtensions;

/// Invoked via the contract's execute functionality.  This function will only accept the request if
/// the sender is the registered contract admin in the [contract_state](crate::store::contract_state::ContractStateV1).
/// The function sets a new collection of attribute names required when an account withdraws their
/// deposit denom from the contract via the [withdraw_trading](crate::execute::withdraw_trading::withdraw_trading)
/// execution route.
///
/// # Parameters
/// * `deps` A dependencies object provided by the cosmwasm framework.  Allows access to useful
/// resources like contract internal storage and a querier to retrieve blockchain objects.
/// * `env` An environment object provided by the cosmwasm framework.  Describes the contract's
/// details, as well as blockchain information at the time of the transaction.
/// * `info` A message information object provided by the cosmwasm framework.  Describes the sender
/// of the instantiation message, as well as the funds provided as an amount during the transaction.
/// * `attributes` The new attributes that will be set in the contract state's
/// [required_withdraw_attributes](crate::store::contract_state::ContractStateV1#required_withdraw_attributes)
/// property upon successful execution.
pub fn admin_update_withdraw_required_attributes(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    attributes: Vec<String>,
) -> Result<Response, ContractError> {
    check_funds_are_empty(&info)?;
    let mut contract_state = get_contract_state_v1(deps.storage)?;
    if info.sender != contract_state.admin {
        return ContractError::NotAuthorizedError {
            message: "only the contract admin may update attributes".to_string(),
        }
        .to_err();
    }
    let previous_attributes = contract_state.required_withdraw_attributes.clone();
    contract_state.required_withdraw_attributes = attributes;
    set_contract_state_v1(deps.storage, &contract_state)?;
    Response::new()
        .add_attribute("action", "admin_update_withdraw_required_attributes")
        .add_attribute("contract_address", env.contract.address.as_str())
        .add_attribute("contract_type", CONTRACT_TYPE)
        .add_attribute("contract_name", &contract_state.contract_name)
        .add_attribute(
            "previous_attributes",
            format!("[{}]", previous_attributes.join(",").as_str()),
        )
        .add_attribute(
            "new_attributes",
            format!(
                "[{}]",
                contract_state
                    .required_withdraw_attributes
                    .join(",")
                    .as_str(),
            ),
        )
        .to_ok()
}

#[cfg(test)]
mod tests {
    use crate::execute::admin_update_withdraw_required_attributes::admin_update_withdraw_required_attributes;
    use crate::store::contract_state::CONTRACT_TYPE;
    use crate::test::attribute_extractor::AttributeExtractor;
    use crate::test::test_constants::{DEFAULT_ADMIN, DEFAULT_CONTRACT_NAME};
    use crate::test::test_instantiate::test_instantiate_with_msg;
    use crate::types::error::ContractError;
    use crate::types::msg::InstantiateMsg;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use provwasm_mocks::mock_provenance_dependencies;

    #[test]
    fn provided_funds_should_cause_an_error() {
        let mut deps = mock_provenance_dependencies();
        let error = admin_update_withdraw_required_attributes(
            deps.as_mut(),
            mock_env(),
            mock_info(DEFAULT_ADMIN, &coins(123, "countingcoins")),
            vec![],
        )
        .expect_err("an error should occur when funds are provided");
        assert!(
            matches!(&error, ContractError::InvalidFundsError { .. }),
            "unexpected error encountered: {error:?}",
        );
    }

    #[test]
    fn missing_contract_state_should_cause_an_error() {
        let mut deps = mock_provenance_dependencies();
        let error = admin_update_withdraw_required_attributes(
            deps.as_mut(),
            mock_env(),
            mock_info(DEFAULT_ADMIN, &[]),
            vec![],
        )
        .expect_err("an error should occur when the contract state is missing");
        assert!(
            matches!(&error, ContractError::StorageError { .. }),
            "unexpected error encountered: {error:?}",
        );
    }

    #[test]
    fn successful_input_should_derive_a_response_with_both_previous_and_new_values() {
        do_successful_attribute_test(
            "Both previous and new values populated",
            vec!["old-value".to_string()],
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            "[old-value]",
            "[a,b,c]",
        );
    }

    #[test]
    fn successful_input_should_derive_a_response_with_missing_previous_values() {
        do_successful_attribute_test(
            "Missing previous values",
            vec![],
            vec!["new-value".to_string()],
            "[]",
            "[new-value]",
        );
    }

    #[test]
    fn successful_input_should_derive_a_response_with_missing_new_values() {
        do_successful_attribute_test(
            "Missing new values",
            vec!["old-value".to_string()],
            vec![],
            "[old-value]",
            "[]",
        );
    }

    fn do_successful_attribute_test<S1: Into<String>, S2: Into<String>, S3: Into<String>>(
        test_name: S1,
        previous_attributes: Vec<String>,
        new_attributes: Vec<String>,
        expected_previous_attributes_attr_value: S2,
        expected_new_attributes_attr_value: S3,
    ) {
        let test_name = test_name.into();
        let mut deps = mock_provenance_dependencies();
        test_instantiate_with_msg(
            deps.as_mut(),
            InstantiateMsg {
                required_withdraw_attributes: previous_attributes,
                ..InstantiateMsg::default()
            },
        );
        let response = admin_update_withdraw_required_attributes(
            deps.as_mut(),
            mock_env(),
            mock_info(DEFAULT_ADMIN, &[]),
            new_attributes,
        )
        .unwrap_or_else(|_| {
            panic!(
                "{}: proper input on an instantiated contract should derive a successful response",
                test_name
            )
        });
        assert!(
            response.messages.is_empty(),
            "{}: no messages should be emitted in the response",
            test_name,
        );
        assert_eq!(
            6,
            response.attributes.len(),
            "{}: six attributes should be emitted in the response",
            test_name,
        );
        response.assert_attribute_with_message_prefix(
            "action",
            "admin_update_withdraw_required_attributes",
            &test_name,
        );
        response.assert_attribute_with_message_prefix(
            "contract_address",
            MOCK_CONTRACT_ADDR,
            &test_name,
        );
        response.assert_attribute_with_message_prefix("contract_type", CONTRACT_TYPE, &test_name);
        response.assert_attribute_with_message_prefix(
            "contract_name",
            DEFAULT_CONTRACT_NAME,
            &test_name,
        );
        response.assert_attribute_with_message_prefix(
            "previous_attributes",
            expected_previous_attributes_attr_value,
            &test_name,
        );
        response.assert_attribute_with_message_prefix(
            "new_attributes",
            expected_new_attributes_attr_value,
            &test_name,
        );
    }
}
