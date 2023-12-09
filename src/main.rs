use anyhow::anyhow;
use axum::{
    Router,
    routing::get,
    extract::State,
    response::{IntoResponse, Response},
};
use shuttle_secrets::SecretStore;
use std::sync::Arc;

enum MyError {
    ReqwestError(reqwest::Error),
    OtherError,
}

// Convert reqwest::Error towards MyError::ReqwestError
impl From<reqwest::Error> for MyError {
    fn from(error: reqwest::Error) -> Self {
        MyError::ReqwestError(error)
    }
}

impl IntoResponse for MyError {
    fn into_response(self) -> Response {
        let body = match self {
            MyError::ReqwestError(reqwest_error) => format!("Reqwest Error: {}", reqwest_error),
            MyError::OtherError => "Other Error to Implement (fs for example)".to_string(),
        };

        body.into_response()
    }
}

#[derive(Clone)]
struct AppState {
    my_jwt_token: String,
    mercury_backend_endpoint: String,
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_axum::ShuttleAxum {

    // Retrieve the secret variables
    let Some(my_jwt_token) = secret_store.get("MY_JWT_TOKEN") else {
        return Err(anyhow!("MY_JWT_TOKEN not set in Secrets.toml file").into());
    };
    let Some(mercury_backend_endpoint) = secret_store.get("MERCURY_BACKEND_ENDPOINT") else {
        return Err(anyhow!("MERCURY_BACKEND_ENDPOINT not set in Secrets.toml file").into());
    };

    // Create the AppState
    let state = Arc::new(AppState {
        my_jwt_token,
        mercury_backend_endpoint,
    });

    
    let router = Router::new().route("/get_deploy", get(get_deploy_event)).with_state(state);

    Ok(router.into())
}


async fn get_deploy_event(State(state): State<Arc<AppState>>) -> String {
  
    state.mercury_backend_endpoint.clone()
    
}