use crate::AppState;
use crate::error::MyError;
use std::sync::Arc;
use axum::extract::{State, Path};
use stellar_xdr::curr as stellar_xdr;
use stellar_xdr::{
    ScAddress,
    ScVal,
    LedgerKey,
    LedgerKeyContractData,
    ContractDataDurability,
    WriteXdr,
    Limits,
    Hash,
};
use soroban_sdk::{Address, Env};
use sha2::{Sha256, Digest};
use serde_json::json;

// Axum Handler for subscribing to contract expiration
pub async fn subscribe_contract_expiration(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>
) -> Result<(), MyError> {
    
    // Build the LedgerKey knowing the contract id
    let soroban_string = soroban_sdk::String::from_str(&Env::default(), id.as_str());
    let address = Address::from_string(&soroban_string); // CAN PANICK!!!!
    let sc_address = ScAddress::try_from(address.clone())
        .map_err(|e| MyError::ScAddressConversionFailed(address, e))?;
    
    let ledger_key = LedgerKey::ContractData(
        LedgerKeyContractData {
            contract: sc_address,
            key: ScVal::LedgerKeyContractInstance,
            durability: ContractDataDurability::Persistent,
        }
    );
    tracing::debug!("LedgerKey is : {:#?}", ledger_key);

    let ledger_key_xdr = ledger_key
        .to_xdr_base64(Limits::none())
        .map_err(MyError::ToXdrError)?;
    tracing::debug!("LedgerKey XDR is : {:#?}", ledger_key_xdr);

    let hashed_ledger_key_xdr = Hash(Sha256::digest(ledger_key_xdr).into());
    tracing::debug!("LedgerKey Hashed is : {:#?}", hashed_ledger_key_xdr);
    
    let hash_xdr = hashed_ledger_key_xdr
        .to_xdr_base64(Limits::none())
        .map_err(MyError::ToXdrError)?;
    tracing::debug!("Xdr of LedgerKey Hashed is : {:#?}", hash_xdr);

    // Create the request body
    let request_body = json!({
        "hash": hash_xdr
    });

    // Send the subscription request to Mercury
    tokio::task::spawn_blocking(move || {
        let client = reqwest::blocking::Client::new();
        let res = client.post(format!("{}/expiration", state.mercury_backend_endpoint.clone()))
            .bearer_auth(state.my_jwt_token.clone())
            .json(&request_body)
            .send().unwrap();
        tracing::debug!("Mercury subscription response: {:#?}", res);
    });

    Ok(())
}