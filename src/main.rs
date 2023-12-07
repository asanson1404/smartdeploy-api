use axum::{extract::State, routing::get, Router};
use shuttle_secrets::SecretStore;
use std::sync::Arc;

async fn print_jwt_token(State(state): State<Arc<AppState>>) -> String {
    state.jwt.clone()
}

#[derive(Clone)]
struct AppState {
    jwt: String,
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_axum::ShuttleAxum {

    let Some(secret) = secret_store.get("MY_JWT_TOKEN") else {
        panic!("API KEY NOT FOUND")
    };

    let state = Arc::new(AppState {
        jwt: secret,
    });

    
    let router = Router::new().route("/", get(print_jwt_token)).with_state(state);

    Ok(router.into())
}
