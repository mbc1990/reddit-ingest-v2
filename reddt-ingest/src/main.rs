#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;
extern crate reqwest;

use std::fs;
use std::fmt;
use std::fs::File;
use std::env;
use std::error::Error;
use std::io::Read;
use std::collections::HashMap;

use reqwest::header::{Headers, Authorization, Basic, UserAgent, Bearer};

#[derive(Deserialize)]
struct AuthResponse {
    access_token: String,

    #[serde(default)]
    error: i32,
}

struct RedditClient {
    auth_token: String,
    conf: Config,
}

#[derive(Debug)]
struct ApiParseError;

impl Error for ApiParseError {
    fn description(&self) -> &str {
        "Some kind of failure parsing reddit response"
    }
}

impl fmt::Display for ApiParseError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to parse reddit response")
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
  modhash: Option<String>,

  dist: Option<i64>,

  children: Option<Vec<DataRootInterface>>,

  after: Option<String>,

  before: Option<String>,
}

// TODO: Where does this go in the tree
#[derive(Serialize, Deserialize, Debug)]
struct Data1 {
  subreddit: String,
  selftext: String,
  gilded: i64,
  title: String,
  downs: i64,
  name: String,
  subreddit_type: String,
  ups: i64,
  domain: String,
  is_original_content: bool,
  category: Option<String>,
  score: i64,
  thumbnail: String,
  edited: bool,
  content_categories: Option<String>,
  is_self: bool,
  created: f64,
  author_id: Option<String>,
  post_categories: Option<String>,
  likes: Option<String>,  // TODO: i32?
  view_count: Option<String>,  // TODO: i32?
  pinned: bool,
  over_18: bool,
  media: Option<String>,  // TODO: Should be a media struct
  media_only: bool,
  locked: bool,
  subreddit_id: String,
  id: String,
  author: String,
  num_comments: i64,
  permalink: String,
  stickied: bool,
  url: Option<String>,
  created_utc: f64,
  is_video: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct RootInterface {
  kind: String,
  data: Data,
}

#[derive(Serialize, Deserialize, Debug)]
struct DataRootInterface{
  kind: String,
  data: Data1,
}

impl RedditClient {
    pub fn new(conf: Config) -> RedditClient {
        let mut rc = RedditClient {
            conf: conf,
            auth_token: String::new(),
        };
        rc.authenticate();
        rc
    }

    fn authenticate(&mut self) {
        // Get access token
        let client = reqwest::Client::new();
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

    pub fn get_subreddit(&self) -> Result<RootInterface, ApiParseError> {
        println!("Attempting to get subreddit");
        let client = reqwest::Client::new();
        let url = "https://oauth.reddit.com/r/news";

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

        if let Ok(root_interface) = response.json::<RootInterface>() {
            println!("Success!");
            println!("{:?}", root_interface);
            return Ok(root_interface);
        } else {
            println!("Failure!");
            return Err(ApiParseError);
        }
    }
}

#[derive(Deserialize)]
struct Config {
    client_id: String,
    client_secret: String,
    username: String,  // TODO: Do we need this?
    user_agent: String,
    num_workers: i32,
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
    let rc = RedditClient::new(decoded);
    rc.get_subreddit();
}
