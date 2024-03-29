use crate::AppState;
use crate::error::MyError;
use std::{str::FromStr, sync::Arc};
use axum::{ 
    extract::State, 
    response::{IntoResponse, Response},
    Json 
};
use graphql_client::{GraphQLQuery, Response as GraphQLResponse};
use stellar_xdr::curr as stellar_xdr;
use stellar_xdr::{
    {ScString, StringM},
    {ReadXdr, WriteXdr},
    Limits,
    ScVal,
};

// Generate a module named query_events
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/events/schema.graphql",
    query_path  = "queries/events/query.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct QueryEvents;

// Struct for the Axum Handler Response
pub struct DeployDataEvents(Vec<ScVal>);
// Integrate DeployDataEvents into axum response to use it as a return type in axum handlers
impl IntoResponse for DeployDataEvents {
    fn into_response(self) -> Response {
        Json(self.0).into_response()
    }
}

// Axum Handler to query and decode the deploy events
pub async fn get_deploy_events(State(state): State<Arc<AppState>>) -> Result<DeployDataEvents, MyError> {

    let res;

    /* Scope to drop the mutex right after the query */
    {
        // Generate the event topic xdr
        let topic = ScVal::String(ScString(StringM::from_str("Deploy").unwrap())).to_xdr_base64(Limits::none()).unwrap();

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

        tracing::debug!("GET PUBLISH EVENTS REQUEST SUCCEEDED");

        // Deserialize the response body as Json
        let response_body: GraphQLResponse<query_events::ResponseData> = res.json().await?;

        // Retrieve in a Vec all the deploy events
        let deploy_events = response_body
            .data.ok_or_else(|| MyError::EmptyData)?
            .event_by_topic.ok_or_else(|| MyError::EmptyEventByTopic)?
            .nodes.ok_or_else(|| MyError::EmptyNodes)?;

        // Vec to store all the decoded data events
        let mut decoded_data_events = Vec::new();

        // Decode every events data from xdr to JSON and fill the Vec
        for deploy_event_data in deploy_events {
            // Retrieve data event (XDR form)
            let xdr = deploy_event_data
                .unwrap()
                .data
                .unwrap();
            // Decode the XDR and fill the Vec
            let decoded_val = ScVal::from_xdr_base64(xdr.as_bytes(), Limits::none())
                .map_err(MyError::FromXdrError)?;
            decoded_data_events.push(decoded_val);
        }
        
        return Ok(DeployDataEvents(decoded_data_events));
        
    }

    Err(MyError::BadResponseStatus)

}