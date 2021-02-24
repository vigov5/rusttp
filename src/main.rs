use clap::{App, Arg};
use futures::future;
use std::io::{self, BufRead};
use std::time::Duration;
use tokio::runtime;

fn main() {
    let matches = App::new("Rusttp")
        .version("0.1.0")
        .author("@vigov5")
        .about("httprobe rust version")
        .arg(
            Arg::with_name("concurrency")
                .short("c")
                .takes_value(true)
                .help("set the concurrency level (default 20)"),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .takes_value(true)
                .help("timeout (milliseconds) (default 10000)"),
        )
        .get_matches();

    let concurrent: usize = matches
        .value_of("concurrency")
        .unwrap_or("20")
        .parse::<usize>()
        .unwrap();

    let timeout: u64 = matches
        .value_of("timeout")
        .unwrap_or("10000")
        .parse::<u64>()
        .unwrap();

    let mut urls = Vec::new();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let sub = line.unwrap();
        urls.push(format!("https://{}", sub));
        urls.push(format!("http://{}", sub));
    }

    let rt = runtime::Builder::new_multi_thread()
        .worker_threads(concurrent)
        .enable_time()
        .enable_io()
        .build()
        .unwrap();

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_millis(timeout))
        .build()
        .unwrap();

    let threads: Vec<_> = urls
        .into_iter()
        .map(|url| {
            let client = client.clone();
            rt.spawn(async move {
                let resp = client.get(&url).send().await?;
                println!("{}", url);
                Ok::<reqwest::Response, reqwest::Error>(resp)
            })
        })
        .collect();

    rt.block_on(async {
        future::join_all(threads).await;
    });
}
