use crate::AppState;
use crate::error::MyError;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub async fn retrieve(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Vec<Ttl>>), MyError> {

    let contracts_ttl = sqlx::query_as::<_, Ttl>("SELECT * FROM contracts_ttl")
        .fetch_all(&state.ttl_pool)
        .await?;

    Ok((StatusCode::OK, Json(contracts_ttl)))

}

pub async fn add(
    State(state): State<Arc<AppState>>,
    Json(ttl_data): Json<Ttl>,
) -> Result<impl IntoResponse, impl IntoResponse> {

    match sqlx::query_as::<_, Ttl> ("INSERT INTO contracts_ttl (contract_id, automatic_bump, live_until_ttl)
                                        VALUES ($1, $2, $3)
                                        ON CONFLICT (contract_id) DO UPDATE
                                        SET automatic_bump = $2, live_until_ttl = $3
                                        RETURNING contract_id, automatic_bump, live_until_ttl;
                                    ")
                                    .bind(ttl_data.contract_id)
                                    .bind(ttl_data.automatic_bump)
                                    .bind(ttl_data.live_until_ttl)
                                    .fetch_one(&state.ttl_pool)
                                    .await
    {
        Ok(data) => Ok((StatusCode::CREATED, Json(data))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Ttl {
    pub contract_id: String,
    pub automatic_bump: bool,
    pub live_until_ttl: i32,
}