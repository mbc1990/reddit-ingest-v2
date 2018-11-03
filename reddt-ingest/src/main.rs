#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;
extern crate reqwest;
extern crate threadpool;

use std::fs::File;
use std::env;
use std::io::Read;
use std::sync::mpsc::channel;

use threadpool::ThreadPool;


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

    // TODO: A single thread that periodically adds subreddits to the queue so we continue updating

    // TODO: A worker pool that reads from a queue, making subreddit or comment requests and doing string comparisons (for now)
    let (tx, rx) = channel();
    let n_workers = 4;
    let pool = ThreadPool::new(n_workers);

    // TODO: Second worker pool that does the string comparisons

    let needle = "liberal";

    for subreddit in subreddits.iter() {
        println!("{:?}", subreddit);
        let api_query = &["r/", subreddit].concat();
        let resp = rc.do_authenticated_request(api_query).unwrap();
        let stories = resp["data"]["children"].as_array().unwrap();
        let mut total_comments = 0;
        for their_story in stories.iter() {
            let story = their_story.clone();
            pool.execute(move|| {
                // Sort by new since this is supposed to be semi-real time
                let permalink = &story["data"]["permalink"];
                let comments_query = &[permalink.as_str().unwrap(), "?sort=new"].concat();

                let comments = rc.do_authenticated_request(comments_query).unwrap();
                for entry in comments.as_array().unwrap().iter() {
                    let mut comments_for_story = 0;
                    let raw_comments = rc.parse_comment_tree(&entry);
                    for comment in raw_comments.iter() {
                        if comment.contains(needle) {
                            println!("{:?}", comment.as_str());

                            // TODO: This will need to work at some point
                            tx.send(comment.to_owned());
                        }
                        // TODO: This probaly won't work
                        total_comments += 1;
                        comments_for_story += 1;
                    }
                    // The first entry has no comments? TODO: Confirm the api structure
                    if comments_for_story > 1 {
                        println!("{:?} total comments for story: {:?}", comments_for_story, permalink);
                    }
                }
            });
        }
        println!("{:?} total comments for {:?}", total_comments, subreddit);
    }
    for output in rx.iter() {
        println!("Output: {:?} ", output);
    }
}
