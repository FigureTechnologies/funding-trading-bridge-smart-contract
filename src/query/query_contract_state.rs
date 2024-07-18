use crate::store::contract_state::get_contract_state_v1;
use crate::types::error::ContractError;
use cosmwasm_std::{to_json_binary, Binary, Deps};
use result_extensions::ResultExtensions;

/// Fetches the current values within the [contract state](crate::store::contract_state::ContractStateV1).
///
/// # Parameters
///
/// * `deps` A dependencies object provided by the cosmwasm framework.  Allows access to useful
/// resources like contract internal storage and a querier to retrieve blockchain objects.
pub fn query_contract_state(deps: Deps) -> Result<Binary, ContractError> {
    to_json_binary(&get_contract_state_v1(deps.storage)?)?.to_ok()
}

#[cfg(test)]
mod tests {
    use crate::query::query_contract_state::query_contract_state;
    use crate::store::contract_state::{get_contract_state_v1, ContractStateV1};
    use crate::test::test_instantiate::test_instantiate;
    use cosmwasm_std::from_json;
    use provwasm_mocks::mock_provenance_dependencies;

    #[test]
    fn test_query_with_no_storage() {
        let deps = mock_provenance_dependencies();
        query_contract_state(deps.as_ref())
            .expect_err("an error should occur when no contract state has been initialized");
    }

    #[test]
    fn test_query_with_stored_state() {
        let mut deps = mock_provenance_dependencies();
        test_instantiate(deps.as_mut());
        let expected_state = get_contract_state_v1(&deps.storage)
            .expect("contract state should load after instantiation");
        let state_from_query = query_contract_state(deps.as_ref())
            .expect("contract state binary should load from query");
        let state_from_query = from_json::<ContractStateV1>(&state_from_query)
            .expect("contract state binary should properly deserialize");
        assert_eq!(
            expected_state, state_from_query,
            "the contract state from storage should equate to the deserialized value from query",
        );
    }
}
