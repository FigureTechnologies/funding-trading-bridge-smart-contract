use crate::store::contract_state::{get_contract_state_v1, set_contract_state_v1, CONTRACT_TYPE};
use crate::types::error::ContractError;
use crate::util::validation_utils::check_funds_are_empty;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use result_extensions::ResultExtensions;

pub fn admin_update_deposit_required_attributes(
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
    let previous_attributes = contract_state.required_deposit_attributes.clone();
    contract_state.required_deposit_attributes = attributes;
    set_contract_state_v1(deps.storage, &contract_state)?;
    Response::new()
        .add_attribute("action", "admin_update_deposit_required_attributes")
        .add_attribute("contract_address", env.contract.address.as_str())
        .add_attribute("contract_type", CONTRACT_TYPE)
        .add_attribute("contract_name", &contract_state.contract_name)
        .add_attribute(
            "previous_attributes",
            previous_attributes.join(",").as_str(),
        )
        .add_attribute(
            "new_attributes",
            contract_state
                .required_deposit_attributes
                .join(",")
                .as_str(),
        )
        .to_ok()
}

#[cfg(test)]
mod tests {
    use crate::execute::admin_update_deposit_required_attributes::admin_update_deposit_required_attributes;
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
        let error = admin_update_deposit_required_attributes(
            deps.as_mut(),
            mock_env(),
            mock_info(DEFAULT_ADMIN, &coins(400, "fourhundredcoins")),
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
        let error = admin_update_deposit_required_attributes(
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
    fn successful_input_should_derive_a_response() {
        let mut deps = mock_provenance_dependencies();
        test_instantiate_with_msg(
            deps.as_mut(),
            InstantiateMsg {
                required_deposit_attributes: vec!["prevA".to_string(), "prevB".to_string()],
                ..InstantiateMsg::default()
            },
        );
        let response = admin_update_deposit_required_attributes(
            deps.as_mut(),
            mock_env(),
            mock_info(DEFAULT_ADMIN, &[]),
            vec!["new".to_string()],
        )
        .expect("proper input on an instantiated contract should derive a successful response");
        assert!(
            response.messages.is_empty(),
            "no messages should be emitted in the response",
        );
        assert_eq!(
            6,
            response.attributes.len(),
            "six attributes should be emitted in the response",
        );
        response.assert_attribute("action", "admin_update_deposit_required_attributes");
        response.assert_attribute("contract_address", MOCK_CONTRACT_ADDR);
        response.assert_attribute("contract_type", CONTRACT_TYPE);
        response.assert_attribute("contract_name", DEFAULT_CONTRACT_NAME);
        response.assert_attribute("previous_attributes", "prevA,prevB");
        response.assert_attribute("new_attributes", "new");
    }
}
