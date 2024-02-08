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
pub struct PublishDataEvents(Vec<ScVal>);
// Integrate PublishDataEvents into axum response to use it as a return type in axum handlers
impl IntoResponse for PublishDataEvents {
    fn into_response(self) -> Response {
        Json(self.0).into_response()
    }
}

// Axum Handler to query and decode the publish events
pub async fn get_publish_events(State(state): State<Arc<AppState>>) -> Result<PublishDataEvents, MyError> {

    // Generate the event topic xdr
    let topic = ScVal::String(ScString(StringM::from_str("Publish").unwrap())).to_xdr_base64(Limits::none()).unwrap();

    // GraphQL request variables
    let variables = query_events::Variables {
        t1: topic,
    };

    // Build the GraphQL request body
    let request_body = QueryEvents::build_query(variables);

    // Post the GraphQL request
    let client = reqwest::Client::new();
    let res = client
            .post(format!("{}/graphql", state.mercury_graphql_endpoint.clone()))
            .bearer_auth(state.my_jwt_token.clone())
            .json(&request_body)
            .send()
            .await?;

    if res.status().is_success() {

        tracing::debug!("GET PUBLISH EVENTS REQUEST SUCCEEDED");

        // Deserialize the response body as Json
        let response_body: GraphQLResponse<query_events::ResponseData> = res.json().await?;

        // Retrieve in a Vec all the publish events
        let publish_events = response_body
            .data.ok_or_else(|| MyError::EmptyData)?
            .event_by_topic.ok_or_else(|| MyError::EmptyEventByTopic)?
            .nodes.ok_or_else(|| MyError::EmptyNodes)?;

        // Vec to store all the decoded data events
        let mut decoded_data_events = Vec::new();

        // Decode every events data from xdr to JSON and fill the Vec
        for publish_event_data in publish_events {
            // Retrieve data event (XDR form)
            let xdr = publish_event_data
                .unwrap()
                .data
                .unwrap();
            // Decode the XDR and fill the Vec
            let decoded_val = ScVal::from_xdr_base64(xdr.as_bytes(), Limits::none())
                .map_err(MyError::FromXdrError)?;
            decoded_data_events.push(decoded_val);
        }
        
        return Ok(PublishDataEvents(decoded_data_events));
        
    }

    Err(MyError::BadResponseStatus)

}