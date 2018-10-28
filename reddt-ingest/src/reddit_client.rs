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
    pub fn get_subreddit(&self, url: &str ) -> Result<serde_json::Value, serde_json::Error> {
        let client = Client::new();
        // let url = "https://oauth.reddit.com/r/news";

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
        return Ok(v)
    }

    pub fn get_comments(&self, url: &String) -> Result<serde_json::Value, serde_json::Error> {
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
    // TODO: To get whole comment trees, needs to make paging http requests
    pub fn parse_comment_tree(&self, entry: &serde_json::Value) -> Vec<String> {
        let mut comments = Vec::new();
        if entry["data"]["children"].is_null() {
            return comments;
        }

        let inner_entries = entry["data"]["children"].as_array().unwrap().to_owned();
        for inner in inner_entries.iter() {
            if inner["data"]["replies"].is_null() {
                continue;
            }
            let comment_body = &inner["data"]["body"];
            comments.push(comment_body.to_string());
            let children = &inner["data"]["replies"];
            let child_comments = &self.parse_comment_tree(children);
            comments.append(&mut child_comments.clone());
        }
        return comments;
    }
}
