use crate::store::contract_state::get_contract_state_v1;
use crate::types::error::ContractError;
use cosmwasm_std::{to_binary, Binary, Deps};
use result_extensions::ResultExtensions;

pub fn query_contract_state(deps: Deps) -> Result<Binary, ContractError> {
    to_binary(&get_contract_state_v1(deps.storage)?)?.to_ok()
}
