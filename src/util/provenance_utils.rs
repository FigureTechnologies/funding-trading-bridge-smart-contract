use crate::types::error::ContractError;
use cosmwasm_std::{Deps, DepsMut};
use provwasm_std::types::cosmos::bank::v1beta1::BankQuerier;
use provwasm_std::types::cosmos::base::query::v1beta1::PageRequest;
use provwasm_std::types::provenance::attribute::v1::AttributeQuerier;
use provwasm_std::types::provenance::marker::v1::{MarkerAccount, MarkerQuerier};
use provwasm_std::types::provenance::name::v1::{MsgBindNameRequest, NameRecord};
use result_extensions::ResultExtensions;

/// Generates a [name bind msg](MsgBindNameRequest) that will properly assign the given name value
/// to a target address.  Assumes the parent name is unrestricted or that the contract has access to
/// bind a name to the parent name.
///
/// # Parameters
/// * `name` The dot-qualified name to use on-chain for name binding. Ex: myname.sc.pb will generate
/// a msg that binds "myname" to the existing parent name "sc.pb".
/// * `bind_to_address` The bech32 address to which the name will be bound.
/// * `restricted` If true, the name will be bound as a restricted name, preventing future name
/// bindings from using it as a parent name.
pub fn msg_bind_name<S1: Into<String>, S2: Into<String>>(
    name: S1,
    bind_to_address: S2,
    restricted: bool,
) -> Result<MsgBindNameRequest, ContractError> {
    let fully_qualified_name = name.into();
    let mut name_parts = fully_qualified_name.split('.').collect::<Vec<&str>>();
    let bind_address = bind_to_address.into();
    let bind_record = if let Some(bind) = name_parts.to_owned().first() {
        if bind.is_empty() {
            return ContractError::InvalidFormatError {
                message: format!(
                    "cannot bind to an empty name string [{}]",
                    fully_qualified_name
                ),
            }
            .to_err();
        }
        Some(NameRecord {
            name: bind.to_string(),
            address: bind_address.to_owned(),
            restricted,
        })
    } else {
        return ContractError::InvalidFormatError {
            message: format!(
                "cannot derive bind name from input [{}]",
                fully_qualified_name
            ),
        }
        .to_err();
    };
    let parent_record = if name_parts.len() > 1 {
        // Trim the first element, because that is the new name to be bound
        name_parts.remove(0);
        let parent_name = name_parts.join(".").to_string();
        Some(NameRecord {
            name: parent_name.to_owned(),
            // The parent record must also use the address being bound to as its address in order for
            // the bind to succeed.  This is the only way in which Provenance accepts a non-restricted
            // name bind
            address: bind_address,
            restricted: false,
        })
    } else {
        None
    };
    MsgBindNameRequest {
        record: bind_record,
        parent: parent_record,
    }
    .to_ok()
}

/// Ensures that the target account has all the specified attributes.  Does not check for valid
/// attribute body contents.
///
/// # Parameters
/// * `deps` A dependencies object provided by the cosmwasm framework.  Allows access to useful
/// resources like contract internal storage and a querier to retrieve blockchain objects.
/// * `account` The bech32 address for which to pull and verify attributes.
/// * `attributes` All attribute names to verify.
pub fn check_account_has_all_attributes<S: Into<String>>(
    deps: &DepsMut,
    account: S,
    attributes: &[String],
) -> Result<(), ContractError> {
    if attributes.is_empty() {
        return ().to_ok();
    }
    let querier = AttributeQuerier::new(&deps.querier);
    let account_addr = account.into();
    let mut latest_response = querier.attributes(account_addr.to_owned(), None)?;
    let mut remaining_attributes = attributes.to_vec();
    while !remaining_attributes.is_empty() {
        for attr in latest_response.attributes.iter() {
            remaining_attributes.retain(|name| name != &attr.name);
        }
        if !remaining_attributes.is_empty() {
            if latest_response.pagination.is_some()
                && !latest_response
                    .pagination
                    .clone()
                    .unwrap()
                    .next_key
                    .clone()
                    .unwrap()
                    .is_empty()
            {
                latest_response = querier.attributes(
                    account_addr.to_owned(),
                    Some(PageRequest {
                        key: latest_response
                            .pagination
                            .unwrap()
                            .next_key
                            .clone()
                            .unwrap()
                            .to_owned(),
                        offset: 0,
                        limit: 25,
                        count_total: false,
                        reverse: false,
                    }),
                )?;
            } else {
                return ContractError::InvalidAccountError {
                    message: "account does not have all required attributes".to_string(),
                }
                .to_err();
            }
        }
    }
    ().to_ok()
}

