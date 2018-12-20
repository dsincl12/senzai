use std::env;
use std::process::exit;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

extern crate chrono;
extern crate getopts;
extern crate reqwest;

use chrono::prelude::*;
use getopts::Options;

struct LatencyCheck {
    duration: u32,
    url: String,
    ignore_first: bool,
    client: reqwest::Client,
}

impl LatencyCheck {
    fn new(duration: u32, url: String) -> LatencyCheck {
        LatencyCheck {
            duration,
            url,
            ignore_first: true,
            client: reqwest::Client::new(),
        }
    }

    fn begin(&mut self) {
        let (channel_tx, channel_rx) = channel();

        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(1));
            channel_tx.send("").unwrap();
        });

        let mut count = 0;

        for _ in channel_rx {
            if self.ignore_first {
                self.measure_latency();
                self.ignore_first = false;

                continue; // ignore first check
            }

            count += 1;

            if count > self.duration {
                exit(0);
            }

            println!("latency: {} ms", self.measure_latency().num_milliseconds());
        }
    }

    fn measure_latency(&self) -> chrono::Duration {
        let start = Local::now();

        match self.client.head(&self.url).send() {
            Ok(_) => Local::now() - start,
            Err(e) => panic!("error: {}", e),
        }
    }
}

fn print_usage() {
    println!("usage: senzai -t <interval in seconds> -u <url>");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut options = Options::new();
    options.optopt("t", "", "interval in seconds", "INTERVAL");
    options.optopt("u", "", "URL to measure latency of", "URL");

    let matches = match options.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => panic!("error: {}", e.to_string()),
    };

    match matches.opt_str("t") {
        Some(interval) => match matches.opt_str("u") {
            Some(url) => {
                LatencyCheck::new(interval.parse().unwrap(), url).begin();
            }
            None => {
                println!("error: missing url argument");
                print_usage();
            }
        },
        None => {
            println!("error: missing interval argument");
            print_usage();
        }
    }
}
