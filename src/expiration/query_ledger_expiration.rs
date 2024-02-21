use crate::AppState;
use crate::error::MyError;
use std::sync::Arc;
use axum::{
    extract::{State, Path},
    Json
};
use graphql_client::{GraphQLQuery, Response as GraphQLResponse};

// Generate a module named ledger_instance_expiration
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/ledger_expiration/schema.graphql",
    query_path  = "queries/ledger_expiration/query.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct LedgerInstanceExpiration;

// Axum Handler to query ledger expiration
pub async fn contract_instance_expiration(
    State(state): State<Arc<AppState>>,
    Path(encoded_hash_xdr): Path<String>
) -> Result<Json<ledger_instance_expiration::LedgerInstanceExpirationEntryExpirationByHashXdr>, MyError> {

    let res;

    /* Scope to drop the mutex right after the query */
    {
        // GraphQL request variables
        let variables = ledger_instance_expiration::Variables {
            hash_xdr: encoded_hash_xdr,
        };

        // Build the GraphQL request body
        let request_body = LedgerInstanceExpiration::build_query(variables);
        
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

        tracing::debug!("GET LEDGER EXPIRATION REQUEST SUCCEEDED");

        // Deserialize the response body as Json
        let response_body: GraphQLResponse<ledger_instance_expiration::ResponseData> = res.json().await?;

        // Retrieve in a Vec all the deploy events
        let expiration_data = response_body
            .data.ok_or_else(|| MyError::EmptyData)?
            .entry_expiration_by_hash_xdr.ok_or_else(|| MyError::EmptyEntryExpiration)?;
        
        return Ok(Json(expiration_data));

    }

    Err(MyError::BadResponseStatus)
}