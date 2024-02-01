use crate::AppState;
use crate::error::MyError;
use std::sync::Arc;
use axum::{ 
    extract::State, 
    response::{ IntoResponse, Response },
    Json 
};
use graphql_client::{ GraphQLQuery, Response as GraphQLResponse };
use stellar_xdr::curr as stellar_xdr;
use stellar_xdr::{
    ReadXdr,
    Limits,
    ScVal,
};

// Generate a module named publish_events
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/schema.graphql",
    query_path  = "queries/query_publish.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct PublishEvents;

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

    // Build the GraphQL request body (with empty Variable field) 
    let request_body = PublishEvents::build_query(publish_events::Variables {});

    // Post the GraphQL request
    let client = reqwest::Client::new();
    let res = client
            .post(format!("{}/graphql", state.mercury_graphql_endpoint.clone()))
            .bearer_auth(state.my_jwt_token.clone())
            .json(&request_body)
            .send()
            .await?;

    tracing::debug!("Response Status: {}", res.status());

    if res.status().is_success() {

        tracing::debug!("Success");

        // Deserialize the response body as Json
        let response_body: GraphQLResponse<publish_events::ResponseData> = res.json().await?;

        // Retrieve in a Vec all the publish events/ edges
        let publish_events = response_body
            .data.ok_or_else(|| MyError::EmptyResponseData)?
            .event_by_topic.ok_or_else(|| MyError::EmptyEventByTopic)?
            .edges.ok_or_else(|| MyError::EmptyEdges)?;

        // Vec to store all the decoded data events
        let mut decoded_data_events = Vec::new();

        // Decode every events data from xdr to JSON and fill the Vec
        for publish_event in publish_events {

            // Retrieve data event (XDR form)
            let xdr = publish_event
                .ok_or_else(|| MyError::EmptyEventEdge)?
                .node.ok_or_else(|| MyError::EmptyEventNode)?
                .data.ok_or_else(|| MyError::EmptyEventXdrData)?;

            // Decode the XDR and fill the Vec
            let bytes = xdr.as_bytes();
            let decoded_val = ScVal::from_xdr_base64(bytes, Limits::none())
                .map_err(MyError::FromXdrError)?;
            tracing::debug!("Decoded Data is : {:#?}", decoded_val);
            decoded_data_events.push(decoded_val);

        }
        
        return Ok(PublishDataEvents(decoded_data_events));

        // Decode a Symbol
        //let bytes = "AAAADwAAAAZkZXBsb3kAAA==".as_bytes();
        //let decoded_val = ScVal::from_xdr_base64(bytes, Limits::none());
        //let json_val = Json(decoded_val);
        //tracing::debug!("Decoded Symbol is : {:?}", json_val);

    }

    Err(MyError::BadResponseStatus)

}