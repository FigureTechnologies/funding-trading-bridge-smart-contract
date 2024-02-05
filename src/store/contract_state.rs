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

/// Stores the core contract configurations created on instantiation and modified on migration.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ContractStateV1 {
    /// The bech32 address of the account that has admin rights within this contract.
    pub admin: Addr,
    /// A free-form name defining this particular contract instance.  Used for identification on
    /// query purposes only.
    pub contract_name: String,
    /// The crate name, used to ensure that newly-migrated instances match the same contract format.
    pub contract_type: String,
    /// The crate version, used to ensure that newly-migrated instances do not attempt to use an
    /// identical or older version.
    pub contract_version: String,
    /// Defines the marker denom that is deposited to this contract in exchange for [trading_marker](ContractStateV1#trading_marker)
    /// denom.
    pub deposit_marker: Denom,
    /// Defines the marker denom that is sent to accounts from this contract in exchange for
    /// [deposit_marker](ContractStateV1#deposit_marker).
    pub trading_marker: Denom,
    /// Defines any blockchain attributes required on accounts in order to execute the [fund_trading](crate::execute::fund_trading::fund_trading)
    /// execution route.
    pub required_deposit_attributes: Vec<String>,
    /// Defines any blockchain attributes required on accounts in order to execute the
    /// [withdraw_trading](crate::execute::withdraw_trading::withdraw_trading) execution route.
    pub required_withdraw_attributes: Vec<String>,
}
impl ContractStateV1 {
    /// Constructs a new instance of this struct.
    ///
    /// # Parameters
    /// * `admin` The bech32 address of the account that has admin rights within this contract.
    /// * `contract_name` A free-form name defining this particular contract instance.  Used for
    /// identification on query purposes only.
    /// * `deposit_marker` Defines the marker denom that is deposited to this contract in exchange
    /// for [trading_marker](ContractStateV1#trading_marker) denom.
    /// * `trading_marker` Defines the marker denom that is sent to accounts from this contract in
    /// exchange for [deposit_marker](ContractStateV1#deposit_marker).
    /// * `required_deposit_attributes` Defines any blockchain attributes required on accounts in
    /// order to execute the [fund_trading](crate::execute::fund_trading::fund_trading) execution
    /// route.
    /// * `required_withdraw_attributes` Defines any blockchain attributes required on accounts in
    /// order to execute the [withdraw_trading](crate::execute::withdraw_trading::withdraw_trading)
    /// execution route.
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

/// Overwrites the existing singleton contract storage instance of [ContractStateV1] with the input
/// reference.  An error is returned if the store write is unsuccessful.
///
/// # Parameters
///
/// * `storage` A mutable instance of the contract storage value, allowing internal store
/// manipulation.
/// * `contract_state` The new value for which an internal storage write will be done.
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

/// Fetches the current contract instance of contract state.  This call should never fail because
/// the state is set on contract instantiation, but an error will be returned if store communication
/// fails.
///
/// # Parameters
///
/// * `storage` An immutable instance of the contract storage value, allowing internal store data
/// fetches.
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
