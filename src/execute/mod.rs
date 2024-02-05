//! Contains all execution routes used by the [contract file](crate::contract).

/// This execution route allows the contract admin to choose a new admin.
pub mod admin_update_admin;
/// This execution route allows the contract admin to choose new attributes required when invoking
/// [fund_trading].
pub mod admin_update_deposit_required_attributes;
/// This execution route allows the contract admin to choose new attributes required when invoking
/// [withdraw_trading].
pub mod admin_update_withdraw_required_attributes;
/// This execution route converts the [deposit marker](crate::types::msg::InstantiateMsg#deposit_marker)
/// denom to the [trading marker](crate::types::msg::InstantiateMsg#trading_marker) denom by transferring
/// the deposit marker denom from the sender to the contract, and then minting and withdrawing new
/// trading marker denom to the sender's account.
pub mod fund_trading;
/// This execution route converts the [trading marker](crate::types::msg::InstantiateMsg#trading_marker)
/// denom to the [deposit marker](crate::types::msg::InstantiateMsg#deposit_marker) denom by transferring
/// the trading marker denom from the sender to the trading marker itself, burning the received values,
/// and then returning deposit marker denom to the sender's account.
pub mod withdraw_trading;
