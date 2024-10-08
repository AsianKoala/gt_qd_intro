use crate::models;
use reqwest::Error;

pub struct ServerConfig {
    pub token: String,
    pub endpoint: String,
    pub ping_interval: u64,
}

/// # Returns
/// * `Result<ServerConfig, Error>` - On success, returns a `ServerConfig` containing the token, endpoint, and ping interval.
///   On failure, returns a `reqwest::Error` indicating what went wrong during the request.
///
/// # Errors
/// This function can return an error if the request fails (e.g., network issues) or if the response cannot be parsed correctly.
pub async fn get_ws_server_info() -> Result<ServerConfig, Error> {
    let url = "https://api-futures.kucoin.com/api/v1/bullet-public";
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .send()
        .await?
        .json::<models::ServerRoot>()
        .await?;

    Ok(ServerConfig {
        token: response.data.token,
        endpoint: response.data.instance_servers[0].endpoint.clone(),
        ping_interval: response.data.instance_servers[0].ping_interval,
    })
}
