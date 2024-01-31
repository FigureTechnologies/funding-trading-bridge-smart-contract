use crate::types::denom::{Denom, DenomConversion};
use crate::types::error::ContractError;
use result_extensions::ResultExtensions;

pub fn convert_denom(
    source_amount: u128,
    source_denom: &Denom,
    target_denom: &Denom,
) -> Result<DenomConversion, ContractError> {
    let source_precision = source_denom.precision.u64();
    let target_precision = target_denom.precision.u64();
    let precision_diff = u32::try_from((source_precision as i64 - target_precision as i64).abs())
        .map_err(|e| ContractError::ConversionError {
            message: format!("source precision [{source_precision}] and target precision [{target_precision}] have too large a difference to convert: {e:?}")
        })?;
    let precision_modifier = 10u128.pow(precision_diff);
    let (target_amount, remainder) = match source_precision {
        // If source precision is greater, the value needs some of its values trimmed off for target
        // conversion amount.
        s if s > target_precision => {
            let target_amount = source_amount / precision_modifier;
            let remainder = source_amount % precision_modifier;
            (target_amount, remainder)
        }
        // If source precision is lesser, the value should get zeroes added to become the target.
        // The value increases, so there is never a remainder.
        s if s < target_precision => {
            let target_amount = source_amount * precision_modifier;
            (target_amount, 0u128)
        }
        // If the precisions are equal, then it is a 1 to 1 conversion and the result is the input
        _ => (source_amount, 0u128),
    };
    DenomConversion {
        source_amount,
        target_amount,
        remainder,
    }
    .to_ok()
}

#[cfg(test)]
pub mod tests {
    use crate::types::denom::Denom;
    use crate::util::conversion_utils::convert_denom;

    #[test]
    fn test_source_precision_greater_than_target_precision() {
        let amount = 123456789;
        let source_denom = Denom::new("source", 4);
        let target_denom = Denom::new("target", 1);
        let very_large_result = convert_denom(amount, &source_denom, &target_denom)
            .expect("The conversion should succeed with valid inputs");
        assert_eq!(
            123456, very_large_result.target_amount,
            "Value {amount}: The resulting amount should be all values that fit into the target destination type",
        );
        assert_eq!(
            789, very_large_result.remainder,
            "Value {amount}: The remainder amount should equate to all precision that could not be converted",
        );
        let amount = 1000;
        let just_large_enough_result = convert_denom(amount, &source_denom, &target_denom)
            .expect("The conversion should succeed with valid inputs");
        assert_eq!(
            1, just_large_enough_result.target_amount,
            "Value {amount}: The resulting amount should be just the value before the decimal place",
        );
        assert_eq!(
            0, just_large_enough_result.remainder,
            "Value {amount}: There should be no remainder because all values after the decimal place were zeroes",
        );
        let amount = 1101;
        let small_overflow_result = convert_denom(amount, &source_denom, &target_denom)
            .expect("The conversion should succeed with valid inputs");
        assert_eq!(
            1, small_overflow_result.target_amount,
            "Value {amount}: The resulting amount should be the value before the decimal place",
        );
        assert_eq!(
            101, small_overflow_result.remainder,
            "Value {amount}: The remainder should properly contain the overflow",
        );
        let amount = 123;
        let full_overflow_result = convert_denom(amount, &source_denom, &target_denom)
            .expect("The conversion should succeed with valid inputs");
        assert_eq!(
            0, full_overflow_result.target_amount,
            "Value {amount}: The resulting amount should be zero because all converted amounts were remainders",
        );
        assert_eq!(
            123, full_overflow_result.remainder,
            "Value {amount}: The remainder should be the whole value due to overflow past precision conversion",
        );
        let amount = 0;
        let zero_result = convert_denom(amount, &source_denom, &target_denom)
            .expect("The conversion should succeed with valid inputs");
        assert_eq!(
            0, zero_result.target_amount,
            "Value {amount}: The target amount should be zero because the initial value was zero",
        );
        assert_eq!(
            0, zero_result.remainder,
            "Value {amount}: The remainder should be zero because the initial value was zero",
        );
    }

    #[test]
    fn test_source_precision_lower_than_target_precision() {
        let amount = 123456789;
        let source_denom = Denom::new("source", 1);
        let target_denom = Denom::new("target", 4);
        let very_large_result = convert_denom(amount, &source_denom, &target_denom)
            .expect("The conversion should succeed with valid inputs");
        assert_eq!(
            123456789000, very_large_result.target_amount,
            "Value {amount}: The target amount should have extra zeroes for the increased precision",
        );
        assert_eq!(
            0, very_large_result.remainder,
            "Value {amount}: A conversion with lower source precision than target should never have a remainder",
        );
        let amount = 2;
        let simple_result = convert_denom(amount, &source_denom, &target_denom)
            .expect("The conversion should succeed with valid inputs");
        assert_eq!(
            2000, simple_result.target_amount,
            "Value {amount}: The target amount should have extra zeroes for the increased precision",
        );
        assert_eq!(
            0, simple_result.remainder,
            "Value {amount}: A conversion with lower source precision than target should never have a remainder",
        );
        let amount = 0;
        let zero_result = convert_denom(amount, &source_denom, &target_denom)
            .expect("The conversion should succeed with valid inputs");
        assert_eq!(
            0, zero_result.target_amount,
            "Value {amount}: The target amount should be zero because the input was zero",
        );
        assert_eq!(
            0, zero_result.remainder,
            "Value {amount}: A conversion with lower source precision than target should never have a remainder",
        );
    }

    #[test]
    fn test_source_precision_equal_to_target_precision() {
        let amount = 123456789;
        let source_denom = Denom::new("source", 3);
        let target_denom = Denom::new("target", 3);
        let large_result = convert_denom(amount, &source_denom, &target_denom)
            .expect("The conversion should succeed with valid inputs");
        assert_eq!(
            amount, large_result.target_amount,
            "Value {amount}: The target amount should equate to the input because there is no precision diff",
        );
        assert_eq!(
            0, large_result.remainder,
            "Value {amount}: The remainder should be zero because no conversion was necessary",
        );
        let amount = 6;
        let simple_result = convert_denom(amount, &source_denom, &target_denom)
            .expect("The conversion should succeed with valid inputs");
        assert_eq!(
            amount, simple_result.target_amount,
            "Value {amount}: The target amount should equate to the input because there is no precision diff",
        );
        assert_eq!(
            0, simple_result.remainder,
            "Value {amount}: The remainder should be zero because no conversion was necessary",
        );
        let amount = 0;
        let zero_result = convert_denom(amount, &source_denom, &target_denom)
            .expect("The conversion should succeed with valid inputs");
        assert_eq!(
            0, zero_result.target_amount,
            "Value {amount}: The target amount should be zero because the input was zero",
        );
        assert_eq!(
            0, zero_result.remainder,
            "Value {amount}: The remainder should be zero because the input was zero",
        );
    }

    #[test]
    fn test_example_use_case() {
        let amount = 987123456;
        let source_denom = Denom::new("trading", 6);
        let target_denom = Denom::new("deposit", 2);
        let result = convert_denom(amount, &source_denom, &target_denom)
            .expect("The conversion should succeed with valid inputs");
        assert_eq!(
            98712, result.target_amount,
            "Input {amount}: Expected the proper target amount output from input",
        );
        assert_eq!(
            3456, result.remainder,
            "Input {amount}: Expected the proper remainder amount from input",
        );
    }
}
