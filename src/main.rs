use axum::{
    Router,
    routing::get
};
use anyhow::anyhow;
use shuttle_secrets::SecretStore;
use std::sync::Arc;
use publish_events::get_publish_events;
use deploy_events::get_deploy_events;

mod publish_events;
mod deploy_events;
mod error;

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
    let router = Router::new()
        .route("/get_publish", get(get_publish_events))
        .route("/get_deploy", get(get_deploy_events))
        .with_state(state);

    Ok(router.into())
}
