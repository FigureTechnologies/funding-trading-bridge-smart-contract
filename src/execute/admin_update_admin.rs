use crate::store::contract_state::{get_contract_state_v1, set_contract_state_v1, CONTRACT_TYPE};
use crate::types::error::ContractError;
use crate::util::validation_utils::check_funds_are_empty;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use result_extensions::ResultExtensions;

/// Invoked via the contract's execute functionality.  This function will only accept the request if
/// the sender is the registered contract admin in the [contract state](crate::store::contract_state::ContractStateV1).
/// The function swaps the current value in the contract state for the newly-provided value,
/// effectively removing the previous admin and setting a new one.
///
/// # Parameters
/// * `deps` A dependencies object provided by the cosmwasm framework.  Allows access to useful
/// resources like contract internal storage and a querier to retrieve blockchain objects.
/// * `env` An environment object provided by the cosmwasm framework.  Describes the contract's
/// details, as well as blockchain information at the time of the transaction.
/// * `info` A message information object provided by the cosmwasm framework.  Describes the sender
/// of the instantiation message, as well as the funds provided as an amount during the transaction.
/// * `new_admin_address` The bech32 Provenance Blockchain address that will become the new admin
/// upon successful invocation of this function.
pub fn admin_update_admin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_admin_address: String,
) -> Result<Response, ContractError> {
    check_funds_are_empty(&info)?;
    let mut contract_state = get_contract_state_v1(deps.storage)?;
    if info.sender != contract_state.admin {
        return ContractError::NotAuthorizedError {
            message: "only the contract admin may change the admin".to_string(),
        }
        .to_err();
    }
    let previous_admin_addr = contract_state.admin.to_owned();
    let new_admin_addr = deps.api.addr_validate(new_admin_address.as_str())?;
    contract_state.admin = new_admin_addr;
    set_contract_state_v1(deps.storage, &contract_state)?;
    Response::new()
        .add_attribute("action", "admin_update_admin")
        .add_attribute("contract_address", env.contract.address.as_str())
        .add_attribute("contract_type", CONTRACT_TYPE)
        .add_attribute("contract_name", &contract_state.contract_name)
        .add_attribute("previous_admin", previous_admin_addr.as_str())
        .add_attribute("new_admin", new_admin_address)
        .to_ok()
}

#[cfg(test)]
mod tests {
    use crate::execute::admin_update_admin::admin_update_admin;
    use crate::store::contract_state::CONTRACT_TYPE;
    use crate::test::attribute_extractor::AttributeExtractor;
    use crate::test::test_constants::{DEFAULT_ADMIN, DEFAULT_CONTRACT_NAME};
    use crate::test::test_instantiate::test_instantiate;
    use crate::types::error::ContractError;
    use cosmwasm_std::testing::{message_info, mock_env, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{coins, Addr};
    use provwasm_mocks::mock_provenance_dependencies;

    #[test]
    fn provided_funds_should_cause_an_error() {
        let mut deps = mock_provenance_dependencies();
        let error = admin_update_admin(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(DEFAULT_ADMIN), &coins(10, "nhash")),
            "test".to_string(),
        )
        .expect_err("an error should occur when funds are provided");
        assert!(
            matches!(&error, ContractError::InvalidFundsError { .. },),
            "unexpected error encountered: {error:?}",
        );
    }

    #[test]
    fn missing_contract_state_should_cause_an_error() {
        let mut deps = mock_provenance_dependencies();
        let error = admin_update_admin(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(DEFAULT_ADMIN), &[]),
            "test".to_string(),
        )
        .expect_err("an error should occur when the contract state is missing");
        assert!(
            matches!(&error, ContractError::StorageError { .. },),
            "unexpected error encountered: {error:?}",
        );
    }

    #[test]
    fn successful_input_should_derive_a_response() {
        let mut deps = mock_provenance_dependencies();
        test_instantiate(deps.as_mut());
        let new_admin = "new-admin".to_string();
        let response = admin_update_admin(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(DEFAULT_ADMIN), &[]),
            new_admin.to_owned(),
        )
        .expect("proper input on an instantiated contract should derive a successful response");
        assert!(
            response.messages.is_empty(),
            "no messages should be emitted in the response"
        );
        assert_eq!(
            6,
            response.attributes.len(),
            "six attributes should be emitted in the response"
        );
        response.assert_attribute("action", "admin_update_admin");
        response.assert_attribute("contract_address", MOCK_CONTRACT_ADDR);
        response.assert_attribute("contract_type", CONTRACT_TYPE);
        response.assert_attribute("contract_name", DEFAULT_CONTRACT_NAME);
        response.assert_attribute("previous_admin", DEFAULT_ADMIN);
        response.assert_attribute("new_admin", new_admin);
    }
}
