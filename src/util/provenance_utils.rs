use crate::types::error::ContractError;
use cosmwasm_std::DepsMut;
use provwasm_std::types::cosmos::bank::v1beta1::BankQuerier;
use provwasm_std::types::cosmos::base::query::v1beta1::PageRequest;
use provwasm_std::types::provenance::attribute::v1::AttributeQuerier;
use provwasm_std::types::provenance::marker::v1::{MarkerAccount, MarkerQuerier};
use provwasm_std::types::provenance::name::v1::{MsgBindNameRequest, NameRecord};
use result_extensions::ResultExtensions;

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
            if remaining_attributes.contains(&attr.name) {
                remaining_attributes.retain(|name| name != &attr.name);
            }
        }
        if !remaining_attributes.is_empty() {
            if latest_response.pagination.is_some()
                && !latest_response
                    .pagination
                    .clone()
                    .unwrap()
                    .next_key
                    .is_empty()
            {
                latest_response = querier.attributes(
                    account_addr.to_owned(),
                    Some(PageRequest {
                        key: latest_response.pagination.unwrap().next_key.to_owned(),
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

pub fn check_account_has_enough_denom<S1: Into<String>, S2: Into<String>>(
    deps: &DepsMut,
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

pub fn get_marker_address_for_denom<S: Into<String>>(
    deps: &DepsMut,
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
    use crate::util::provenance_utils::msg_bind_name;

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
}
