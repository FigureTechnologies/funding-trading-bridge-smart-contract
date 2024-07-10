use crate::store::contract_state::{get_contract_state_v1, CONTRACT_TYPE};
use crate::types::error::ContractError;
use crate::util::conversion_utils::convert_denom;
use crate::util::provenance_utils::{
    check_account_has_all_attributes, check_account_has_enough_denom,
};
use crate::util::validation_utils::check_funds_are_empty;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use provwasm_std::types::cosmos::base::v1beta1::Coin;
use provwasm_std::types::provenance::marker::v1::{
    MsgMintRequest, MsgTransferRequest, MsgWithdrawRequest,
};
use result_extensions::ResultExtensions;

/// Invoked via the contract's execute functionality.  The function will attempt to pull [trade_amount](fund_trading#trade_amount)
/// of the deposit marker's denom from the sender's account with a marker transfer, discern how much
/// of the trading denom to which the submitted amount is equivalent, and then mint and withdraw
/// that equivalent amount into the sender's account.
///
/// # Parameters
/// * `deps` A dependencies object provided by the cosmwasm framework.  Allows access to useful
/// resources like contract internal storage and a querier to retrieve blockchain objects.
/// * `env` An environment object provided by the cosmwasm framework.  Describes the contract's
/// details, as well as blockchain information at the time of the transaction.
/// * `info` A message information object provided by the cosmwasm framework.  Describes the sender
/// of the instantiation message, as well as the funds provided as an amount during the transaction.
/// * `trade_amount` The amount of the deposit marker to pull from the sender's account in exchange
/// for trading denom.
pub fn fund_trading(
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
        &contract_state.required_deposit_attributes,
    )?;
    let conversion = convert_denom(
        trade_amount,
        &contract_state.deposit_marker,
        &contract_state.trading_marker,
    )?;
    if conversion.target_amount == 0 {
        return ContractError::InvalidFundsError {
            message: format!(
                "sent [{}{}], but that is not enough to convert to at least one [{}]",
                trade_amount,
                &contract_state.deposit_marker.name,
                &contract_state.trading_marker.name,
            ),
        }
        .to_err();
    }
    // Transfer the necessary amount from the sender (total amount requested - remainder that cannot be converted)
    let transferred_amount = trade_amount - conversion.remainder;
    check_account_has_enough_denom(
        &deps.as_ref(),
        info.sender.as_str(),
        &contract_state.deposit_marker.name,
        transferred_amount,
    )?;
    let transfer_msg = MsgTransferRequest {
        administrator: env.contract.address.to_string(),
        amount: Some(Coin {
            denom: contract_state.deposit_marker.name.to_owned(),
            amount: transferred_amount.to_string(),
        }),
        from_address: info.sender.to_string(),
        to_address: env.contract.address.to_string(),
    };
    // Mint the amount of coin to which the conversion equates
    let minted_coin = Coin {
        denom: contract_state.trading_marker.name.to_owned(),
        amount: conversion.target_amount.to_string(),
    };
    let mint_msg = MsgMintRequest {
        administrator: env.contract.address.to_string(),
        amount: Some(minted_coin.to_owned()),
    };
    // Withdraw the newly-minted coin to the sender, effectively making the trade
    let withdraw_msg = MsgWithdrawRequest {
        denom: contract_state.trading_marker.name.to_owned(),
        administrator: env.contract.address.to_string(),
        to_address: info.sender.to_string(),
        amount: vec![minted_coin.to_owned()],
    };
    Response::new()
        .add_message(transfer_msg)
        .add_message(mint_msg)
        .add_message(withdraw_msg)
        .add_attribute("action", "fund_trading")
        .add_attribute("contract_address", env.contract.address.to_string())
        .add_attribute("contract_type", CONTRACT_TYPE)
        .add_attribute("contract_name", &contract_state.contract_name)
        .add_attribute("deposit_input_denom", &contract_state.deposit_marker.name)
        .add_attribute("deposit_requested_amount", trade_amount.to_string())
        .add_attribute("deposit_actual_amount", transferred_amount.to_string())
        .add_attribute("received_denom", minted_coin.denom)
        .add_attribute("received_amount", minted_coin.amount)
        .to_ok()
}

