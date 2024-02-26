use crate::error::MyError;
use soroban_cli::commands::{
    network,
    config,
    contract::Durability,
};
use soroban_cli::key;
use soroban_cli::rpc::Client;

pub async fn read_ledger_ttl(contract_id: String, source_account: String,) -> Result<(i64, u32), MyError> {

    let config = config::Args {
        network: network::Args {
            rpc_url: None,
            network_passphrase: None,
            network: Some("testnet".to_owned()),
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
