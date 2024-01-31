use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use funding_trading_bridge_smart_contract::store::contract_state::ContractStateV1;
use funding_trading_bridge_smart_contract::types::msg::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};

fn main() {
    let mut out_dir = current_dir().expect("Could not fetch current directory");
    out_dir.push("schema");
    create_dir_all(&out_dir).expect("Could not create output directory");
    remove_schemas(&out_dir).expect("Could not remove existing schemas in output directory");
    // Top-level Msg values
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(MigrateMsg), &out_dir);
    // Query results
    export_schema(&schema_for!(ContractStateV1), &out_dir);
}
