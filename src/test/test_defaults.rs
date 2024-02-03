use crate::test::test_constants::{
    DEFAULT_BOUND_NAME, DEFAULT_CONTRACT_NAME, DEFAULT_DEPOSIT_DENOM_NAME,
    DEFAULT_DEPOSIT_DENOM_PRECISION, DEFAULT_REQUIRED_DEPOSIT_ATTRIBUTE,
    DEFAULT_REQUIRED_WITHDRAW_ATTRIBUTE, DEFAULT_TRADING_DENOM_NAME,
    DEFAULT_TRADING_DENOM_PRECISION,
};
use crate::types::denom::Denom;
use crate::types::msg::InstantiateMsg;
use cosmwasm_std::Uint64;

impl Default for InstantiateMsg {
    fn default() -> Self {
        Self {
            contract_name: DEFAULT_CONTRACT_NAME.to_string(),
            deposit_marker: Denom {
                name: DEFAULT_DEPOSIT_DENOM_NAME.to_string(),
                precision: Uint64::new(DEFAULT_DEPOSIT_DENOM_PRECISION),
            },
            trading_marker: Denom {
                name: DEFAULT_TRADING_DENOM_NAME.to_string(),
                precision: Uint64::new(DEFAULT_TRADING_DENOM_PRECISION),
            },
            required_deposit_attributes: vec![DEFAULT_REQUIRED_DEPOSIT_ATTRIBUTE.to_string()],
            required_withdraw_attributes: vec![DEFAULT_REQUIRED_WITHDRAW_ATTRIBUTE.to_string()],
            name_to_bind: Some(DEFAULT_BOUND_NAME.to_string()),
        }
    }
}
