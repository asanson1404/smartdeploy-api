use axum::response::{IntoResponse, Response};

pub enum MyError {
    ReqwestError(reqwest::Error),
    BadResponseStatus,
    EmptyData,
    EmptyEventByTopic,
    EmptyNodes,
    EmptyEntryExpiration,
    StringToContractConversionFailed(String, stellar_strkey::DecodeError),
    FromXdrError(stellar_xdr::curr::Error),
    ToXdrError(stellar_xdr::curr::Error),
    ExtendError(soroban_cli::commands::contract::extend::Error),
    ConfigNetworkError(soroban_cli::commands::config::Error),
    KeyError(soroban_cli::key::Error),
    RpcError(soroban_cli::rpc::Error),
}

// Convert soroban_cli::rpc::Error towards MyError::RpcError
impl From<soroban_cli::rpc::Error> for MyError {
    fn from(error: soroban_cli::rpc::Error) -> Self {
        MyError::RpcError(error)
    }
}

// Convert soroban_cli::key::Error towards MyError::KeyError
impl From<soroban_cli::key::Error> for MyError {
    fn from(error: soroban_cli::key::Error) -> Self {
        MyError::KeyError(error)
    }
}

// Convert soroban_cli::commands::contract::extend::Error towards MyError::ExtendError
impl From<soroban_cli::commands::contract::extend::Error> for MyError {
    fn from(error: soroban_cli::commands::contract::extend::Error) -> Self {
        MyError::ExtendError(error)
    }
}

// Convert reqwest::Error towards MyError::ReqwestError
impl From<reqwest::Error> for MyError {
    fn from(error: reqwest::Error) -> Self {
        MyError::ReqwestError(error)
    }
}

// Integrate Error into axum response to use it as a return type in axum handlers 
impl IntoResponse for MyError {
    fn into_response(self) -> Response {
        let body = match self {
            MyError::ReqwestError(reqwest_error) => format!("Reqwest Error: {}", reqwest_error),
            MyError::BadResponseStatus => "Bad Response Status: response status not included between 200 and 300 (excluded).\nResponse not sent".to_string(),
            MyError::EmptyData => "Empty data: Mercury database returns an empty field data".to_string(),
            MyError::EmptyEventByTopic => "Empty event_by_topic: Mercury database returns an empty field event_by_topic".to_string(),
            MyError::EmptyNodes => "Empty Nodes: No such events indexed".to_string(),
            MyError::EmptyEntryExpiration => "Empty entry expiration: the contract instance hasn't been bumped, Mercury can't start tracking it.".to_string(),
            MyError::StringToContractConversionFailed(address, decode_error) => format!("Failed to convert String {:#?} into Contract: {:#?}", address, decode_error),
            MyError::FromXdrError(conversion_error) => format!("Failed to convert xdr value: {}", conversion_error),
            MyError::ToXdrError(conversion_error) => format!("Failed to create xdr value : {}", conversion_error),
            MyError::ExtendError(extend_error) => format!("Failed to extend contract: {}", extend_error),
            MyError::ConfigNetworkError(config_error) => format!("Failed to get network config when using Soroban CLI: {}", config_error),
            MyError::KeyError(key_error) => format!("Failed to parse ledger key when using soroban CLI: {}", key_error),
            MyError::RpcError(rpc_error) => format!("Failed to communicate with RPC: {}", rpc_error),
        };

        body.into_response()
    }
}