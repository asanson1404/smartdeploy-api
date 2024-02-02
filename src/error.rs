use axum::response::{IntoResponse, Response};

pub enum MyError {
    ReqwestError(reqwest::Error),
    BadResponseStatus,
    EmptyResponseData,
    EmptyEventByTopic,
    EmptyEdges,
    EmptyEventEdge,
    EmptyEventNode,
    EmptyEventXdrData,
    ScAddressConversionFailed(soroban_sdk::Address, soroban_sdk::ConversionError),
    FromXdrError(stellar_xdr::curr::Error),
    ToXdrError(stellar_xdr::curr::Error),
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
            MyError::EmptyResponseData => "Empty Response Data: Mercury database returns an empty JSON object".to_string(),
            MyError::EmptyEventByTopic => "Empty EventByTopic: the field event_by_topic is empty. Mercury database returns an empty field event_by_topic".to_string(),
            MyError::EmptyEdges => "Empty Edges: the field edges is empty. Mercury database returns an empty field edges. No such events indexed".to_string(),
            MyError::EmptyEventEdge => "Empty Event Edge: Mercury returns an empty event".to_string(),
            MyError::EmptyEventNode => "Empty Event Node: Mercury returns an empty event".to_string(),
            MyError::EmptyEventXdrData => "Empty Event Data: No XDR data for that event".to_string(),
            MyError::ScAddressConversionFailed(address, conversion_error) => format!("Failed to convert Address {:#?} into ScAddress: {:#?}", address, conversion_error),
            MyError::FromXdrError(conversion_error) => format!("Failed to convert xdr value: {}", conversion_error),
            MyError::ToXdrError(conversion_error) => format!("Failed to create xdr value : {}", conversion_error),
        };

        body.into_response()
    }
}