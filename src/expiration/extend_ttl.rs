use crate::error::MyError;
use soroban_cli::commands::contract::Durability;
use soroban_cli::commands::network;
use soroban_cli::commands::{contract::extend, config};
use soroban_cli::{fee, key};

pub async fn bump_instance(contract_id: String, ledgers_to_extend: u32, source_account: String,) -> Result<(), MyError> {

    let network = network::Args {
        rpc_url: None,
        network_passphrase: None,
        network: Some("testnet".to_owned()),
    };

    extend::Cmd {
        ledgers_to_extend,
        key: key::Args {
            contract_id: Some(contract_id),
            key: None,
            key_xdr: None,
            wasm: None,
            wasm_hash: None,
            durability: Durability::Persistent,
        },   
        ttl_ledger_only: false,
        config: config::Args {
            network,
            source_account,
            ..Default::default()
        },
        fee: fee::Args::default(),
    }
    .run()
    .await?;

    Ok(())
}

/*
pub struct MyExtendCmd {
    pub extend_cmd: extend::Cmd,
}

impl MyExtendCmd {
    pub async fn run(&self) -> Result<(), MyError> {
        self.extend_cmd.clone().run().await?;
        Ok(())
    }

    pub fn new(key_xdr: String, contract_id: String, ledgers_to_extend: u32) -> Self {
        let network = network::Args {
            rpc_url: None,
            network_passphrase: None,
            network: Some("testnet".to_owned()),
        };
        let extend_cmd = extend::Cmd {
            ledgers_to_extend,
            key: key::Args {
                contract_id: Some(contract_id),
                key: None,
                key_xdr: Some(vec![key_xdr.clone()]),
                wasm: None,
                wasm_hash: None,
                durability: Durability::Persistent,
            },   
            ttl_ledger_only: false,
            config: config::Args {
                network,
                source_account: "GD2DGTQWRWGEX4K5TFPVPGD6SXRSYTXTFBB2QOU3E4WVBYM7PINJKVVD".to_owned(),
                ..Default::default()
            },
            fee: fee::Args::default(),
        };
        Self {
            extend_cmd,
        }
    }
}
*/