/// Ensures that the target account holds enough of the target denom name by verifying their
/// balances in the bank module.
///
/// # Parameters
/// * `deps` A dependencies object provided by the cosmwasm framework.  Allows access to useful
/// resources like contract internal storage and a querier to retrieve blockchain objects.
/// * `account` The bech32 address of the account for which to verify balances.
/// * `denom` The coin denomination for which balances are to be checked.
/// * `required_amount` The minimum amount of coin that the target account must hold for the given
/// denom to be considered valid.
pub fn check_account_has_enough_denom<S1: Into<String>, S2: Into<String>>(
    deps: &Deps,
    account: S1,
    denom: S2,
    required_amount: u128,
) -> Result<(), ContractError> {
    let querier = BankQuerier::new(&deps.querier);
    let account_address = account.into();
    let target_denom = denom.into();
    let balance_response = querier.balance(account_address.to_owned(), target_denom.to_owned())?;
    if let Some(coin) = balance_response.balance {
        let numeric_balance = coin.amount.parse::<u128>()?;
        if numeric_balance < required_amount {
            ContractError::InvalidAccountError {
                message: format!(
                    "required [{required_amount}], but account only holds [{numeric_balance}]"
                ),
            }
            .to_err()
        } else {
            ().to_ok()
        }
    } else {
        ContractError::InvalidFundsError {
            message: format!("account [{account_address}] has no [{target_denom}] balance"),
        }
        .to_err()
    }
}

/// Fetches the bech32 address associated with the marker account for the given denomination.
///
/// # Parameters
/// * `deps` A dependencies object provided by the cosmwasm framework.  Allows access to useful
/// resources like contract internal storage and a querier to retrieve blockchain objects.
/// * `denom` The on-chain name for the marker denom.
pub fn get_marker_address_for_denom<S: Into<String>>(
    deps: &Deps,
    denom: S,
) -> Result<String, ContractError> {
    let marker_denom = denom.into();
    let querier = MarkerQuerier::new(&deps.querier);
    let marker_response = querier.marker(marker_denom.to_owned())?;
    if let Some(marker_account_any) = marker_response.marker {
        if let Ok(marker_account) = MarkerAccount::try_from(marker_account_any) {
            if let Some(base_account) = marker_account.base_account {
                base_account.address.to_ok()
            } else {
                ContractError::NotFoundError {
                    message: format!(
                        "unable to resolve base account from marker account [{}]",
                        &marker_denom
                    ),
                }
                .to_err()
            }
        } else {
            ContractError::NotFoundError {
                message: format!("unable to resolve marker account for denom [{marker_denom}]"),
            }
            .to_err()
        }
    } else {
        ContractError::NotFoundError {
            message: format!("unable to query marker by name [{}]", &marker_denom),
        }
        .to_err()
    }
}

#[cfg(test)]
mod tests {
    use crate::types::error::ContractError;
    use crate::util::provenance_utils::{
        check_account_has_all_attributes, check_account_has_enough_denom,
        get_marker_address_for_denom, msg_bind_name,
    };
    use prost::Message;
    use provwasm_mocks::{mock_provenance_dependencies_with_custom_querier, MockProvenanceQuerier};
    use provwasm_std::shim::Any;
    use provwasm_std::types::cosmos::auth::v1beta1::BaseAccount;
    use provwasm_std::types::cosmos::bank::v1beta1::{QueryBalanceRequest, QueryBalanceResponse};
    use provwasm_std::types::cosmos::base::query::v1beta1::PageResponse;
    use provwasm_std::types::cosmos::base::v1beta1::Coin;
    use provwasm_std::types::provenance::attribute::v1::{
        Attribute, AttributeType, QueryAttributesRequest, QueryAttributesResponse,
    };
    use provwasm_std::types::provenance::marker::v1::{
        MarkerAccount, MarkerStatus, MarkerType, QueryMarkerRequest, QueryMarkerResponse,
    };

    #[test]
    fn msg_bind_name_creates_proper_binding_with_fully_qualified_name() {
        let name = "test.name.bro";
        let address = "some-address";
        let msg =
            msg_bind_name(name, address, true).expect("valid input should not yield an error");
        let parent = msg.parent.expect("the result should include a parent msg");
        assert_eq!(
            "name.bro", parent.name,
            "parent name should be properly derived",
        );
        assert_eq!(
            address, parent.address,
            "parent address value should be set as the bind address because that's what enables binds to unrestricted parent addresses",
        );
        assert!(
            !parent.restricted,
            "parent restricted should always be false",
        );
        let bind = msg.record.expect("the result should include a name record");
        assert_eq!(
            "test", bind.name,
            "the bound name should be properly derived",
        );
        assert_eq!(
            address, bind.address,
            "the bound name should have the specified address",
        );
        assert!(
            bind.restricted,
            "the restricted value should equate to the value specified",
        );
    }

