#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate toml;
extern crate rand;
extern crate reqwest;

use rand::Rng;
use std::fs::File;
use std::io::{Write};
use std::{thread, time};
use std::env;
use std::io::Read;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use reqwest::Client;
use reqwest::header::{Headers, Authorization, Basic, UserAgent};
use std::collections::HashMap;
use std::io;
use reddit_api_task::RedditAPITask;

mod reddit_api_client;
mod reddit_api_task;
mod config;

fn authenticate(client_id: String, client_secret: String, user_agent: String) -> String {
    // N.B. For running the binary directly, make sure you specify SSL_CERT_DIR=/etc/ssl/certs
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

// Anyone can submit a new task, this assigns it to a worker
pub fn queue_manager(rx_incoming_tasks: Receiver<RedditAPITask>, worker_txs: Vec<Sender<RedditAPITask>>, num_workers: usize) {
    loop {
        let new_work = rx_incoming_tasks.recv();
        match new_work {
            Ok(task) => {
                let worker_idx = rand::thread_rng().gen_range(0, num_workers);
                let worker = worker_txs.get(worker_idx).unwrap();
                let res = worker.send(task);

                match res {
                    Ok(_val) => {
                    }
                    Err(e) => {
                        println!("Faiure to enqueue: {:?}", e);
                    }
                }
            }
            Err(err) => {
                println!("Failure to read task in queue worker: {:?}", err.to_string());
            }
        }
    }
}

pub fn worker(rx_work_queue: Receiver<RedditAPITask>, tx_output: Sender<String>, tx_work_queue: Sender<RedditAPITask>, user_agent: String) {
    let client = reddit_api_client::RedditAPIClient::new(user_agent);
    loop {
        let new_work = rx_work_queue.recv();
        let task = new_work.unwrap();
        let api_path = &["/r/", task.query.as_str()].concat();

        // Perform subreddit api query
        if task.task_type == "subreddit" {
            let subreddit_result = client.do_authenticated_request_with_token(api_path, &task.auth_token);
            match subreddit_result {
                Ok(val) => {
                    let stories = val["data"]["children"].as_array().unwrap();
                    for their_story in stories.iter() {
                        let permalink = &their_story["data"]["permalink"];
                        let comments_query = &[permalink.as_str().unwrap(), "?sort=new"].concat();
                        let task = RedditAPITask {
                            task_type: "comments".parse().unwrap(),
                            query: comments_query.parse().unwrap(),
                            auth_token: task.auth_token.clone()
                        };

                        // Enqueue the comments api request
                        let res = tx_work_queue.send(task);
                        match res {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Error sending comments task {:?}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Error getting subreddit: {:?}", e)
                }

            }
        } else if task.task_type == "comments" {
            // Perform comments api request
            let comments_result = client.do_authenticated_request_with_token(&task.query, &task.auth_token);
            match comments_result {
                Ok(comments) => {
                    for entry in comments.as_array().unwrap().iter() {
                        let raw_comments = parse_comment_tree(&entry);
                        for comment in raw_comments.iter() {
                            let result = tx_output.send(comment.to_string());
                            match result {
                                Ok(_) => {}
                                Err(e) => {
                                    println!("Error sending output {:?}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Error downloading comments {:?} ", e);
                }
            }
        }
    }
}

fn start_recurring_queries(client_id: String, client_secret: String, user_agent: String, subreddits: Vec<String>, tx_work_queue: Sender<RedditAPITask>) {
    loop {
        let auth_token = authenticate(client_id.clone(), client_secret.clone(), user_agent.clone());
        for subreddit in subreddits.iter() {
            let copied_input = subreddit.clone();
            let task = RedditAPITask {
                task_type: "subreddit".parse().unwrap(),
                query: copied_input,
                auth_token: auth_token.clone()
            };
            let res = tx_work_queue.send(task);
            match res {
                Ok(_val) => {}
                Err(e) => {
                    println!("Faiure to send work to manager: {:?}", e);
                }
            }
        }
        thread::sleep(time::Duration::from_millis(10000));
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

    // Subreddits to query
    let subreddits = decoded.subreddits.clone();

    // Channels for sending work to workers
    let mut worker_txs = Vec::new();

    // Channels for communicating output back to main thread
    let (tx_output, rx_output) = channel();

    // Queue worker receives tasks and assigns them to workers
    let (tx_work_queue, rx_work_queue) = channel();

    // Start the workers
    for _i in 0..decoded.num_workers.clone() {
        let (tx_work_queue, rx_work_queue) = channel();
        worker_txs.push(tx_work_queue.clone());
        let out_sender = tx_output.clone();
        let user_agent = decoded.user_agent.clone();
        thread::spawn(move || {
            worker(rx_work_queue, out_sender, tx_work_queue.clone(), user_agent);
        });
    }
    let client_id = decoded.client_id.clone();
    let client_secret = decoded.client_secret.clone();
    let user_agent = decoded.user_agent.clone();

    // Start the queue manager (distributes work to workers)
    thread::spawn(move || {
        queue_manager(rx_work_queue, worker_txs, decoded.num_workers.clone() as usize);
    });

    // Start the loop that re-enqueues the input subreddits every n seconds
    thread::spawn(move || {
        start_recurring_queries(client_id.clone(), client_secret.clone(), user_agent.clone(), subreddits, tx_work_queue.clone());
    });

    // Prevent duplication of comments from subsequent API requests
    let mut seen_comments = HashMap::new();

    // Write all downloaded comments to stdout
    for output in rx_output.iter() {

        //Handle deduplication
        if seen_comments.contains_key(&output.clone()) {
            continue;
        }
        seen_comments.insert(output.clone(), true);

        // let data_res = io::stdout().write(output.as_ref());
        let data_res = io::stdout().write(output.as_ref());
        match data_res {
            Ok(_) => {}
            Err(e) => {
                println!("failed to write to stdout: {:?}", e);
            }
        }
        let newline_res = io::stdout().write(b"\n");
        match newline_res {
            Ok(_) => {}
            Err(e) => {
                println!("failed to write to stdout: {:?}", e);
            }
        }
    }
}
