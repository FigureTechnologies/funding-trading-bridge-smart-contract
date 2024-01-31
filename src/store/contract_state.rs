use crate::types::denom::Denom;
use crate::types::error::ContractError;
use cosmwasm_std::{Addr, Storage};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONTRACT_TYPE: &str = env!("CARGO_CRATE_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const NAMESPACE_CONTRACT_STATE_V1: &str = "contract_state_v1";
const CONTRACT_STATE_V1: Item<ContractStateV1> = Item::new(NAMESPACE_CONTRACT_STATE_V1);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ContractStateV1 {
    pub admin: Addr,
    pub contract_name: String,
    pub contract_type: String,
    pub contract_version: String,
    pub deposit_marker: Denom,
    pub trading_marker: Denom,
}
impl ContractStateV1 {
    pub fn new<S: Into<String>>(
        admin: Addr,
        contract_name: S,
        deposit_marker: &Denom,
        trading_marker: &Denom,
    ) -> Self {
        Self {
            admin,
            contract_name: contract_name.into(),
            contract_type: CONTRACT_TYPE.to_string(),
            contract_version: CONTRACT_VERSION.to_string(),
            deposit_marker: Denom::new(&deposit_marker.name, deposit_marker.precision.u64()),
            trading_marker: Denom::new(&trading_marker.name, trading_marker.precision.u64()),
        }
    }
}

pub fn set_contract_state_v1(
    storage: &mut dyn Storage,
    contract_state: &ContractStateV1,
) -> Result<(), ContractError> {
    CONTRACT_STATE_V1
        .save(storage, contract_state)
        .map_err(|e| ContractError::StorageError {
            message: format!("{e:?}"),
        })
}

pub fn get_contract_state_v1(storage: &dyn Storage) -> Result<ContractStateV1, ContractError> {
    CONTRACT_STATE_V1
        .load(storage)
        .map_err(|e| ContractError::StorageError {
            message: format!("{e:?}"),
        })
}

#[cfg(test)]
mod tests {
    // TODO: Testing if time allows
}