    #[test]
    fn msg_bind_name_creates_proper_binding_with_single_node_name() {
        let name = "name";
        let address = "address";
        let msg = msg_bind_name(name, address, false)
            .expect("proper input should produce a success result");
        assert!(
            msg.parent.is_none(),
            "the parent record should not be set because the name bind does not require it",
        );
        let bind = msg.record.expect("the result should include a name record");
        assert_eq!(
            "name", bind.name,
            "the bound name should be properly derived",
        );
        assert_eq!(
            address, bind.address,
            "the bound name should have the specified address",
        );
        assert!(
            !bind.restricted,
            "the restricted value should equate to the value specified",
        );
    }

    #[test]
    fn msg_bind_name_should_properly_guard_against_bad_input() {
        let _expected_error_message = "cannot derive bind name from input []".to_string();
        assert!(
            matches!(
                msg_bind_name("", "address", true)
                    .expect_err("an error should occur when no name is specified"),
                ContractError::InvalidFormatError {
                    message: _expected_error_message,
                },
            ),
            "unexpected error message when specifying an empty name",
        );
        let _expected_error_message = "cannot bind to an empty name string [.suffix]".to_string();
        assert!(
            matches!(
                msg_bind_name(".suffix", "address", true)
                    .expect_err("an error should occur when specifying a malformed name"),
                ContractError::InvalidFormatError {
                    message: _expected_error_message,
                },
            ),
            "unexpected error message when specifying a malformed name",
        );
    }

    #[test]
    fn check_account_has_all_attributes_should_succeed_when_attributes_present() {
        let mut querier = MockProvenanceQuerier::new(&[]);
        let account = "account".to_string();
        QueryAttributesRequest::mock_response(
            &mut querier,
            QueryAttributesResponse {
                account: account.to_owned(),
                attributes: vec![
                    Attribute {
                        name: "first".to_string(),
                        value: vec![],
                        attribute_type: AttributeType::String as i32,
                        address: "some-addr".to_string(),
                        expiration_date: None,
                    },
                    Attribute {
                        name: "second".to_string(),
                        value: vec![],
                        attribute_type: AttributeType::Json as i32,
                        address: "other-addr".to_string(),
                        expiration_date: None,
                    },
                ],
                pagination: Some(PageResponse {
                    next_key: Some(vec![]),
                    total: 2,
                }),
            },
        );
        let mut deps = mock_provenance_dependencies_with_custom_querier(querier);
        check_account_has_all_attributes(
            &deps.as_mut(),
            account,
            &["first".to_string(), "second".to_string()],
        )
        .expect("when all required attributes are in results, a success should occur");
    }

    #[test]
    fn check_account_has_all_attributes_should_fail_when_attributes_missing() {
        let mut querier = MockProvenanceQuerier::new(&[]);
        let account = "account".to_string();
        QueryAttributesRequest::mock_response(
            &mut querier,
            QueryAttributesResponse {
                account: account.to_owned(),
                attributes: vec![Attribute {
                    name: "wrong_attribute".to_string(),
                    value: vec![],
                    attribute_type: AttributeType::String as i32,
                    address: "some-addr".to_string(),
                    expiration_date: None,
                }],
                pagination: Some(PageResponse {
                    next_key: Some(vec![]),
                    total: 2,
                }),
            },
        );
        let mut deps = mock_provenance_dependencies_with_custom_querier(querier);
        let error = check_account_has_all_attributes(
            &deps.as_mut(),
            account,
            &["right_attribute".to_string()],
        )
        .expect_err("when one or more attributes is missing, an error should occur");
        let _expected_error_message = "account does not have all required attributes".to_string();
        assert!(
            matches!(
                error,
                ContractError::InvalidAccountError {
                    message: _expected_error_message,
                },
            ),
            "unexpected error occurred when account missing one or more attributes",
        );
    }

    #[test]
    fn check_account_has_enough_denom_thresholds_work_correctly() {
        let mut querier = MockProvenanceQuerier::new(&[]);
        QueryBalanceRequest::mock_response(
            &mut querier,
            QueryBalanceResponse {
                balance: Some(Coin {
                    amount: "300".to_string(),
                    denom: "denom".to_string(),
                }),
            },
        );
        let deps = mock_provenance_dependencies_with_custom_querier(querier);
        check_account_has_enough_denom(&deps.as_ref(), "account", "denom", 300)
            .expect("the exact amount required should cause a pass");
        check_account_has_enough_denom(&deps.as_ref(), "account", "denom", 299)
            .expect("having more than the amount required should cause a pass");
        let error = check_account_has_enough_denom(&deps.as_ref(), "account", "denom", 301)
            .expect_err("having less than the amount required should cause an error");
        let _expected_error_message = "required [301], but account only holds [300]".to_string();
        assert!(
            matches!(
                error,
                ContractError::InvalidAccountError {
                    message: _expected_error_message,
                },
            ),
            "unexpected error message emitted when too high amount required",
        );
    }

