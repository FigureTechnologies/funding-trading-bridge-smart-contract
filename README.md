# Funding Trading Bridge Smart Contract

The Funding Trading Bridge Smart Contract exists to convert funds from one denomination to another, allowing for each
denomination to dictate a precision by which its coin is valued.  It is designed with [CosmWasm](https://github.com/CosmWasm/cosmwasm)
and utilizes [ProvWasm](https://github.com/provenance-io/provwasm) to target the [Provenance Blockchain](https://provenance.io/)
for its deployment.

## Status
[![Latest Release][release-badge]][release-latest]
[![Apache 2.0 License][license-badge]][license-url]

[license-badge]: https://img.shields.io/github/license/FigureTechnologies/funding-trading-bridge-smart-contract.svg
[license-url]: https://github.com/FigureTechnologies/funding-trading-bridge-smart-contract/blob/main/LICENSE
[release-badge]: https://img.shields.io/github/tag/FigureTechnologies/funding-trading-bridge-smart-contract.svg
[release-latest]: https://github.com/FigureTechnologies/funding-trading-bridge-smart-contract/releases/latest

## Technical Overview and Setup

This contract is designed to facilitate movement of funds from two different restricted [Markers](https://developer.provenance.io/docs/pb/modules/marker-module):
- The first marker is the "deposit marker."  This marker should be a coin that is distributed from some external system.
This marker can be defined with the contract in the following ways:
  - If the marker is configured with the `allow_forced_transfer` option, the contract can simply be given the `ACCESS_TRANSFER`
permission for this marker.
  - If no forced transfers are allowed, the `required_attributes` feature of markers can be used to ensure that the contract
can move funds from the accounts involved with the marker.
  - An [Authz Grant](https://docs.cosmos.network/v0.46/modules/authz/03_messages.html#MsgGrant) can be leveraged to 
temporarily grant the contract access to transfer funds from the marker.
- The second marker is the "trading marker." This marker should be a coin that is primarily managed by the contract. To
ensure that the contract functions correctly, the contract must be given the following permissions to this marker:
  - `ACCESS_MINT` (to create new tokens when the deposit coin is added to the contract)
  - `ACCESS_WITHDRAW` (to move newly-minted tokens to accounts after they add their deposit coin)
  - `ACCESS_TRANSFER` (to transfer both the deposit and trading denoms from accounts as they interact with the contract)
  - `ACCESS_BURN` (to destroy any trading tokens that the contract receives from accounts).

Once the contract is instantiated and the marker permissions are properly configured, the contract will function to 
essentially convert the "deposit marker" denom to the "trading marker" denom.

## Instantiation

To instantiate the contract, use the standard [CosmWasm instantiation functionality](https://docs.cosmwasm.com/docs/getting-started/interact-with-contract/#instantiating-the-contract)
after fetching the `.wasm` file for the latest release from this repository's release section.  Use the json version of 
the [InstantiateMsg](src/types/msg.rs) struct; the file details the various fields and their descriptions.

## Execution Routes

The contract's various execution routes and their usages are as follows.  View the [Msg Definitions](src/types/msg.rs)
and inspect the `ExecuteMsg` struct to see their parameters and descriptions.

- `admin_update_admin`: This route allows the current admin of the contract, who is established at instantiation, to 
choose a new account address to be the admin. 
- `admin_update_deposit_required_attributes`: This route allows the contract admin to choose a new list of 
[Provenance Attributes](https://developer.provenance.io/docs/pb/modules/attribute-module/) that must appear on accounts
that invoke the `fund_trading` route.
- `admin_update_withdraw_required_attributes`: This route allows the contract admin to choose a new list of
[Provenance Attributes](https://developer.provenance.io/docs/pb/modules/attribute-module/) that must appear on accounts
that invoke the `withdraw_trading` route.
- `fund_trading`: This route allows an account possessing an amount of deposit denom to have its denom traded for an 
amount of trading denom.  It automatically converts the values to the proper precision and ensures that any values that
cannot fit into the trading denom's precision remain in the account.
- `withdraw_trading`: This route allows an account possessing an amount of trading denom received from the contract to
return it to the contract and receive its equivalent in the deposit denom.  It automatically converts the values to the
proper precision and ensures that any values that cannot fit into the trading denom's precision remain in the account.

## Query Routes

The contract's various query routes and their usages are as follows.  View the [Msg Definitions](src/types/msg.rs)
and inspect the `QueryMsg` struct to see their parameters and descriptions.

- `query_contract_state`: This route returns the internal contract state, which dictates the denoms specified by the 
contract, its name and version, as well as other metadata.