#[cfg(test)]
mod tests {
    use crate::execute::fund_trading::fund_trading;
    use crate::store::contract_state::CONTRACT_TYPE;
    use crate::test::attribute_extractor::AttributeExtractor;
    use crate::test::test_constants::{
        DEFAULT_CONTRACT_NAME, DEFAULT_DEPOSIT_DENOM_NAME, DEFAULT_REQUIRED_DEPOSIT_ATTRIBUTE,
        DEFAULT_TRADING_DENOM_NAME,
    };
    use crate::test::test_instantiate::{test_instantiate, test_instantiate_with_msg};
    use crate::types::denom::Denom;
    use crate::types::error::ContractError;
    use crate::types::msg::InstantiateMsg;
    use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{coins, CosmosMsg};
    use provwasm_mocks::{
        mock_provenance_dependencies, mock_provenance_dependencies_with_custom_querier,
        MockProvenanceQuerier,
    };
    use provwasm_std::types::cosmos::bank::v1beta1::{QueryBalanceRequest, QueryBalanceResponse};
    use provwasm_std::types::cosmos::base::v1beta1::Coin;
    use provwasm_std::types::provenance::attribute::v1::{
        Attribute, AttributeType, QueryAttributesRequest, QueryAttributesResponse,
    };
    use provwasm_std::types::provenance::marker::v1::{
        MsgMintRequest, MsgTransferRequest, MsgWithdrawRequest,
    };

    #[test]
    fn provided_funds_should_cause_an_error() {
        let mut deps = mock_provenance_dependencies();
        let error = fund_trading(
            deps.as_mut(),
            mock_env(),
            mock_info("some-sender", &coins(10, "nhash")),
            10,
        )
        .expect_err("an error should be emitted when coin is provided");
        assert!(
            matches!(error, ContractError::InvalidFundsError { .. },),
            "unexpected error type encountered when providing funds",
        );
    }

