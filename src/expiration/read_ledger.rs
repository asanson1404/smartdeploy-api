use crate::AppState;
use crate::error::MyError;
use std::sync::Arc;
use axum::extract::{State, Path};
use axum::Json;
use soroban_cli::commands::{
    network,
    config,
    contract::Durability,
};
use soroban_cli::key;
use soroban_cli::rpc::Client;

pub async fn read_ledger_ttl(
    contract_id: String,
    rpc_url: String,
    network_passphrase: String,
    source_account: String,
) -> Result<(i64, u32), MyError> {

    let config = config::Args {
        network: network::Args {
            rpc_url: Some(rpc_url),
            network_passphrase: Some(network_passphrase),
            network: None,
        },
        source_account,
        ..Default::default()
    };

    let network = config
        .get_network()
        .map_err(MyError::ConfigNetworkError)?;

    let key = key::Args {
        contract_id: Some(contract_id),
        key: None,
        key_xdr: None,
        wasm: None,
        wasm_hash: None,
        durability: Durability::Persistent,
    }.parse_keys()?;

    let client = Client::new(&network.rpc_url)?;

    let full_ledger_entries = client.get_full_ledger_entries(&key).await?;
    let latest_ledger = full_ledger_entries.latest_ledger;

    let live_until_ledger_seq = full_ledger_entries.entries[0].live_until_ledger_seq;

    Ok((latest_ledger, live_until_ledger_seq))

}

pub async fn read_ledger_ttl_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>
) -> Result<Json<(i64, u32)>, MyError> {

    let config = config::Args {
        network: network::Args {
            rpc_url: Some(state.rpc_url.clone()),
            network_passphrase: Some(state.network_passphrase.clone()),
            network: None,
        },
        source_account: state.source_account.clone(),
        ..Default::default()
    };

    let network = config
        .get_network()
        .map_err(MyError::ConfigNetworkError)?;

    let key = key::Args {
        contract_id: Some(id),
        key: None,
        key_xdr: None,
        wasm: None,
        wasm_hash: None,
        durability: Durability::Persistent,
    }.parse_keys()?;

    let client = Client::new(&network.rpc_url)?;

    let full_ledger_entries = client.get_full_ledger_entries(&key).await?;
    let latest_ledger = full_ledger_entries.latest_ledger;

    let live_until_ledger_seq = full_ledger_entries.entries[0].live_until_ledger_seq;

    Ok(Json((latest_ledger, live_until_ledger_seq)))

}
