use crate::AppState;
use axum::extract::{Path, State};
use std::sync::Arc;
use crate::error::MyError;
use soroban_cli::commands::{
    contract::Durability,
    network,
    {contract::extend, config},
};
use soroban_cli::{fee, key};
use super::read_ledger::read_ledger_ttl;

// Handler to bump a contract instance
// Need the contract id and the number of ledgers to extend
pub async fn bump_contract_instance(
    State(state): State<Arc<AppState>>,
    Path((id, ledgers_to_extend)): Path<(String, u32)>
) -> Result<String, MyError> {

    let network = network::Args {
        rpc_url: Some(state.rpc_url.clone()),
        network_passphrase: Some(state.network_passphrase.clone()),
        network: None,
    };

    extend::Cmd {
        ledgers_to_extend,
        key: key::Args {
            contract_id: Some(id.clone()),
            key: None,
            key_xdr: None,
            wasm: None,
            wasm_hash: None,
            durability: Durability::Persistent,
        },   
        ttl_ledger_only: false,
        config: config::Args {
            network,
            source_account: state.source_account.clone(),
            ..Default::default()
        },
        fee: fee::Args::default(),
    }
    .run()
    .await?;

    // Read ledger ttl to return the new ttl
    let (latest_ledger, live_until_ledger_seq) = read_ledger_ttl(
        id,
        state.rpc_url.clone(),
        state.network_passphrase.clone(),
        state.source_account.clone()
    )
    .await?;

    let new_ttl = live_until_ledger_seq - latest_ledger as u32;

    Ok(new_ttl.to_string())
}
