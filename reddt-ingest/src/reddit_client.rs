extern crate serde_json;

use std::collections::HashMap;

use reqwest::header::{Headers, Authorization, Basic, UserAgent, Bearer};
use reqwest::Client;
use config;

#[derive(Deserialize)]
struct AuthResponse {
    access_token: String
}

pub struct RedditClient {
    auth_token: String,
    conf: config::Config,
}

impl RedditClient {
    pub fn new(conf: config::Config) -> RedditClient {
        let mut rc = RedditClient {
            conf: conf,
            auth_token: String::new(),
        };
        rc.authenticate();
        rc
    }

    // Basic Authentication
    fn authenticate(&mut self) {
        let client = Client::new();
        let auth_endpoint = "https://www.reddit.com/api/v1/access_token";
        let mut headers = Headers::new();
        headers.set(
            Authorization(
                Basic {
                    username: self.conf.client_id.to_owned(),
                    password: Some(self.conf.client_secret.to_owned())
                }
            )
        );

        headers.set(UserAgent::new(self.conf.user_agent.clone()));

        let mut params = HashMap::new();
        params.insert("grant_type", "client_credentials");

        // Required
        params.insert("device_id", "1");

        let mut response = client.post(auth_endpoint)
            .headers(headers)
            .form(&params)
            .send()
            .unwrap();

        let json: AuthResponse = response.json().unwrap();
        self.auth_token = json.access_token;
    }

    // TODO: this body is the same as get_comments, refactor`
    pub fn get_subreddit(&self) -> Result<serde_json::Value, serde_json::Error> {
        println!("Attempting to get subreddit");
        let client = Client::new();
        let url = "https://oauth.reddit.com/r/news";

        let mut headers = Headers::new();
        headers.set(
            Authorization(
                Bearer {
                    token: self.auth_token.clone()
                }
            )
        );
        headers.set(UserAgent::new(self.conf.user_agent.clone()));
        let mut response = client.get(url)
            .headers(headers)
            .send()
            .expect("Failed to send request");
        println!("Response for get subreddit: ");
        let v: serde_json::Value = serde_json::from_str(&response.text().unwrap())?;
        println!("{:?}", v);
        return Ok(v)
    }

    pub fn get_comments(&self, url: &String) -> Result<serde_json::Value, serde_json::Error> {
        println!("Attempting to get comments");
        println!("{:?}", url);
        let client = Client::new();
        let mut headers = Headers::new();
        headers.set(
            Authorization(
                Bearer {
                    token: self.auth_token.clone()
                }
            )
        );
        headers.set(UserAgent::new(self.conf.user_agent.clone()));
        let mut response = client.get(url)
            .headers(headers)
            .send()
            .expect("Failed to send request");

        let v: serde_json::Value = serde_json::from_str(&response.text().unwrap())?;
        return Ok(v);
    }

    pub fn parse_comment_tree(&self, entry: &serde_json::Value) -> Vec<&str> {
        let comments = Vec::new();
        let inner_entries = entry["data"]["children"].as_array().unwrap();
        // TODO: Get top level comment, add to vector
        // TODO: recursively call parse_comment_tree for each child
        // TODO: return top level comment + all child comments
        for inner in inner_entries.iter() {
            println!("Inner entry");
            println!("{:?}", inner);
        }
        return comments;
    }
}
