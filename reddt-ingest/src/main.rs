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
    let subreddits = decoded.subreddits.clone();
    let rc = reddit_client::RedditClient::new(decoded);

    let needle = "libs";

    for subreddit in subreddits.iter() {
        println!("{:?}", subreddit);
        let api_query = &["r/", subreddit].concat();
        let resp = rc.do_authenticated_request(api_query).unwrap();
        let stories = resp["data"]["children"].as_array().unwrap();
        let mut total_comments = 0;
        for story in stories.iter() {
            let permalink = &story["data"]["permalink"];

            // TODO: This can't possibly be the simplest way to write this, but it *does* fix the problem of ending up with escaped \" characters in  the query
            let comments_query = permalink.as_str().unwrap().to_string();

            let comments = rc.do_authenticated_request(&comments_query).unwrap();
            for entry in comments.as_array().unwrap().iter() {
                let raw_comments = rc.parse_comment_tree(&entry);
                for comment in raw_comments.iter() {
                    if comment.contains(needle) {
                        println!("{:?}", comment)
                    }
                    total_comments += 1;
                }
            }
        }
        println!("{:?} total comments for {:?}", total_comments, subreddit);
    }
}
