use crate::store::contract_state::{get_contract_state_v1, CONTRACT_TYPE};
use crate::types::error::ContractError;
use crate::util::conversion_utils::convert_denom;
use crate::util::provenance_utils::{
    check_account_has_all_attributes, check_account_has_enough_denom, get_marker_address_for_denom,
};
use crate::util::validation_utils::check_funds_are_empty;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use provwasm_std::types::cosmos::base::v1beta1::Coin;
use provwasm_std::types::provenance::marker::v1::{MsgBurnRequest, MsgTransferRequest};
use result_extensions::ResultExtensions;

/// Invoked via the contract's execute functionality.  The function will attempt to pull [trade_amount](withdraw_trading#trade_amount)
/// of the trading marker's denom from the sender's account with a marker transfer, discern how much
/// of the deposit denom to which the submitted amount is equivalent, transfer that amount to the
/// sender, and then burn the exchanged trading marker denom.
///
/// # Parameters
/// * `deps` A dependencies object provided by the cosmwasm framework.  Allows access to useful
/// resources like contract internal storage and a querier to retrieve blockchain objects.
/// * `env` An environment object provided by the cosmwasm framework.  Describes the contract's
/// details, as well as blockchain information at the time of the transaction.
/// * `info` A message information object provided by the cosmwasm framework.  Describes the sender
/// of the instantiation message, as well as the funds provided as an amount during the transaction.
/// * `trade_amount` The amount of the trading marker to pull from the sender's account in exchange
/// for deposit denom.
pub fn withdraw_trading(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    trade_amount: u128,
) -> Result<Response, ContractError> {
    check_funds_are_empty(&info)?;
    let contract_state = get_contract_state_v1(deps.storage)?;
    check_account_has_all_attributes(
        &deps,
        &info.sender,
        &contract_state.required_withdraw_attributes,
    )?;
    let conversion = convert_denom(
        trade_amount,
        &contract_state.trading_marker,
        &contract_state.deposit_marker,
    )?;
    if conversion.target_amount == 0 {
        return ContractError::InvalidFundsError {
            message: format!(
                "sent [{}{}], but that is not enough to convert to at least one [{}]",
                trade_amount,
                &contract_state.trading_marker.name,
                &contract_state.deposit_marker.name,
            ),
        }
        .to_err();
    }
    let collected_amount = trade_amount - conversion.remainder;
    check_account_has_enough_denom(
        &deps.as_ref(),
        info.sender.as_str(),
        &contract_state.trading_marker.name,
        collected_amount,
    )?;
    // Collect the amount to be traded to the contract from the sender and give it directly to the
    // marker in order to stage it for burning
    let collect_funds_msg = MsgTransferRequest {
        administrator: env.contract.address.to_string(),
        amount: Some(Coin {
            denom: contract_state.trading_marker.name.to_owned(),
            amount: collected_amount.to_string(),
        }),
        from_address: info.sender.to_string(),
        to_address: get_marker_address_for_denom(
            &deps.as_ref(),
            &contract_state.trading_marker.name,
        )?,
    };
    // Release the total converted amount of funds back to the user
    let release_funds_msg = MsgTransferRequest {
        administrator: env.contract.address.to_string(),
        amount: Some(Coin {
            denom: contract_state.deposit_marker.name.to_owned(),
            amount: conversion.target_amount.to_string(),
        }),
        from_address: env.contract.address.to_string(),
        to_address: info.sender.to_string(),
    };
    // Burn all coins that were received except those that could not be converted, these will be
    // refunded
    let burn_msg = MsgBurnRequest {
        administrator: env.contract.address.to_string(),
        amount: Some(Coin {
            amount: collected_amount.to_string(),
            denom: contract_state.trading_marker.name.to_owned(),
        }),
    };
    Response::new()
        .add_message(collect_funds_msg)
        .add_message(release_funds_msg)
        .add_message(burn_msg)
        .add_attribute("action", "withdraw_trading")
        .add_attribute("contract_address", env.contract.address.to_string())
        .add_attribute("contract_type", CONTRACT_TYPE)
        .add_attribute("contract_name", &contract_state.contract_name)
        .add_attribute("withdraw_input_denom", &contract_state.trading_marker.name)
        .add_attribute("withdraw_input_amount", trade_amount.to_string())
        .add_attribute("withdraw_actual_amount", collected_amount.to_string())
        .add_attribute("received_denom", &contract_state.deposit_marker.name)
        .add_attribute("received_amount", conversion.target_amount.to_string())
        .to_ok()
}

