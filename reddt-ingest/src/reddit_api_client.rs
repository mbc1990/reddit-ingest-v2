extern crate serde_json;

use std::collections::HashMap;

use reqwest::header::{Headers, Authorization, Basic, UserAgent, Bearer};
use reqwest::Client;
use config;

#[derive(Deserialize)]
pub struct AuthResponse {
    pub access_token: String
}

pub struct RedditAPIClient {
    auth_token: String,
    user_agent: String
}

impl RedditAPIClient {

    pub fn new(auth_token: String, user_agent: String) -> RedditAPIClient {
        let mut rac = RedditAPIClient{
            auth_token,
            user_agent
        };
        rac
    }

    pub fn do_authenticated_request(&self, api_path: &String) -> Result<serde_json::Value, serde_json::Error> {
        let url = &["https://oauth.reddit.com/", api_path].concat();
        let client = Client::new();
        let mut headers = Headers::new();
        headers.set(
            Authorization(
                Bearer {
                    token: self.auth_token.clone()
                }
            )
        );
        headers.set(UserAgent::new(self.user_agent.clone()));
        let mut response = client.get(url)
            .headers(headers)
            .send()
            .expect("Failed to send request");

        let v: serde_json::Value = serde_json::from_str(&response.text().unwrap())?;
        return Ok(v);
    }

}
