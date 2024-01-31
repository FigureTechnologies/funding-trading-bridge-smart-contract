use cosmwasm_std::MessageInfo;
use result_extensions::ResultExtensions;
use crate::types::error::ContractError;

pub fn check_funds_are_empty(info: &MessageInfo)  -> Result<(), ContractError> {
    if !info.funds.is_empty() {
        ContractError::InvalidFundsError {
            message: "funds provided but empty funds required".to_string(),
        }.to_err()
    } else {
        ().to_ok()
    }
}
