use crate::AppState;
use crate::error::MyError;
use std::sync::Arc;
use axum::{
    extract::{State, Path},
    Json
};
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
use sha2::{Sha256, Digest};
use serde_json::json;
use super::read_ledger::read_ledger_ttl;

// Axum Handler for subscribing to contract expiration
pub async fn subscribe_contract_expiration(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>
) -> Result<Json<serde_json::Value>, MyError> {
    
    // Build the LedgerKey knowing the contract id
    let contract = stellar_strkey::Contract::from_string(id.as_str())
        .map_err(|e| MyError::StringToContractConversionFailed(id.clone(), e))?;

    let ledger_key = LedgerKey::ContractData(
        LedgerKeyContractData {
            contract: ScAddress::Contract(Hash(contract.0)),
            key: ScVal::LedgerKeyContractInstance,
            durability: ContractDataDurability::Persistent,
        }
    );

    // Build the LedgerKey XDR (/!\ not base64)
    let ledger_key_xdr = ledger_key
        .to_xdr(Limits::none())
        .map_err(MyError::ToXdrError)?;

    // Hash the above XDR
    let hashed_ledger_key_xdr = Hash(Sha256::digest(ledger_key_xdr).into());

    // Build the XDR of the LedgerKey Hashed
    let hash_xdr = hashed_ledger_key_xdr
        .to_xdr_base64(Limits::none())
        .map_err(MyError::ToXdrError)?;
    tracing::debug!("Xdr of LedgerKey Hashed is : {:#?}", hash_xdr);

    // Create the request body
    let request_body = json!({
        "hash_xdr": hash_xdr
    });

    let client = reqwest::Client::new();
    let res = client.post(format!("{}/expiration", state.mercury_backend_endpoint))
        .bearer_auth(state.mercury_jwt_token.lock().unwrap())
        .json(&request_body)
        .send()
        .await?;
    tracing::debug!("Mercury subscription response: {:#?}", res);

    if res.status().is_success() {

        let (current_ledger, ledger_ttl) = read_ledger_ttl(
            id.clone(),
            state.rpc_url.clone(),
            state.network_passphrase.clone(),
            state.source_account.clone()
        ).await?;

        tracing::debug!("SUCCESSFULLY SUBSCRIBE TO CONTRACT EXPIRATION TRACKING: {}", id);
        
        let ret_val = json!({
            "status": "CONTRACT_INSTANCE_SUBSCRIBED",
            "contract_id": id,
            "hash_xdr": hash_xdr,
            "current_ledger": current_ledger,
            "ledger_ttl": ledger_ttl
        });

        return Ok(Json(ret_val));
    }

    Err(MyError::BadResponseStatus)

}