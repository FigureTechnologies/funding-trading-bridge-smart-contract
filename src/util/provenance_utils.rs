use crate::types::error::ContractError;
use cosmwasm_std::DepsMut;
use provwasm_std::types::cosmos::base::query::v1beta1::PageRequest;
use provwasm_std::types::provenance::attribute::v1::AttributeQuerier;
use result_extensions::ResultExtensions;

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
        if !remaining_attributes.is_empty()
            && latest_response.pagination.is_some()
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
    ().to_ok()
}
