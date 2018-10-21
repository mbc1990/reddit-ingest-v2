extern crate serde_json;

use std::fmt;
use std::error::Error;
use std::collections::HashMap;

use reqwest::header::{Headers, Authorization, Basic, UserAgent, Bearer};
use reqwest::Client;
use config;
use reddit_response::RootInterface;

#[derive(Deserialize)]
struct AuthResponse {
    access_token: String,

    #[serde(default)]
    error: i32,
}

pub struct RedditClient {
    auth_token: String,
    conf: config::Config,
}

#[derive(Debug)]
pub struct ApiParseError;

impl Error for ApiParseError {
    fn description(&self) -> &str {
        "Some kind of failure parsing reddit response"
    }
}

impl fmt::Display for ApiParseError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to parse response")
    }
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

    fn authenticate(&mut self) {
        // Get access token
        let client = Client::new();
        let auth_endpoint = "https://www.reddit.com/api/v1/access_token";

        // Set headers
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

        // Set body
        let mut params = HashMap::new();
        params.insert("grant_type", "client_credentials");
        params.insert("device_id", "1");


        // Set headers
        let mut response = client.post(auth_endpoint)
            .headers(headers)
            .form(&params)
            .send()
            .unwrap();

        let json: AuthResponse  = response.json().unwrap();
        self.auth_token = json.access_token;
    }

    pub fn get_subreddit(&self) -> Result<serde_json::Value, serde_json::Error> {
        println!("Attempting to get subreddit");
        let client = Client::new();
        let url = "https://oauth.reddit.com/r/the_donald";

        // Set headers
        let mut headers = Headers::new();
        headers.set(
            Authorization(
                Bearer {
                    token: self.auth_token.to_owned()
                }
            )
        );
        headers.set(UserAgent::new(self.conf.user_agent.clone()));
        let mut response = client.get(url)
            .headers(headers)
            .send()
            .expect("Failed to send request");

        let v: serde_json::Value = serde_json::from_str(&response.text().unwrap())?;
        println!("{:?}", v);
        return Ok(v);

        /*
        if let Ok(root_interface) = response.json::<RootInterface>() {
            println!("Success!");
            return Ok(root_interface);
        } else {
            println!("Failure!");
            return Err(ApiParseError);
        }
        */
    }

    pub fn get_comments(&self, url: &String) -> Result<RootInterface, ApiParseError> {
        println!("Attempting to get comments");
        let client = Client::new();

        // Set headers
        let mut headers = Headers::new();
        headers.set(
            Authorization(
                Bearer {
                    token: self.auth_token.to_owned()
                }
            )
        );
        headers.set(UserAgent::new(self.conf.user_agent.clone()));
        let mut response = client.get(url)
            .headers(headers)
            .send()
            .expect("Failed to send request");

        // TODO: RootInterface only applies to subreddis? So it's not being parsed correctly here?
        if let Ok(root_interface) = response.json::<RootInterface>() {
            println!("Success!");
            return Ok(root_interface);
        } else {
            println!("Failure!");
            println!("{:?}", response.status());
            println!("{:?}", response.text());
            return Err(ApiParseError);
        }
    }

    /*
    pub fn get_comments(&self, story_url: String) {
        println!("Attempting to get comments");
        // Set headers
        let client = Client::new();
        let mut headers = Headers::new();
        headers.set(
            Authorization(
                Bearer {
                    token: self.auth_token.to_owned()
                }
            )
        );
        headers.set(UserAgent::new(self.conf.user_agent.clone()));
        let base = String::from("https://oauth.reddit.com");
        base.push_str(&story_url);
        println!("URL: {}", base);
        let mut response = client.get(base.to_string())
            .headers(headers)
            .send()
            .expect("Failed to send request");
        println!("{:?}", response);
    }
    */
}
