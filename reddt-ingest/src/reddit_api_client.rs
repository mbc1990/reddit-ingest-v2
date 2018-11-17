extern crate serde_json;

use reqwest::header::{Headers, Authorization, UserAgent, Bearer};
use reqwest::Client;

#[derive(Deserialize)]
pub struct AuthResponse {
    pub access_token: String
}

pub struct RedditAPIClient {
    user_agent: String
}

impl RedditAPIClient {

    pub fn new(user_agent: String) -> RedditAPIClient {
        let mut rac = RedditAPIClient{
            user_agent
        };
        rac
    }

    pub fn do_authenticated_request_with_token(&self, api_path: &String, auth_token: &String) -> Result<serde_json::Value, serde_json::Error> {
        let url = &["https://oauth.reddit.com/", api_path].concat();
        let client = Client::new();
        let mut headers = Headers::new();
        headers.set(
            Authorization(
                Bearer {
                    token: auth_token.parse().unwrap()
                }
            )
        );
        headers.set(UserAgent::new(self.user_agent.clone()));

        // TODO: Add error handling - this is where auth failures will come from
        let mut response = client.get(url)
            .headers(headers)
            .send()
            .expect("Failed to send request");

        let v: serde_json::Value = serde_json::from_str(&response.text().unwrap())?;
        return Ok(v);
    }
}
