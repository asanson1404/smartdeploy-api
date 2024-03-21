use axum::routing::{Router, get, post};
use sqlx::PgPool;
use anyhow::anyhow;
use shuttle_secrets::SecretStore;
use tower_http::cors::CorsLayer;
use http::{Method, HeaderName, HeaderValue};
use std::sync::{Arc, Mutex};
use events::{
    get_publish::get_publish_events,
    get_deploy::get_deploy_events,
    get_claim::get_claim_events,
};
use expiration::{
    subscribe_ledger_expiration::subscribe_contract_expiration,
    query_ledger_expiration::get_contract_instance_expiration,
    read_ledger::read_ledger_ttl_handler,
    extend_ttl::bump_contract_instance,
};
use postgres::db_communication::{retrieve, add};

mod events {
    pub mod get_deploy;
    pub mod get_publish;
    pub mod get_claim;
}
mod expiration {
    pub mod extend_ttl;
    pub mod read_ledger;
    pub mod subscribe_ledger_expiration;
    pub mod query_ledger_expiration;
}
mod postgres {
    pub mod db_communication; 
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
    rpc_url: String,
    network_passphrase: String,
    source_account: String,
    ttl_pool: PgPool,
    front_end_url: String,
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    #[shuttle_shared_db::Postgres] ttl_pool: PgPool
) -> shuttle_axum::ShuttleAxum {

    /*
        POSTGRES_URL = postgres://postgres:postgres@localhost:19463/smartdeploy-api
        Migrate the database: sqlx migrate run --database-url $POSTGRES_URL
        Display migration information: sqlx migrate info --database-url $POSTGRES_URL
        Reset the database: sqlx database reset --database-url $POSTGRES_URL
     */

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
    let Some(rpc_url) = secret_store.get("RPC_URL") else {
        return Err(anyhow!("RPC_URL not set in Secrets.toml file").into());
    };
    let Some(network_passphrase) = secret_store.get("NETWORK_PASSPHRASE") else {
        return Err(anyhow!("NETWORK_PASSPHRASE not set in Secrets.toml file").into());
    };
    let Some(source_account) = secret_store.get("SOURCE_ACCOUNT") else {
        return Err(anyhow!("SOURCE_ACCOUNT not set in Secrets.toml file").into());
    };
    let Some(front_end_url) = secret_store.get("FRONT_END_URL") else {
        return Err(anyhow!("FRONT_END_URL not set in Secrets.toml file").into());
    };

    // Create the AppState
    let state = Arc::new(AppState {
        mercury_jwt_token: Arc::new(Mutex::new("".to_string())),
        mercury_backend_endpoint,
        mercury_graphql_endpoint,
        mercury_id,
        mercury_pwd,
        rpc_url,
        network_passphrase,
        source_account,
        ttl_pool,
        front_end_url,
    });

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(state.front_end_url.parse::<HeaderValue>().unwrap())
        .allow_headers([HeaderName::from_static("content-type")]);

    update_token::renew_jwt_cron_job(state.clone()).await;

    // Create the routes of the API
    let router = Router::new()
        .route("/get_publish", get(get_publish_events)).layer(cors.clone())
        .route("/get_deploy", get(get_deploy_events)).layer(cors.clone())
        .route("/get_claim", get(get_claim_events)).layer(cors.clone())
        .route("/subscribe_contract_expiration/:id", get(subscribe_contract_expiration)).layer(cors.clone())
        .route("/query_ledger_expiration/:encoded_hash_xdr", get(get_contract_instance_expiration)).layer(cors.clone())
        .route("/read_ledger_ttl/:id", get(read_ledger_ttl_handler)).layer(cors.clone())
        .route("/bump_contract_instance/:id/:ledgers_to_extend", get(bump_contract_instance)).layer(cors.clone())
        .route("/contracts_ttl", post(add)).layer(cors.clone())
        .route("/contracts_ttl", get(retrieve)).layer(cors)
        .with_state(state);

    Ok(router.into())
}
