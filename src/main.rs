use axum::routing::{Router, get};
use anyhow::anyhow;
use shuttle_secrets::SecretStore;
use std::sync::Arc;
use publish_events::get_publish_events;
use deploy_events::get_deploy_events;
use subscribe_ledger_expiration::subscribe_contract_expiration;

mod publish_events;
mod deploy_events;
mod subscribe_ledger_expiration;
mod error;

#[derive(Clone)]
struct AppState {
    my_jwt_token: String,
    mercury_backend_endpoint: String,
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
    let Some(mercury_backend_endpoint) = secret_store.get("MERCURY_BACKEND_ENDPOINT") else {
        return Err(anyhow!("MERCURY_BACKEND_ENDPOINT not set in Secrets.toml file").into());
    };
    let Some(mercury_graphql_endpoint) = secret_store.get("MERCURY_GRAPHQL_ENDPOINT") else {
        return Err(anyhow!("MERCURY_GRAPHQL_ENDPOINT not set in Secrets.toml file").into());
    };

    // Create the AppState
    let state = Arc::new(AppState {
        my_jwt_token,
        mercury_backend_endpoint,
        mercury_graphql_endpoint,
    });

    // Create the routes of the API
    let router = Router::new()
        .route("/get_publish", get(get_publish_events))
        .route("/get_deploy", get(get_deploy_events))
        .route("/subscribe_contract_expiration/:id", get(subscribe_contract_expiration))
        .with_state(state);

    Ok(router.into())
}
