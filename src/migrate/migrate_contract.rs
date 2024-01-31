use crate::store::contract_state::{
    get_contract_state_v1, set_contract_state_v1, ContractStateV1, CONTRACT_TYPE, CONTRACT_VERSION,
};
use crate::types::error::ContractError;
use cosmwasm_std::{to_binary, DepsMut, Response};
use result_extensions::ResultExtensions;
use semver::Version;

pub fn migrate_contract(deps: DepsMut) -> Result<Response, ContractError> {
    let mut contract_state = get_contract_state_v1(deps.storage)?;
    validate_migration(&contract_state)?;
    contract_state.contract_version = CONTRACT_VERSION.to_string();
    set_contract_state_v1(deps.storage, &contract_state)?;
    Response::new()
        .add_attribute("action", "migrate")
        .add_attribute("new_version", CONTRACT_VERSION)
        .set_data(to_binary(&contract_state)?)
        .to_ok()
}

fn validate_migration(contract_state: &ContractStateV1) -> Result<(), ContractError> {
    if CONTRACT_TYPE != contract_state.contract_type {
        return ContractError::MigrationError {
            message: format!(
                "target migration contract type [{CONTRACT_TYPE}] does not match stored contract type [{}]",
                contract_state.contract_type,
            ),
        }
        .to_err();
    }
    let existing_contract_version = contract_state.contract_version.parse::<Version>()?;
    let new_contract_version = CONTRACT_VERSION.parse::<Version>()?;
    if existing_contract_version >= new_contract_version {
        return ContractError::MigrationError {
            message: format!(
                "target migration contract version [{CONTRACT_VERSION}] is too low to use. stored contract version is [{existing_contract_version}]",
            )
        }
        .to_err();
    }
    ().to_ok()
}