    #[test]
    fn missing_contract_state_should_cause_an_error() {
        let mut deps = mock_provenance_dependencies();
        let error = fund_trading(deps.as_mut(), mock_env(), mock_info("some-sender", &[]), 10)
            .expect_err("an error should be emitted when no contract state exists");
        assert!(
            matches!(error, ContractError::StorageError { .. },),
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
                    amount: "9".to_string(),
                    denom: DEFAULT_DEPOSIT_DENOM_NAME.to_string(),
                }),
            },
        );
        QueryAttributesRequest::mock_response(
            &mut querier,
            QueryAttributesResponse {
                account: "sender".to_string(),
                attributes: vec![Attribute {
                    name: DEFAULT_REQUIRED_DEPOSIT_ATTRIBUTE.to_string(),
                    value: vec![],
                    attribute_type: AttributeType::String as i32,
                    address: "addr".to_string(),
                }],
                pagination: None,
            },
        );
        let mut deps = mock_provenance_dependencies_with_custom_querier(querier);
        test_instantiate(deps.as_mut());
        let error = fund_trading(deps.as_mut(), mock_env(), mock_info("some-sender", &[]), 10)
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
                    denom: DEFAULT_DEPOSIT_DENOM_NAME.to_string(),
                }),
            },
        );
        QueryAttributesRequest::mock_response(
            &mut querier,
            QueryAttributesResponse {
                account: "some-sender".to_string(),
                attributes: vec![],
                pagination: None,
            },
        );
        let mut deps = mock_provenance_dependencies_with_custom_querier(querier);
        test_instantiate(deps.as_mut());
        let error = fund_trading(deps.as_mut(), mock_env(), mock_info("some-sender", &[]), 10)
            .expect_err("an error should occur when the sender does not have a required attribute");
        assert!(
            matches!(error, ContractError::InvalidAccountError { .. },),
            "unexpected error when account is missing required attributes",
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
                    denom: DEFAULT_DEPOSIT_DENOM_NAME.to_string(),
                }),
            },
        );
        QueryAttributesRequest::mock_response(
            &mut querier,
            QueryAttributesResponse {
                account: "sender".to_string(),
                attributes: vec![Attribute {
                    name: DEFAULT_REQUIRED_DEPOSIT_ATTRIBUTE.to_string(),
                    value: vec![],
                    attribute_type: AttributeType::String as i32,
                    address: "addr".to_string(),
                }],
                pagination: None,
            },
        );
        let mut deps = mock_provenance_dependencies_with_custom_querier(querier);
        // Setup trading marker to have a smaller precision than deposit, which will cause a single
        // digit conversion to fail
        test_instantiate_with_msg(
            deps.as_mut(),
            InstantiateMsg {
                deposit_marker: Denom::new("denom1", 2),
                trading_marker: Denom::new("denom2", 1),
                ..InstantiateMsg::default()
            },
        );
        let error = fund_trading(deps.as_mut(), mock_env(), mock_info("sender", &[]), 9)
            .expect_err("a conversion that does not produce any trading denom should fail");
        let _expected_err =
            "sent [9denom1], but that is not enough to convert to at least one [denom2]"
                .to_string();
        assert!(
            matches!(
                error,
                ContractError::InvalidFundsError {
                    message: _expected_err,
                },
            ),
            "unexpected error occurred when invalid conversion occurs",
        );
    }

    #[test]
    fn successful_parameters_should_produce_a_result() {
        let mut querier = MockProvenanceQuerier::new(&[]);
        QueryBalanceRequest::mock_response(
            &mut querier,
            QueryBalanceResponse {
                balance: Some(Coin {
                    amount: "103".to_string(),
                    denom: DEFAULT_DEPOSIT_DENOM_NAME.to_string(),
                }),
            },
        );
        QueryAttributesRequest::mock_response(
            &mut querier,
            QueryAttributesResponse {
                account: "sender".to_string(),
                attributes: vec![Attribute {
                    name: DEFAULT_REQUIRED_DEPOSIT_ATTRIBUTE.to_string(),
                    value: vec![],
                    attribute_type: AttributeType::String as i32,
                    address: "addr".to_string(),
                }],
                pagination: None,
            },
        );
        let mut deps = mock_provenance_dependencies_with_custom_querier(querier);
        // Setup the trading marker to have a smaller precision than the deposit, requiring some
        // remainder to be returned.  Ex:
        // Sender wants to send 103, which equates to 1.03.  However, trading marker has a precision
        // of 1, which will convert to 10 (aka 1.0).  The 3 will be dropped and be a remaining value
        // for the sender
        test_instantiate_with_msg(
            deps.as_mut(),
            InstantiateMsg {
                deposit_marker: Denom::new(DEFAULT_DEPOSIT_DENOM_NAME, 2),
                trading_marker: Denom::new(DEFAULT_TRADING_DENOM_NAME, 1),
                ..InstantiateMsg::default()
            },
        );
        let response = fund_trading(deps.as_mut(), mock_env(), mock_info("sender", &[]), 103)
            .expect("proper circumstances should derive a successful result");
        assert_eq!(
            3,
            response.messages.len(),
            "expected the response to include three messages",
        );
        response.messages.iter().for_each(|msg| match &msg.msg {
            CosmosMsg::Stargate { type_url, value } => match type_url.as_str() {
                "/provenance.marker.v1.MsgTransferRequest" => {
                    let req = MsgTransferRequest::try_from(value.to_owned())
                        .expect("the value should properly deserialize to a transfer request");
                    assert_eq!(
                        MOCK_CONTRACT_ADDR,
                        req.administrator,
                        "the contract address should be set as the administrator of the transfer request",
                    );
                    let coin = req.amount.expect("expected the amount to be set on the transfer request");
                    assert_eq!(
                        100.to_string(),
                        coin.amount,
                        "the correct amount of funds should be taken from the sender",
                    );
                    assert_eq!(
                        DEFAULT_DEPOSIT_DENOM_NAME,
                        coin.denom,
                        "the correct denom should be taken from the sender",
                    );
                    assert_eq!(
                        "sender",
                        req.from_address,
                        "the sender should be the from_address",
                    );
                    assert_eq!(
                        MOCK_CONTRACT_ADDR,
                        req.to_address,
                        "the contract should be the to_address",
                    );
                }
                "/provenance.marker.v1.MsgMintRequest" => {
                    let req = MsgMintRequest::try_from(value.to_owned())
                        .expect("the value should properly deserialize to a mint request");
                    assert_eq!(
                        MOCK_CONTRACT_ADDR,
                        req.administrator,
                        "the administrator of the mint msg should be the contract",
                    );
                    let coin = req.amount.expect("expected the amount to be set on the mint request");
                    assert_eq!(
                        10.to_string(),
                        coin.amount,
                        "the amount minted should equate to the amount after the precision conversion",
                    );
                    assert_eq!(
                        DEFAULT_TRADING_DENOM_NAME,
                        coin.denom,
                        "the denom minted should be the trading denom",
                    );
                }
                "/provenance.marker.v1.MsgWithdrawRequest" => {
                    let req = MsgWithdrawRequest::try_from(value.to_owned())
                        .expect("expected the msg to be a withdraw request");
                    assert_eq!(
                        DEFAULT_TRADING_DENOM_NAME,
                        req.denom,
                        "the withdraw request should withdraw from the trading marker",
                    );
                    assert_eq!(
                        MOCK_CONTRACT_ADDR,
                        req.administrator,
                        "the withdraw request should use the contract address as the administrator",
                    );
                    assert_eq!(
                        "sender",
                        req.to_address,
                        "the withdraw request should send the coin to the sender",
                    );
                    assert_eq!(
                        1,
                        req.amount.len(),
                        "the amount field should have a single coin",
                    );
                    let coin = req.amount.first().unwrap();
                    assert_eq!(
                        10.to_string(),
                        coin.amount,
                        "the withdrawn amount should be the upconverted denom",
                    );
                    assert_eq!(
                        DEFAULT_TRADING_DENOM_NAME,
                        coin.denom,
                        "the withdrawn denom should be the trading denom",
                    );
                }
                url => panic!("unexpected type url in emitted msg: {url}"),
            },
            msg => panic!("unexpected message emitted: {msg:?}"),
        });
        assert_eq!(
            9,
            response.attributes.len(),
            "expected nine attributes to be emitted",
        );
        response.assert_attribute("action", "fund_trading");
        response.assert_attribute("contract_address", MOCK_CONTRACT_ADDR);
        response.assert_attribute("contract_type", CONTRACT_TYPE);
        response.assert_attribute("contract_name", DEFAULT_CONTRACT_NAME);
        response.assert_attribute("deposit_input_denom", DEFAULT_DEPOSIT_DENOM_NAME);
        response.assert_attribute("deposit_requested_amount", "103");
        response.assert_attribute("deposit_actual_amount", "100");
        response.assert_attribute("received_denom", DEFAULT_TRADING_DENOM_NAME);
        response.assert_attribute("received_amount", "10");
    }

    #[test]
    fn request_that_does_not_need_full_amount_expected_succeeds() {
        let mut querier = MockProvenanceQuerier::new(&[]);
        QueryBalanceRequest::mock_response(
            &mut querier,
            QueryBalanceResponse {
                balance: Some(Coin {
                    amount: "200".to_string(),
                    denom: DEFAULT_DEPOSIT_DENOM_NAME.to_string(),
                }),
            },
        );
        QueryAttributesRequest::mock_response(
            &mut querier,
            QueryAttributesResponse {
                account: "sender".to_string(),
                attributes: vec![Attribute {
                    name: DEFAULT_REQUIRED_DEPOSIT_ATTRIBUTE.to_string(),
                    value: vec![],
                    attribute_type: AttributeType::String as i32,
                    address: "addr".to_string(),
                }],
                pagination: None,
            },
        );
        let mut deps = mock_provenance_dependencies_with_custom_querier(querier);
        // Setup the trading marker to have a smaller precision than the deposit, requiring some
        // remainder to be returned.  Ex:
        // Sender wants to send 250, which equates to 2.50.  They don't actually have 250, but they
        // do have 200, which is allowed.  This should be allowed to proceed.
        test_instantiate_with_msg(
            deps.as_mut(),
            InstantiateMsg {
                deposit_marker: Denom::new(DEFAULT_DEPOSIT_DENOM_NAME, 3),
                trading_marker: Denom::new(DEFAULT_TRADING_DENOM_NAME, 1),
                ..InstantiateMsg::default()
            },
        );
        fund_trading(deps.as_mut(), mock_env(), mock_info("sender", &[]), 250)
            .expect("proper circumstances should derive a successful result");
    }
}
