#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
  modhash: Option<String>,
  dist: Option<i64>,
  pub children: Option<Vec<DataRootInterface>>,
  after: Option<String>,
  before: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data1 {
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
  pub permalink: String,
  stickied: bool,
  url: Option<String>,
  created_utc: f64,
  is_video: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RootInterface {
  kind: String,
  pub data: Data,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DataRootInterface{
  kind: String,
  pub data: Data1,
}

