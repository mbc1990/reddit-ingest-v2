extern crate serde_json;

use std::sync::mpsc::Receiver;
use reddit_api_task::RedditAPITask;
use std::sync::mpsc::Sender;
use reddit_api_client;

pub struct RedditWorker {
    rx_work_queue: Receiver<RedditAPITask>,
    tx_output: Sender<String> ,
    tx_work_queue: Sender<RedditAPITask>,
    client: reddit_api_client::RedditAPIClient
}

impl RedditWorker {
    pub fn new(rx_work_queue: Receiver<RedditAPITask>, tx_output: Sender<String>, tx_work_queue: Sender<RedditAPITask>, user_agent: String) -> RedditWorker {
        let client = reddit_api_client::RedditAPIClient::new(user_agent);
        let rw = RedditWorker {
            rx_work_queue,
            tx_output,
            tx_work_queue,
            client
        };
        rw
    }

    pub fn start(&self) {
        loop {
            let new_work = self.rx_work_queue.recv();
            let task = new_work.unwrap();
            if task.task_type == "subreddit" {
                self.process_subreddit(task);
            } else if task.task_type == "comments" {
                self.process_comments(task);
            }
        }
    }

    fn process_subreddit(&self, task: RedditAPITask) {
        let api_path = &["/r/", task.query.as_str()].concat();
        let subreddit_result = self.client.do_authenticated_request_with_token(api_path, &task.auth_token);
        let val = subreddit_result.unwrap();
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
            let res = self.tx_work_queue.send(task);
            match res {
                Ok(_) => {}
                Err(e) => {
                    println!("Error sending comments task {:?}", e);
                }
            }
        }
    }

    fn process_comments(&self, task: RedditAPITask) {
        let comments_result = self.client.do_authenticated_request_with_token(&task.query, &task.auth_token);
        let comments = comments_result.unwrap();
        for entry in comments.as_array().unwrap().iter() {
            let raw_comments = self.parse_comment_tree(&entry);
            for comment in raw_comments.iter() {
                let result = self.tx_output.send(comment.to_string());
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error sending output {:?}", e);
                    }
                }
            }
        }
    }

    // Recursively parse a comment tree into an unordered list of comment text
    fn parse_comment_tree(&self, entry: &serde_json::Value) -> Vec<String> {
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
            let child_comments = self.parse_comment_tree(children);
            comments.append(&mut child_comments.clone());
        }

        return comments;
    }
}