    #[test]
    fn check_account_has_enough_denom_no_balance_produces_error() {
        let mut querier = MockProvenanceQuerier::new(&[]);
        QueryBalanceRequest::mock_response(&mut querier, QueryBalanceResponse { balance: None });
        let deps = mock_provenance_dependencies_with_custom_querier(querier);
        let error = check_account_has_enough_denom(&deps.as_ref(), "account", "denom", 1)
            .expect_err("an error should occur if the response includes no balance");
        let _expected_error_message = "account [account] has no [denom] balance".to_string();
        assert!(
            matches!(
                error,
                ContractError::InvalidFundsError {
                    message: _expected_error_message,
                },
            ),
            "unexpected error message emitted when no balance found",
        );
    }

    #[test]
    fn get_marker_address_for_denom_guards_against_missing_marker() {
        let mut querier = MockProvenanceQuerier::new(&[]);
        QueryMarkerRequest::mock_response(&mut querier, QueryMarkerResponse { marker: None });
        let deps = mock_provenance_dependencies_with_custom_querier(querier);
        let error = get_marker_address_for_denom(&deps.as_ref(), "marker")
            .expect_err("an error should occur when the marker is not found");
        let _expected_message = "unable to query marker by name [marker]".to_string();
        assert!(
            matches!(
                error,
                ContractError::NotFoundError {
                    message: _expected_message
                },
            ),
            "unexpected error message emitted when marker missing",
        );
    }

    #[test]
    fn get_marker_address_for_denom_guards_against_incorrect_marker_account_type() {
        // TODO: Test circumstance where marker account is malformed.  Provwasm 2.0.0 does not
        // allow for serializing anything except MarkerAccount to Any, so it cannot be tested
    }

    #[test]
    fn get_marker_address_for_denom_guards_against_missing_base_account() {
        let mut querier = MockProvenanceQuerier::new(&[]);
        QueryMarkerRequest::mock_response(
            &mut querier,
            QueryMarkerResponse {
                marker: Some(Any {
                    type_url: "/provenance.marker.v1.MarkerAccount".to_string(),
                    value: MarkerAccount {
                        base_account: None,
                        manager: "some-manager".to_string(),
                        access_control: vec![],
                        status: MarkerStatus::Active as i32,
                        denom: "marker".to_string(),
                        supply: "100".to_string(),
                        marker_type: MarkerType::Restricted as i32,
                        supply_fixed: false,
                        allow_governance_control: false,
                        allow_forced_transfer: false,
                        required_attributes: vec![],
                    }
                    .encode_to_vec(),
                }),
            },
        );
        let deps = mock_provenance_dependencies_with_custom_querier(querier);
        let error = get_marker_address_for_denom(&deps.as_ref(), "marker")
            .expect_err("an error should occur when the marker is missing a base account");
        let _expected_message =
            "unable to resolve base account from marker account [marker]".to_string();
        assert!(
            matches!(
                error,
                ContractError::NotFoundError {
                    message: _expected_message
                },
            ),
            "unexpected error message emitted when marker account data is invalid",
        );
    }

    #[test]
    fn get_marker_address_for_denom_should_succeed_with_a_proper_response() {
        let mut querier = MockProvenanceQuerier::new(&[]);
        QueryMarkerRequest::mock_response(
            &mut querier,
            QueryMarkerResponse {
                marker: Some(Any {
                    type_url: "/provenance.marker.v1.MarkerAccount".to_string(),
                    value: MarkerAccount {
                        base_account: Some(BaseAccount {
                            address: "marker-address".to_string(),
                            pub_key: None,
                            account_number: 312,
                            sequence: 68,
                        }),
                        manager: "some-manager".to_string(),
                        access_control: vec![],
                        status: MarkerStatus::Active as i32,
                        denom: "marker".to_string(),
                        supply: "100".to_string(),
                        marker_type: MarkerType::Restricted as i32,
                        supply_fixed: false,
                        allow_governance_control: false,
                        allow_forced_transfer: false,
                        required_attributes: vec![],
                    }
                    .encode_to_vec(),
                }),
            },
        );
        let deps = mock_provenance_dependencies_with_custom_querier(querier);
        let marker_address = get_marker_address_for_denom(&deps.as_ref(), "marker")
            .expect("a response should be emitted when marker output is properly formed");
        assert_eq!(
            "marker-address", marker_address,
            "the correct marker address should be extracted",
        );
    }
}
