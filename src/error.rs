use axum::response::{IntoResponse, Response};

//#[derive(thiserror::Error, Debug)]
pub enum MyError {
    ReqwestError(reqwest::Error),
    BadResponseStatus,
    EmptyData,
    EmptyEventByTopic,
    EmptyNodes,
    StringToContractConversionFailed(String, stellar_strkey::DecodeError),
    FromXdrError(stellar_xdr::curr::Error),
    ToXdrError(stellar_xdr::curr::Error),
    ExtendError(soroban_cli::commands::contract::extend::Error),
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
            MyError::StringToContractConversionFailed(address, decode_error) => format!("Failed to convert String {:#?} into Contract: {:#?}", address, decode_error),
            MyError::FromXdrError(conversion_error) => format!("Failed to convert xdr value: {}", conversion_error),
            MyError::ToXdrError(conversion_error) => format!("Failed to create xdr value : {}", conversion_error),
            MyError::ExtendError(extend_error) => format!("Failed to extend contract: {}", extend_error),
        };

        body.into_response()
    }
}