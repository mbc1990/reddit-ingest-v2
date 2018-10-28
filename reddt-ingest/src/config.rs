#[derive(Deserialize)]
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub username: String,  // TODO: Do we need this?
    pub user_agent: String,
    pub num_workers: i32,
    pub subreddits: Vec<String>,
    pub slack_webhooks: Vec<String>
}
