use crate::types::error::ContractError;
use cosmwasm_std::MessageInfo;
use result_extensions::ResultExtensions;
use uuid::Uuid;

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

/// Verifies that the provided string is a valid attribute name for the Provenance Blockchain,
/// following their rules:
/// - The attribute must not be empty.
/// - The attribute must have at maximum 16 segments, separated by periods.
/// - Each segment must be between 2 and 32 characters.
/// - Each segment must be alphanumeric.
/// - Each segment can have a single '-' character, or be a valid uuid if it includes '-' characters.
///
/// Referenced code (at time of writing): https://github.com/provenance-io/provenance/blob/main/x/name/types/name.go#L82
/// Referenced documentation describing these requirements (at time of writing): https://github.com/provenance-io/provenance/blob/main/x/name/spec/01_concepts.md
///
/// # Parameters
///
/// * `name` The fully-qualified attribute name.  Ex: name-thing.name
pub fn validate_attribute_name<S: Into<String>>(name: S) -> Result<(), ContractError> {
    let name = name.into();
    let name_parts = name.split('.').collect::<Vec<&str>>();
    if name_parts.len() > 16 {
        return ContractError::InvalidFormatError {
            message: format!("Attribute name {name} has too many segments"),
        }
        .to_err();
    }
    if name_parts
        .iter()
        .any(|part| !(2usize..33usize).contains(&part.len()))
    {
        return ContractError::InvalidFormatError {
            message: format!(
                "Attribute name {name} contains at least one segment with an incorrect size"
            ),
        }
        .to_err();
    }
    if name_parts.iter().any(|part| {
        // A segment is immediately valid if it conforms as a valid UUID
        Uuid::parse_str(part).is_err()
            // A segment can include only one dash
            && (part.chars().filter(|c| c == &'-').count() > 1
            // A segment must be fully alphanumeric, barring the single dash allowance
                || !part
                    .chars()
                    .filter(|c| c != &'-')
                    .all(char::is_alphanumeric))
    }) {
        return ContractError::InvalidFormatError {
            message: format!(
                "Attribute name {name} contains at least one segment that is not a uuid, has more than one dash character, or violates alphanumeric values"
            ),
        }
        .to_err();
    }
    ().to_ok()
}

#[cfg(test)]
mod tests {
    use crate::util::validation_utils::{check_funds_are_empty, validate_attribute_name};
    use cosmwasm_std::testing::message_info;
    use cosmwasm_std::{coin, coins, Addr};

    #[test]
    fn test_check_funds_are_empty_cases() {
        check_funds_are_empty(&message_info(&Addr::unchecked("sender"), &[]))
            .expect("empty funds should pass without an error");
        check_funds_are_empty(&message_info(
            &Addr::unchecked("sender"),
            &coins(10, "denom"),
        ))
        .expect_err("a single coin should produce an error");
        check_funds_are_empty(&message_info(
            &Addr::unchecked("sender"),
            &[coin(1, "denomA"), coin(1, "denomB")],
        ))
        .expect_err("multiple coins should produce an error");
    }

    #[test]
    fn test_valid_attribute_name_use_cases() {
        // Invalid Cases:
        // Empty string is not allowed
        assert_attribute_invalid("");
        // 16 segments at max
        assert_attribute_invalid("aa.aa.aa.aa.aa.aa.aa.aa.aa.aa.aa.aa.aa.aa.aa.aa.aa");
        // Empty segment is not allowed
        assert_attribute_invalid("part.");
        assert_attribute_invalid(".part");
        // Each segment must be between 2 and 32 characters
        assert_attribute_invalid("a");
        assert_attribute_invalid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        assert_attribute_invalid("validthing.b");
        // Each segment must be only alphanumeric
        assert_attribute_invalid("properformat.iµhñuo˜¨ñ:");
        assert_attribute_invalid("hellothere.😄kjdsfijds.93ksdjlfd008");
        // No whitespace in segments
        assert_attribute_invalid("normalish.butthen.itgot weird");
        assert_attribute_invalid("aw jeez.rick");
        // Includes too many dashes
        assert_attribute_invalid("--.uu.sdfsd");
        assert_attribute_invalid("a-b.haha-asdddd-djdjdj");

        // Valid Cases:
        // Single segment
        assert_attribute_valid("onename");
        // Max segments
        assert_attribute_valid("aa.aa.aa.aa.aa.aa.aa.aa.aa.aa.aa.aa.aa.aa.aa.aa");
        // Character limits
        assert_attribute_valid("aa");
        assert_attribute_valid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        assert_attribute_valid("aa.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        // Alphanumeric
        assert_attribute_valid("1234.jjjjdijdjidJAUSUD.902NJSAhdsjs");
        // UUID segments
        assert_attribute_invalid("9372bae6-3f0a-11ef-b0d9-b3a1f5fefa08.aa");
        // Dash segments
        assert_attribute_valid("this-is.a-valid.name");
    }

    fn assert_attribute_valid<S: Into<String>>(attribute_name: S) {
        let attribute_name = attribute_name.into();
        match validate_attribute_name(&attribute_name) {
            Ok(()) => {}
            Err(e) => {
                panic!(
                    "Expected attribute {attribute_name} to be valid, but got: {:?}",
                    e
                )
            }
        };
    }

    fn assert_attribute_invalid<S: Into<String>>(attribute_name: S) {
        let attribute_name = attribute_name.into();
        validate_attribute_name(&attribute_name).expect_err(&format!(
            "expected attribute {attribute_name} to be invalid"
        ));
    }
}
