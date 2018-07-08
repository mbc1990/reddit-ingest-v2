#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;
extern crate reqwest;

use std::fs::File;
use std::env;
use std::io::Read;
use std::collections::HashMap;

use reqwest::header::{Headers, Authorization, Basic, UserAgent};

#[derive(Deserialize)]
struct AuthResponse {
    access_token: String,

    #[serde(default)]
    error: i32,
}

fn authenticate(conf: Config) -> Result<AuthResponse, reqwest::Error>{
    let client_id = conf.client_id;
    let client_secret = conf.client_secret;
    let username = conf.username;
    let user_agent = conf.user_agent;

    // Get access token
    let client = reqwest::Client::new();
    let auth_endpoint = "https://www.reddit.com/api/v1/access_token";

    // Set headers
    let mut headers = Headers::new();
    headers.set(
       Authorization(
           Basic {
               username: client_id.to_owned(),
               password: Some(client_secret.to_owned())
           }
       )
    );
    headers.set(UserAgent::new(user_agent));

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
    let json: AuthResponse  = response.json()?;
    Ok(json)
}

#[derive(Debug, Deserialize)]
struct Config {
    client_id: String,
    client_secret: String,
    username: String,
    user_agent: String,
}

fn main() {
    let mut args = env::args();
    let mut input = String::new();
    if args.len() > 1 {
        let name = args.nth(1).unwrap();
        File::open(&name).and_then(|mut f| {
            f.read_to_string(&mut input)
        }).unwrap();
    } else {
		println!("Must pass in file name of configuration");
        return
    }
    let decoded: Config = toml::from_str(&input).unwrap();
    let resp = authenticate(decoded);
    println!("{}", resp.unwrap().access_token);
}