#[cfg(test)]
mod tests {
    use crate::execute::withdraw_trading::withdraw_trading;
    use crate::store::contract_state::CONTRACT_TYPE;
    use crate::test::attribute_extractor::AttributeExtractor;
    use crate::test::test_constants::{
        DEFAULT_CONTRACT_NAME, DEFAULT_DEPOSIT_DENOM_NAME, DEFAULT_REQUIRED_WITHDRAW_ATTRIBUTE,
        DEFAULT_TRADING_DENOM_NAME,
    };
    use crate::test::test_instantiate::{test_instantiate, test_instantiate_with_msg};
    use crate::types::denom::Denom;
    use crate::types::error::ContractError;
    use crate::types::msg::InstantiateMsg;
    use cosmwasm_std::testing::{message_info, mock_env, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{coins, Addr, AnyMsg, CosmosMsg};
    use provwasm_mocks::{
        mock_provenance_dependencies, mock_provenance_dependencies_with_custom_querier,
        MockProvenanceQuerier,
    };
    use provwasm_std::shim::Any;
    use provwasm_std::types::cosmos::auth::v1beta1::BaseAccount;
    use provwasm_std::types::cosmos::bank::v1beta1::{QueryBalanceRequest, QueryBalanceResponse};
    use provwasm_std::types::cosmos::base::v1beta1::Coin;
    use provwasm_std::types::provenance::attribute::v1::{
        Attribute, AttributeType, QueryAttributesRequest, QueryAttributesResponse,
    };
    use provwasm_std::types::provenance::marker::v1::{
        MarkerAccount, MarkerStatus, MarkerType, MsgBurnRequest, MsgTransferRequest,
        QueryMarkerRequest, QueryMarkerResponse,
    };

    #[test]
    fn provided_funds_should_cause_an_error() {
        let mut deps = mock_provenance_dependencies();
        let error = withdraw_trading(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked("sender"), &coins(10, "somecoin")),
            10,
        )
        .expect_err("an error should be emitted when coin is provided");
        assert!(
            matches!(error, ContractError::InvalidFundsError { .. }),
            "unexpected error type encountered when providing funds",
        );
    }

