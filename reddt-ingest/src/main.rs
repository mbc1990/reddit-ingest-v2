#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;
extern crate reqwest;

use std::fs::File;
use std::env;
use std::io::Read;

mod reddit_client;
mod config;

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
    let decoded: config::Config = toml::from_str(&input).unwrap();
    let rc = reddit_client::RedditClient::new(decoded);

    let needle = "soros";

    // let subreddits = ["https://oauth.reddit.com/r/the_donald", "https://oauth.reddit.com/r/conservative,","https://oauth.reddit.com/r/libertarian"];

    let subreddits = ["https://oauth.reddit.com/r/news", "https://oauth.reddit.com/r/worldnews,","https://oauth.reddit.com/r/boston"];


    for subreddit in subreddits.iter() {
        let test_sub = rc.get_subreddit(subreddits[0]).unwrap();
        let stories = test_sub["data"]["children"].as_array().unwrap();
        for story in stories.iter() {
            let permalink = &story["data"]["permalink"];
            let full_url = &["https://oauth.reddit.com", permalink.as_str().unwrap()].concat();
            let test_comments = rc.get_comments(full_url).unwrap();
            for entry in test_comments.as_array().unwrap().iter() {
                let raw_comments = rc.parse_comment_tree(&entry);
                for comment in raw_comments.iter() {
                    if comment.contains(needle) {
                        // TODO: Duplication?
                        println!("{:?}", comment)
                    }
                }
            }
        }
    }
}
