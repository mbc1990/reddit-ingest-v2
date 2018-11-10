#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate toml;
extern crate reqwest;
extern crate threadpool;

use std::fs::File;
use std::env;
use std::io::Read;
use std::sync::mpsc::channel;

use threadpool::ThreadPool;
use std::thread;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use reqwest::Client;
use reqwest::header::{Headers, Authorization, Basic, UserAgent, Bearer};
use std::collections::HashMap;


mod reddit_api_client;
mod reddit_client;
mod config;

fn authenticate(client_id: String, client_secret: String, user_agent: String) -> String {
    let client = Client::new();
    let auth_endpoint = "https://www.reddit.com/api/v1/access_token";
    let mut headers = Headers::new();
    headers.set(
        Authorization(
            Basic {
                username: client_id,
                password: Some(client_secret)
            }
        )
    );

    headers.set(UserAgent::new(user_agent));

    let mut params = HashMap::new();
    params.insert("grant_type", "client_credentials");

    // Required
    params.insert("device_id", "1");

    let mut response = client.post(auth_endpoint)
        .headers(headers)
        .form(&params)
        .send()
        .unwrap();

    let json: reddit_api_client::AuthResponse = response.json().unwrap();
    return json.access_token;
}

pub fn parse_comment_tree(entry: &serde_json::Value) -> Vec<String> {
    let mut comments = Vec::new();
    if entry["data"]["children"].is_null() {
        return comments;
    }

    let inner_entries = entry["data"]["children"].as_array().unwrap().to_owned();
    for inner in inner_entries.iter() {

        // First get the current (parent) comment's text
        let comment_body = &inner["data"]["body"].to_string();
        comments.push(comment_body.to_string());

        // If replies are null, that means either there are no more, or we need to make a request to /morechildren
        if inner["data"]["replies"].is_null() {
            if inner["kind"] == "more" {
                continue;
                //  println!("Trying to get more comments for {:?}", inner);
                // TODO: Make a request for more comments, and continue parsing recursively
                // TODO: This seems to be the endpoint api/morechildren
            } else {
                // We are at a leaf of a comment tree and can stop
                continue;
            }
        }

        // Go over the children and recursively gather their comments
        let children = &inner["data"]["replies"];
        let child_comments = parse_comment_tree(children);
        comments.append(&mut child_comments.clone());
    }

    return comments;
}

// TODO: If this is the "subreddit" worker, it should have another sender to enqueue comments requests
// TODO: Need a worker client here - pass initial auth token?
// pub fn worker(tx_output: Sender<String>, tx_auth_manager: Sender<String>, rx_work_queue: Receiver<String>, rx_auth_manager: Receiver<String>) {
pub fn worker(rx_work_queue: Receiver<String>, tx_output: Sender<String>, auth_token: String, user_agent: String) {
    let client = reddit_api_client::RedditAPIClient::new(auth_token, user_agent);
    loop {
        let new_work = rx_work_queue.try_recv();
        match new_work {
            Ok(val) => {
                println!("Received new work: {:?}", val);
                // TODO: Use the client to make a query
                let output = &[val, "-output".to_string()].concat();
                tx_output.send(output.to_string());
            }
            Err(_e) => {
                // println!("Error receiving from queue (no data?)")
            }
        }
    }
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

    let decoded: config::Config = toml::from_str(&input).unwrap();

    let initial_auth_token = authenticate(decoded.client_id.clone(), decoded.client_secret.clone(), decoded.user_agent.clone());

    // These are the first tasks to be passed to workers
    let subreddits = decoded.subreddits.clone();

    let num_worker_threads = 4;

    // Channels for sending work to workers
    let mut worker_txs = Vec::new();

    // Channels for communicating output back to main thread
    let (tx_output, rx_output) = channel();

    // Start the worker threads
    for _i in 0..num_worker_threads {
        let (tx_work_queue, rx_work_queue) = channel();
        worker_txs.push(tx_work_queue.clone());
        let out_sender = tx_output.clone();
        let auth = initial_auth_token.clone();
        let user_agent = decoded.user_agent.clone();
        thread::spawn(move || {
            worker(rx_work_queue, out_sender, auth, user_agent);
        });
    }

    for subreddit in subreddits.iter() {
        let copied_input = subreddit.clone();
        // TODO: Randomly select a worker
        let worker = worker_txs.first().unwrap();
        let res = worker.send(copied_input.to_string());
        match res {
            Ok(_val) => {
                println!("Successfully enqueued {:?}",  subreddit);
            }
            Err(e) => {
                println!("Faiure to enqueue: {:?}", e);
            }
        }
    }

    for output in rx_output.iter() {
        println!("Received output: {:?}", output);
    }
}

/*
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
*/
