use axum::routing::{Router, get};
use anyhow::anyhow;
use shuttle_secrets::SecretStore;
use tower_http::cors::CorsLayer;
use http::{Method, HeaderValue};
use std::sync::{Arc, Mutex};
use events::get_publish::get_publish_events;
use events::get_deploy::get_deploy_events;
use expiration::subscribe_ledger_expiration::subscribe_contract_expiration;

mod events {
    pub mod get_deploy;
    pub mod get_publish;
}
mod expiration {
    mod extend_ttl;
    pub mod subscribe_ledger_expiration;
}
mod error;
mod update_token;

#[derive(Clone)]
struct AppState {
    mercury_jwt_token: Arc<Mutex<String>>,
    mercury_backend_endpoint: String,
    mercury_graphql_endpoint: String,
    mercury_id: String,
    mercury_pwd: String,
    source_account: String,
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_axum::ShuttleAxum {

    // Retrieve the secret variables
    let Some(mercury_backend_endpoint) = secret_store.get("MERCURY_BACKEND_ENDPOINT") else {
        return Err(anyhow!("MERCURY_BACKEND_ENDPOINT not set in Secrets.toml file").into());
    };
    let Some(mercury_graphql_endpoint) = secret_store.get("MERCURY_GRAPHQL_ENDPOINT") else {
        return Err(anyhow!("MERCURY_GRAPHQL_ENDPOINT not set in Secrets.toml file").into());
    };
    let Some(mercury_id) = secret_store.get("MERCURY_EMAIL") else {
        return Err(anyhow!("MERCURY_EMAIL not set in Secrets.toml file").into());
    };
    let Some(mercury_pwd) = secret_store.get("MERCURY_PASSWORD") else {
        return Err(anyhow!("MERCURY_PASSWORD not set in Secrets.toml file").into());
    };
    let Some(source_account) = secret_store.get("SOURCE_ACCOUNT") else {
        return Err(anyhow!("SOURCE_ACCOUNT not set in Secrets.toml file").into());
    };

    // Create the AppState
    let state = Arc::new(AppState {
        mercury_jwt_token: Arc::new(Mutex::new("".to_string())),
        mercury_backend_endpoint,
        mercury_graphql_endpoint,
        mercury_id,
        mercury_pwd,
        source_account,
    });

    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap());

    update_token::renew_jwt_cron_job(state.clone()).await;

    // Create the routes of the API
    let router = Router::new()
        .route("/get_publish", get(get_publish_events)).layer(cors.clone())
        .route("/get_deploy", get(get_deploy_events)).layer(cors.clone())
        .route("/subscribe_contract_expiration/:id", get(subscribe_contract_expiration)).layer(cors)
        .with_state(state);

    Ok(router.into())
}
