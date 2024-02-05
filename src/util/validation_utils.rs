use crate::types::error::ContractError;
use cosmwasm_std::MessageInfo;
use result_extensions::ResultExtensions;

/// Verifies that the funds sent into the message info are empty, ensuring that the contract has not
/// received any funding when invoked.
///
/// # Parameters
///
/// * `info` A message information object provided by the cosmwasm framework.  Describes the sender
/// of the instantiation message, as well as the funds provided as an amount during the transaction.
pub fn check_funds_are_empty(info: &MessageInfo) -> Result<(), ContractError> {
    if !info.funds.is_empty() {
        ContractError::InvalidFundsError {
            message: "funds provided but empty funds required".to_string(),
        }
        .to_err()
    } else {
        ().to_ok()
    }
}

#[cfg(test)]
mod tests {
    use crate::util::validation_utils::check_funds_are_empty;
    use cosmwasm_std::testing::mock_info;
    use cosmwasm_std::{coin, coins};

    #[test]
    fn test_check_funds_are_empty_cases() {
        check_funds_are_empty(&mock_info("sender", &[]))
            .expect("empty funds should pass without an error");
        check_funds_are_empty(&mock_info("sender", &coins(10, "denom")))
            .expect_err("a single coin should produce an error");
        check_funds_are_empty(&mock_info(
            "sender",
            &[coin(1, "denomA"), coin(1, "denomB")],
        ))
        .expect_err("multiple coins should produce an error");
    }
}