    #[test]
    fn missing_contract_state_should_cause_an_error() {
        let mut deps = mock_provenance_dependencies();
        let error = withdraw_trading(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked("sender"), &[]),
            10,
        )
        .expect_err("an error should be emitted when no contract state exists");
        assert!(
            matches!(error, ContractError::StorageError { .. }),
            "unexpected error type encountered when no contract storage exists",
        );
    }

    #[test]
    fn sender_missing_required_amount_should_cause_an_error() {
        let mut querier = MockProvenanceQuerier::new(&[]);
        QueryBalanceRequest::mock_response(
            &mut querier,
            QueryBalanceResponse {
                balance: Some(Coin {
                    amount: "10".to_string(),
                    denom: DEFAULT_TRADING_DENOM_NAME.to_string(),
                }),
            },
        );
        QueryAttributesRequest::mock_response(
            &mut querier,
            QueryAttributesResponse {
                account: "sender".to_string(),
                attributes: vec![Attribute {
                    name: DEFAULT_REQUIRED_WITHDRAW_ATTRIBUTE.to_string(),
                    value: vec![],
                    attribute_type: AttributeType::Json as i32,
                    address: "addr".to_string(),
                    expiration_date: None,
                }],
                pagination: None,
            },
        );
        let mut deps = mock_provenance_dependencies_with_custom_querier(querier);
        test_instantiate(deps.as_mut());
        let error = withdraw_trading(deps.as_mut(), mock_env(), message_info(&Addr::unchecked("sender"), &[]), 10000)
            .expect_err("an error should occur when the sender tries to trade more funds than are available to them");
        assert!(
            matches!(error, ContractError::InvalidAccountError { .. }),
            "unexpected error type encountered when the sender tries to trade too much: {error:?}",
        );
    }

    #[test]
    fn sender_missing_required_attribute_should_cause_an_error() {
        let mut querier = MockProvenanceQuerier::new(&[]);
        QueryBalanceRequest::mock_response(
            &mut querier,
            QueryBalanceResponse {
                balance: Some(Coin {
                    amount: "10".to_string(),
                    denom: DEFAULT_TRADING_DENOM_NAME.to_string(),
                }),
            },
        );
        QueryAttributesRequest::mock_response(
            &mut querier,
            QueryAttributesResponse {
                account: "sender".to_string(),
                attributes: vec![],
                pagination: None,
            },
        );
        let mut deps = mock_provenance_dependencies_with_custom_querier(querier);
        test_instantiate(deps.as_mut());
        let error = withdraw_trading(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked("sender"), &[]),
            10,
        )
        .expect_err("an error should occur when the sender does not have a required attribute");
        assert!(
            matches!(error, ContractError::InvalidAccountError { .. }),
            "unexpected error when account is missing required attribute",
        );
    }

    #[test]
    fn conversion_producing_no_output_denom_should_cause_an_error() {
        let mut querier = MockProvenanceQuerier::new(&[]);
        QueryBalanceRequest::mock_response(
            &mut querier,
            QueryBalanceResponse {
                balance: Some(Coin {
                    amount: "10".to_string(),
                    denom: DEFAULT_TRADING_DENOM_NAME.to_string(),
                }),
            },
        );
        QueryAttributesRequest::mock_response(
            &mut querier,
            QueryAttributesResponse {
                account: "sender".to_string(),
                attributes: vec![Attribute {
                    name: DEFAULT_REQUIRED_WITHDRAW_ATTRIBUTE.to_string(),
                    value: vec![],
                    attribute_type: AttributeType::Json as i32,
                    address: "addr".to_string(),
                    expiration_date: None,
                }],
                pagination: None,
            },
        );
        let mut deps = mock_provenance_dependencies_with_custom_querier(querier);
        // Setup trading marker to have a higher precision than deposit, which will cause a single
        // digit conversion to fail with the input value 7:
        // Input 7 == 0.07, but trading marker can only hold values with one decimal place.
        test_instantiate_with_msg(
            deps.as_mut(),
            InstantiateMsg {
                deposit_marker: Denom::new("denom1", 1),
                trading_marker: Denom::new("denom2", 2),
                ..InstantiateMsg::default()
            },
        );
        let error = withdraw_trading(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked("sender"), &[]),
            7,
        )
        .expect_err("a conversion that does not produce any deposit denom should fail");
        let _expected_err =
            "sent [7denom2], but that is not enough to convert to at least one [denom1]"
                .to_string();
        assert!(
            matches!(
                error,
                ContractError::InvalidFundsError {
                    message: _expected_err,
                },
            ),
            "unexpected error when invalid conversion occurs",
        );
    }

    #[test]
    fn no_trading_marker_found_should_produce_an_error() {
        let mut querier = MockProvenanceQuerier::new(&[]);
        QueryBalanceRequest::mock_response(
            &mut querier,
            QueryBalanceResponse {
                balance: Some(Coin {
                    amount: "10".to_string(),
                    denom: DEFAULT_TRADING_DENOM_NAME.to_string(),
                }),
            },
        );
        QueryAttributesRequest::mock_response(
            &mut querier,
            QueryAttributesResponse {
                account: "sender".to_string(),
                attributes: vec![Attribute {
                    name: DEFAULT_REQUIRED_WITHDRAW_ATTRIBUTE.to_string(),
                    value: vec![],
                    attribute_type: AttributeType::Json as i32,
                    address: "addr".to_string(),
                    expiration_date: None,
                }],
                pagination: None,
            },
        );
        QueryMarkerRequest::mock_response(&mut querier, QueryMarkerResponse { marker: None });
        let mut deps = mock_provenance_dependencies_with_custom_querier(querier);
        test_instantiate_with_msg(
            deps.as_mut(),
            InstantiateMsg {
                deposit_marker: Denom::new("denom1", 2),
                trading_marker: Denom::new("denom2", 1),
                ..InstantiateMsg::default()
            },
        );
        let error = withdraw_trading(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked("sender"), &[]),
            1,
        )
        .expect_err("a missing trading marker should cause a failure");
        let _expected_err = "unable to query marker by name [denom2]".to_string();
        assert!(
            matches!(
                error,
                ContractError::NotFoundError {
                    message: _expected_err,
                },
            ),
            "unexpected error when trading marker missing",
        );
    }

    #[test]
    fn successful_parameters_should_produce_a_result() {
        let mut querier = MockProvenanceQuerier::new(&[]);
        QueryBalanceRequest::mock_response(
            &mut querier,
            QueryBalanceResponse {
                balance: Some(Coin {
                    amount: "4321".to_string(),
                    denom: DEFAULT_TRADING_DENOM_NAME.to_string(),
                }),
            },
        );
        QueryAttributesRequest::mock_response(
            &mut querier,
            QueryAttributesResponse {
                account: "sender".to_string(),
                attributes: vec![Attribute {
                    name: DEFAULT_REQUIRED_WITHDRAW_ATTRIBUTE.to_string(),
                    value: vec![],
                    attribute_type: AttributeType::Json as i32,
                    address: "addr".to_string(),
                    expiration_date: None,
                }],
                pagination: None,
            },
        );
        QueryMarkerRequest::mock_response(
            &mut querier,
            QueryMarkerResponse {
                marker: Some(Any {
                    type_url: "/provenance.marker.v1.MarkerAccount".to_string(),
                    value: MarkerAccount {
                        base_account: Some(BaseAccount {
                            address: "trading-marker-addr".to_string(),
                            pub_key: None,
                            account_number: 32,
                            sequence: 37,
                        }),
                        manager: "some-manager".to_string(),
                        access_control: vec![],
                        status: MarkerStatus::Active as i32,
                        denom: DEFAULT_TRADING_DENOM_NAME.to_string(),
                        supply: "10".to_string(),
                        marker_type: MarkerType::Restricted as i32,
                        supply_fixed: false,
                        allow_governance_control: false,
                        allow_forced_transfer: false,
                        required_attributes: vec![],
                    }
                    .to_proto_bytes(),
                }),
            },
        );
        let mut deps = mock_provenance_dependencies_with_custom_querier(querier);
        // Setup the trading marker to have a higher precision than the deposit, requiring some
        // remainder to be returned. Ex:
        // Sender wants to send 4321 trading to get their deposit denom back, which equates to 4.321.
        // However, the deposit marker has a precision of 2, which will convert to 4.32.  The 1 will
        // be dropped and be a remaining value for the sender.
        test_instantiate_with_msg(
            deps.as_mut(),
            InstantiateMsg {
                deposit_marker: Denom::new(DEFAULT_DEPOSIT_DENOM_NAME, 2),
                trading_marker: Denom::new(DEFAULT_TRADING_DENOM_NAME, 3),
                ..InstantiateMsg::default()
            },
        );
        let response = withdraw_trading(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked("sender"), &[]),
            4321,
        )
        .expect("proper circumstances should derive a successful result");
        assert_eq!(
            3,
            response.messages.len(),
            "expected the response to include three messages",
        );
        response.messages.iter().for_each(|msg| match &msg.msg {
            CosmosMsg::Any(AnyMsg { type_url, value }) => match type_url.as_str() {
                "/provenance.marker.v1.MsgTransferRequest" => {
                    let req = MsgTransferRequest::try_from(value.to_owned())
                        .expect("the transfer request msg should properly deserialize");
                    assert_eq!(
                        MOCK_CONTRACT_ADDR, req.administrator,
                        "the administrator should be the contract",
                    );
                    let amount = req
                        .amount
                        .expect("the transfer request should contain a coin amount");
                    match req.from_address.as_str() {
                        // Funds collection
                        "sender" => {
                            assert_eq!(
                                "4320", amount.amount,
                                "the fund collection should take all input funds except remainder",
                            );
                            assert_eq!(
                                DEFAULT_TRADING_DENOM_NAME, amount.denom,
                                "the fund collection should take the trading denom as input",
                            );
                            assert_eq!(
                                "trading-marker-addr", req.to_address,
                                "the fund collection should send funds back to the trading marker",
                            );
                        }
                        // Funds release
                        MOCK_CONTRACT_ADDR => {
                            assert_eq!(
                                "432", amount.amount,
                                "the fund release should return the properly converted deposit denom",
                            );
                            assert_eq!(
                                DEFAULT_DEPOSIT_DENOM_NAME, amount.denom,
                                "the fund release should return the deposit denom",
                            );
                            assert_eq!(
                                "sender", req.to_address,
                                "the fund release should send the funds back to the sender",
                            );
                        }
                        addr => panic!("transfer request included unexpected from_address: {addr}"),
                    }
                }
                "/provenance.marker.v1.MsgBurnRequest" => {
                    let req = MsgBurnRequest::try_from(value.to_owned())
                        .expect("the burn request msg should properly deserialize");
                    assert_eq!(
                        MOCK_CONTRACT_ADDR, req.administrator,
                        "the burn request should use the contract as the administrator",
                    );
                    let amount = req.amount.expect("the burn request should contain a coin amount");
                    assert_eq!(
                        "4320", amount.amount,
                        "the amount burned should be the amount of trading denom returned to the contract",
                    );
                    assert_eq!(
                        DEFAULT_TRADING_DENOM_NAME, amount.denom,
                        "the denom burned should be the trading denom",
                    );
                }
                url => panic!("unexpected type url in emitted msg: {url}"),
            },
            msg => panic!("unexpected message emitted: {msg:?}"),
        });
        assert_eq!(
            9,
            response.attributes.len(),
            "the response should emit nine attributes",
        );
        response.assert_attribute("action", "withdraw_trading");
        response.assert_attribute("contract_address", MOCK_CONTRACT_ADDR);
        response.assert_attribute("contract_type", CONTRACT_TYPE);
        response.assert_attribute("contract_name", DEFAULT_CONTRACT_NAME);
        response.assert_attribute("withdraw_input_denom", DEFAULT_TRADING_DENOM_NAME);
        response.assert_attribute("withdraw_input_amount", "4321");
        response.assert_attribute("withdraw_actual_amount", "4320");
        response.assert_attribute("received_denom", DEFAULT_DEPOSIT_DENOM_NAME);
        response.assert_attribute("received_amount", "432");
    }

    #[test]
    fn request_that_does_not_need_full_amount_expected_succeeds() {
        let mut querier = MockProvenanceQuerier::new(&[]);
        QueryBalanceRequest::mock_response(
            &mut querier,
            QueryBalanceResponse {
                balance: Some(Coin {
                    amount: "200".to_string(),
                    denom: DEFAULT_TRADING_DENOM_NAME.to_string(),
                }),
            },
        );
        QueryAttributesRequest::mock_response(
            &mut querier,
            QueryAttributesResponse {
                account: "sender".to_string(),
                attributes: vec![Attribute {
                    name: DEFAULT_REQUIRED_WITHDRAW_ATTRIBUTE.to_string(),
                    value: vec![],
                    attribute_type: AttributeType::Json as i32,
                    address: "addr".to_string(),
                    expiration_date: None,
                }],
                pagination: None,
            },
        );
        QueryMarkerRequest::mock_response(
            &mut querier,
            QueryMarkerResponse {
                marker: Some(Any {
                    type_url: "/provenance.marker.v1.MarkerAccount".to_string(),
                    value: MarkerAccount {
                        base_account: Some(BaseAccount {
                            address: "trading-marker-addr".to_string(),
                            pub_key: None,
                            account_number: 32,
                            sequence: 37,
                        }),
                        manager: "some-manager".to_string(),
                        access_control: vec![],
                        status: MarkerStatus::Active as i32,
                        denom: DEFAULT_TRADING_DENOM_NAME.to_string(),
                        supply: "10".to_string(),
                        marker_type: MarkerType::Restricted as i32,
                        supply_fixed: false,
                        allow_governance_control: false,
                        allow_forced_transfer: false,
                        required_attributes: vec![],
                    }
                    .to_proto_bytes(),
                }),
            },
        );
        let mut deps = mock_provenance_dependencies_with_custom_querier(querier);
        // Setup the trading marker to have a higher precision than the deposit, requiring some
        // remainder to be returned. Ex:
        // Sender wants to send 250, which equates to 2.50.  They don't actually have 250, but they
        // do have 200, which is allowed.  This should be allowed to proceed.
        test_instantiate_with_msg(
            deps.as_mut(),
            InstantiateMsg {
                deposit_marker: Denom::new(DEFAULT_DEPOSIT_DENOM_NAME, 1),
                trading_marker: Denom::new(DEFAULT_TRADING_DENOM_NAME, 3),
                ..InstantiateMsg::default()
            },
        );
        withdraw_trading(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked("sender"), &[]),
            250,
        )
        .expect("proper circumstances should derive a successful result");
    }
}
