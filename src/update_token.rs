use crate::AppState;
use tokio::time::{Duration, sleep};
use std::sync::Arc;
use std::{io::Write, fs::File};
use graphql_client::{GraphQLQuery, Response as GraphQLResponse};

// Generate a module named new_jwt_token
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/new_token/schema.graphql",
    query_path  = "queries/new_token/mutation.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct NewJwtToken;

// Function to automatically update the Mercury token access every 7 days
pub async fn renew_jwt_cron_job(state: Arc<AppState>) {
    
    tokio::spawn(async move {

        // Token expiration time
        let mut time_to_sleep;

        // GraphQL message variables
        let variables = new_jwt_token::Variables {
            email: state.mercury_id.clone(),
            password: state.mercury_pwd.clone(),
        };

        // Build the GraphQL request body
        let request_body = NewJwtToken::build_query(variables);

        let client = reqwest::Client::new();

        // Loop to continuously renew the JWT token
        loop {

            tracing::debug!("Updating Mercury JWT token");
            
            // Send the GraphQL mutation to update the token
            // If it returns an error, retry after 10 seconds, otherwise after 7 days
            match client
                    .post(format!("{}/graphql", state.mercury_graphql_endpoint))
                    .json(&request_body)
                    .send()
                    .await {
                        
                        // Update the Mutex and write the token value to a file
                        Ok(res) => {
                            let response_body: GraphQLResponse<new_jwt_token::ResponseData> = res.json().await.unwrap();
                            tracing::debug!("NEW JWT TOKEN RESPONSE: {:?}", response_body);
                            let new_token = response_body
                                .data
                                .unwrap()
                                .authenticate
                                .unwrap()
                                .jwt_token
                                .unwrap();
                            let mut file = File::create("./mercury-access-token").unwrap();
                            file.write_all(new_token.as_bytes()).unwrap();
                            *state.mercury_jwt_token.lock().unwrap() = new_token;
                            time_to_sleep = Duration::from_secs(59 * 60 * 24 * 7); // ~7 days
                        },
                        
                        // Print an error message and try again in 10 seconds
                        Err(e) => {
                            tracing::error!("Error while updating Mercury JWT token: {:?}", e);
                            time_to_sleep = Duration::from_secs(10); // 10 seconds
                        }
            };

            // Wait time_to_sleep before renewing the token
            sleep(time_to_sleep).await;
        }
    });
}