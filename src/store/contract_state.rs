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
    pub required_deposit_attributes: Vec<String>,
    pub required_withdraw_attributes: Vec<String>,
}
impl ContractStateV1 {
    pub fn new<S: Into<String>>(
        admin: Addr,
        contract_name: S,
        deposit_marker: &Denom,
        trading_marker: &Denom,
        required_deposit_attributes: &[String],
        required_withdraw_attributes: &[String],
    ) -> Self {
        Self {
            admin,
            contract_name: contract_name.into(),
            contract_type: CONTRACT_TYPE.to_string(),
            contract_version: CONTRACT_VERSION.to_string(),
            deposit_marker: Denom::new(&deposit_marker.name, deposit_marker.precision.u64()),
            trading_marker: Denom::new(&trading_marker.name, trading_marker.precision.u64()),
            required_deposit_attributes: required_deposit_attributes.to_vec(),
            required_withdraw_attributes: required_withdraw_attributes.to_vec(),
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
    use crate::store::contract_state::{
        get_contract_state_v1, set_contract_state_v1, ContractStateV1, CONTRACT_TYPE,
        CONTRACT_VERSION,
    };
    use crate::types::denom::Denom;
    use cosmwasm_std::{Addr, Uint64};
    use provwasm_mocks::mock_provenance_dependencies;

    #[test]
    fn test_new_contract_state_v1() {
        let state = ContractStateV1::new(
            Addr::unchecked("admin"),
            "contract_name",
            &Denom {
                name: "deposit".to_string(),
                precision: Uint64::new(10),
            },
            &Denom {
                name: "trading".to_string(),
                precision: Uint64::new(4),
            },
            &vec!["required".to_string()],
            &vec!["required".to_string()],
        );
        assert_eq!(
            "admin",
            state.admin.as_str(),
            "the admin value should be set correctly",
        );
        assert_eq!(
            "contract_name", state.contract_name,
            "the contract name value should be set correctly",
        );
        assert_eq!(
            CONTRACT_TYPE, state.contract_type,
            "the contract type value should be set correctly",
        );
        assert_eq!(
            CONTRACT_VERSION.to_string(),
            state.contract_version,
            "the contract version value should be set correctly",
        );
        assert_eq!(
            "deposit", state.deposit_marker.name,
            "the deposit marker name should be set correctly",
        );
        assert_eq!(
            10,
            state.deposit_marker.precision.u64(),
            "the deposit marker precision should be set correctly",
        );
        assert_eq!(
            "trading", state.trading_marker.name,
            "the trading marker name should be set correctly",
        );
        assert_eq!(
            4,
            state.trading_marker.precision.u64(),
            "the trading marker precision should be set correctly",
        );
        assert_eq!(
            vec!["required"],
            state.required_deposit_attributes,
            "the required deposit attributes should have the proper value",
        );
        assert_eq!(
            vec!["required".to_string()],
            state.required_withdraw_attributes,
            "the required withdraw attributes should have the proper value",
        );
    }

    #[test]
    fn test_get_set_contract_state() {
        let mut deps = mock_provenance_dependencies();
        get_contract_state_v1(&deps.storage)
            .expect_err("get contract state before it has been set should cause an error");
        let contract_state = ContractStateV1::new(
            Addr::unchecked("admin"),
            "contract-name",
            &Denom::new("deposit", 10),
            &Denom::new("trading", 4),
            &["required_deposit".to_string()],
            &["required_withdraw".to_string()],
        );
        set_contract_state_v1(&mut deps.storage, &contract_state)
            .expect("setting contract state should succeed");
        let from_storage_state =
            get_contract_state_v1(&deps.storage).expect("getting contract state should succeed");
        assert_eq!(
            contract_state, from_storage_state,
            "expected the state value from storage to equate to the value stored",
        );
    }
}
