use axum::{
    Router,
    routing::get,
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use graphql_client::{ GraphQLQuery, Response as GraphQLResponse };
use anyhow::anyhow;
use shuttle_secrets::SecretStore;
use std::sync::Arc;

enum MyError {
    ReqwestError(reqwest::Error),
    BadResponseStatus,
}

// Convert reqwest::Error towards MyError::ReqwestError
impl From<reqwest::Error> for MyError {
    fn from(error: reqwest::Error) -> Self {
        MyError::ReqwestError(error)
    }
}

// Integrate MyError into axum response to use it as a return type in axum handlers 
impl IntoResponse for MyError {
    fn into_response(self) -> Response {
        let body = match self {
            MyError::ReqwestError(reqwest_error) => format!("Reqwest Error: {}", reqwest_error),
            MyError::BadResponseStatus => "Bad Response Status: response status not included between 200 and 300 (excluded).\nResponse not sent".to_string(),
        };

        body.into_response()
    }
}

#[derive(Clone)]
struct AppState {
    my_jwt_token: String,
    mercury_graphql_endpoint: String,
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_axum::ShuttleAxum {

    // Retrieve the secret variables
    let Some(my_jwt_token) = secret_store.get("MY_JWT_TOKEN") else {
        return Err(anyhow!("MY_JWT_TOKEN not set in Secrets.toml file").into());
    };
    let Some(mercury_graphql_endpoint) = secret_store.get("MERCURY_GRAPHQL_ENDPOINT") else {
        return Err(anyhow!("MERCURY_GRAPHQL_ENDPOINT not set in Secrets.toml file").into());
    };

    // Create the AppState
    let state = Arc::new(AppState {
        my_jwt_token,
        mercury_graphql_endpoint,
    });

    // Create a route to query deploy events
    let router = Router::new().route("/get_deploy", get(get_deploy_event)).with_state(state);

    Ok(router.into())
}

// Generate a module named deploy_events
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/schema.graphql",
    query_path  = "queries/query_deploy.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct DeployEvents;

// Integrate deploy_events::ResponseData into axum response to use it as a return type in axum handlers
impl IntoResponse for deploy_events::ResponseData {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

// Axum Handler to query the deploy events
async fn get_deploy_event(State(state): State<Arc<AppState>>) -> Result<deploy_events::ResponseData, MyError> {

    // Build the GraphQL request body (with empty Variable field) 
    let request_body = DeployEvents::build_query(deploy_events::Variables {});

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
        let response_body: GraphQLResponse<deploy_events::ResponseData> = res.json().await?;
        let response_data: deploy_events::ResponseData = response_body.data.unwrap();
        return Ok(response_data);
    }

    Err(MyError::BadResponseStatus)

}