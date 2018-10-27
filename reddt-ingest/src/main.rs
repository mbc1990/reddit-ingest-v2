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
    let test_sub = rc.get_subreddit().unwrap();
    let stories = test_sub["data"]["children"].as_array().unwrap();
    for story in stories.iter() {
        let author = &story["data"]["author"];
        let permalink = &story["data"]["permalink"];
        let title = &story["data"]["title"];
        println!("{:?}", author);
        println!("{:?}", title);
        println!("{:?}", permalink);
        println!("---------------------------");
        let full_url =  &["https://oauth.reddit.com", permalink.as_str().unwrap()].concat();
        let test_comments = rc.get_comments(full_url).unwrap();
        for entry in test_comments.as_array().unwrap().iter() {
            println!("Found a value in the array");
            println!("{:?}", entry);
            let raw_comments = rc.parse_comment_tree(&entry);
            println!("{:?}", raw_comments);
            for comment in raw_comments.iter() {
                println!("{:?}", comment)
            }
        }
        break;
    }
}
