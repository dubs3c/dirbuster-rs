use std::error::Error;

use clap::Parser;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task;

mod tui;

async fn parse_wordlist(wordlist: &str, sender: Sender<String>) -> Result<(), Box<dyn Error>> {
    let file = File::open(wordlist).await?;

    let mut reader = BufReader::new(file);
    let mut line = String::new();
    while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
        sender.send(line.clone()).await?;
        // Clear the buffer for the next line
        line.clear();
    }
    Ok(())
}

async fn bruteforce(domain: String, mut receiver: Receiver<String>) {
    let req = reqwest::Client::new();
    let mut tasks = Vec::new();
    let concurrency_limit = 50;

    while let Some(filename) = receiver.recv().await {
        let rq = req.to_owned();
        let url = format!("{}{}", domain, filename);
        let task = task::spawn(async move {
            let resp = rq.get(&url).send().await;
            match resp {
                Ok(r) => {
                    if r.status().is_success() {
                        println!("[+] {} : {}", url.trim(), r.status());
                    } else {
                        println!("[-] {} : {}", url.trim(), r.status());
                    }
                }
                Err(e) => {
                    eprintln!("[-] Failed to make request to {}: {}", url, e);
                }
            }
        });
        tasks.push(task);

        if tasks.len() >= concurrency_limit {
            // Await all tasks in the current batch
            for task in tasks.drain(..) {
                let _ = task.await;
            }
        }
    }

    // Make sure all tasks are awiated
    for task in tasks {
        let _ = task.await;
    }
}

#[tokio::main]
async fn main() {
    let args = tui::Args::parse();
    let mut domain = args.domain;
    let wordlist = args.wordlist;

    if !domain.ends_with("/") {
        domain = format!("{}/", domain);
    }

    println!("[!] Testing URL: {}", domain);

    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel(100);
    let producer = tokio::spawn(async move {
        match parse_wordlist(wordlist.as_str(), tx).await {
            Ok(..) => (),
            Err(err) => eprintln!("[-] parse_wordlist failed: {}", err),
        }
    });

    let consumer = tokio::spawn(async move { bruteforce(domain.to_string(), rx).await });

    match producer.await {
        Ok(..) => (),
        Err(err) => eprintln!("[-] parse_wordlist failed: {}", err),
    };

    match consumer.await {
        Ok(..) => (),
        Err(err) => eprintln!("[-] parse_wordlist failed: {}", err),
    };

    println!("[!] Bruteforce complete")
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_WORDLIST: &str = "tests/assets/test_wordlist.txt";

    #[tokio::test]
    async fn test_parse_wordlist() {
        let (tx, mut rx): (Sender<String>, Receiver<String>) = mpsc::channel(100);
        let producer = tokio::spawn(async move {
            match parse_wordlist(VALID_WORDLIST, tx).await {
                Ok(..) => (),
                Err(err) => eprintln!("[-] parse_wordlist failed: {}", err),
            }
        });

        let mut files: Vec<String> = vec![];
        while let Some(filename) = rx.recv().await {
            files.push(filename)
        }

        assert!(producer.await.is_ok());
        assert_eq!(files.len(), 10);
    }
}
