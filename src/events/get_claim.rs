use crate::AppState;
use crate::error::MyError;
use std::{str::FromStr, sync::Arc};
use axum::{ 
    extract::State, 
    response::{IntoResponse, Response},
    Json 
};
use graphql_client::{GraphQLQuery, Response as GraphQLResponse};
use ::stellar_xdr::curr::ScAddress;
use stellar_xdr::curr as stellar_xdr;
use stellar_xdr::{
    {ScString, StringM, ScVal, ScSymbol, ScMap, ScContractInstance},
    {ReadXdr, WriteXdr},
    Hash,
    ContractDataEntry, ContractExecutable,
    Limits
};
use soroban_cli::commands::{
    network,
    config,
};
use soroban_cli::rpc::Client;

// Generate a module named query_events
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/events/schema.graphql",
    query_path  = "queries/events/query.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct QueryEvents;

// Struct for the Axum Handler Response
pub struct ClaimDataEvents(Vec<(ScVal, String)>);
// Integrate ClaimDataEvents into axum response to use it as a return type in axum handlers
impl IntoResponse for ClaimDataEvents {
    fn into_response(self) -> Response {
        Json(self.0).into_response()
    }
}

// Axum Handler to query and decode the claim events
pub async fn get_claim_events(State(state): State<Arc<AppState>>) -> Result<ClaimDataEvents, MyError> {

    let res;

    /* Scope to drop the mutex right after the query */
    {
        // Generate the event topic xdr
        let topic = ScVal::String(ScString(StringM::from_str("Claim").unwrap())).to_xdr_base64(Limits::none()).unwrap();

        // GraphQL request variables
        let variables = query_events::Variables {
            t1: topic,
        };

        // Build the GraphQL request body
        let request_body = QueryEvents::build_query(variables);
        
        // Post the GraphQL request
        let client = reqwest::Client::new();
        res = client
                .post(format!("{}/graphql", state.mercury_graphql_endpoint))
                .bearer_auth(state.mercury_jwt_token.lock().unwrap())
                .json(&request_body)
                .send()
                .await?;
    }

    if res.status().is_success() {

        tracing::debug!("GET CLAIM EVENTS REQUEST SUCCEEDED");

        // Deserialize the response body as Json
        let response_body: GraphQLResponse<query_events::ResponseData> = res.json().await?;

        // Retrieve in a Vec all the claim events
        let claim_events = response_body
            .data.ok_or_else(|| MyError::EmptyData)?
            .event_by_topic.ok_or_else(|| MyError::EmptyEventByTopic)?
            .nodes.ok_or_else(|| MyError::EmptyNodes)?;

        // Return data: Vec with the decoded data events and the contract's wasm hash
        let mut return_data = Vec::<(ScVal, String)>::new();

        // Decode every events data from xdr to JSON and fill the Vec
        for claim_event_data in claim_events {
            // Retrieve data event (XDR form)
            let xdr = claim_event_data
                .unwrap()
                .data
                .unwrap();
            // Decode the XDR and fill the Vec
            let decoded_val = ScVal::from_xdr_base64(xdr.as_bytes(), Limits::none())
                .map_err(MyError::FromXdrError)?;

            // Access the address field in the contract_id object
            let Hash(contract_id) = extract_contract_id(&decoded_val)?;
            // Retrieve the wasm hash
            let wasm_hash = get_wasm_hash(contract_id, state.rpc_url.clone(), state.network_passphrase.clone(), state.source_account.clone()).await?;
            return_data.push((decoded_val, wasm_hash));
        }
        
        return Ok(ClaimDataEvents(return_data));
        
    }

    Err(MyError::BadResponseStatus)

}

// Extract the contract_id from the decoded xdr
fn extract_contract_id(sc_val: &ScVal) -> Result<&Hash, MyError> {

    match sc_val {
        ScVal::Map(Some(ScMap(vec_m))) => {
            let key = ScVal::Symbol(ScSymbol(StringM::from_str("contract_id").unwrap()));
            let entries = vec_m.iter().find(|entry| entry.key == key).unwrap();
            match &entries.val {
                ScVal::Address(ScAddress::Contract(hash)) => {
                    Ok(hash)
                },
                _ => {
                    Err(MyError::InvalidClaimEventData)
                }
            }
        },
        _ => {
            Err(MyError::InvalidClaimEventData)
        }
    }

}

// Retrieve the wasm hash from the contract_id
async fn get_wasm_hash(
    id: &[u8; 32],
    rpc_url: String,
    network_passphrase: String,
    source_account: String,
) -> Result<String, MyError> {

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

    let client = Client::new(&network.rpc_url)?;

    match client.get_contract_data(id).await? {
        ContractDataEntry {
            val:
                ScVal::ContractInstance(ScContractInstance {
                    executable: ContractExecutable::Wasm(hash),
                    ..
                }),
            ..
        } => Ok(hash.to_string()),
        _ => Err(MyError::HashRetrievalFailed),
    }

}