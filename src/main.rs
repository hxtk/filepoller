mod scheduler;

use std::env;
use std::fs;
use std::time::Duration;

use httpdate;
use reqwest::header;
use reqwest::Client;
use reqwest::Url;

use scheduler::TaskScheduler;

/// Given an HTTP client, filename, and url, return true if the HTTP last-modified time
/// for the file at the URL is more recent than the filename's OS last-modified time.
///
/// If the network is unreachable, the local file is unreachable, or the remote header is
/// bad or missing, then it is assumed that the remote file is newer.
fn newer_remote(client: &Client, filename: &str, url: &Url) -> bool {
    // Local file last-modified time.
    let modtime = match fs::metadata(filename) {
        Err(e) => {
            println!("could not stat file: {}", e);
            return true;
        }
        Ok(metadata) => match metadata.modified() {
            Ok(modtime) => modtime,
            Err(e) => {
                println!("could not get last-modified: {}", e);
                return true;
            }
        },
    };

    // Get last-modified time of remote file.
    match client.head(Url::parse(url.as_str()).unwrap()).send() {
        Err(e) => {
            println!("could not head remote: {}", e);
            true
        }
        Ok(res) => {
            match res.headers().get(header::LAST_MODIFIED) {
                None => {
                    println!("missing last-modified header");
                    true
                }
                Some(datebytes) => {
                    match datebytes.to_str() {
                        Err(e) => {
                            println!("bad last-modified header: {}", e);
                            true
                        }
                        Ok(datestr) => {
                            match httpdate::parse_http_date(datestr) {
                                Err(e) => {
                                    println!("bad last-modified header: {}", e);
                                    true
                                }

                                // both time values were valid; compare
                                Ok(remote) => modtime < remote,
                            }
                        }
                    }
                }
            }
        }
    }
}

/// task to fetch a file and save it to a destination of the remote is newer.
fn task(url_str: &str, dest: &str) {
    let url = match Url::parse(url_str) {
        Ok(x) => x,
        Err(e) => {
            println!("error parsing url: {}", e);
            return;
        }
    };

    let client = Client::new();
    if !newer_remote(&client, dest, &url) {
        return;
    }

    let request = client.get(url);
    // Unwrap guaranteed safe because request is built statically.
    match request.send() {
        Ok(mut res) => {
            if res.status().is_success() {
                let mut bytes = Vec::new();
                if let Err(e) = res.copy_to(&mut bytes) {
                    println!("error reading response body: {}", e);
                    return;
                }

                if let Err(e) = fs::write(dest, bytes) {
                    println!("{}", e);
                }
            }
        }
        Err(e) => println!("{}", e),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} <url> <output file>", args[0]);
        std::process::exit(1);
    }

    let poller = TaskScheduler::new();

    if let Err(msg) = poller.add_task(Duration::new(30, 0), move || task(&args[1], &args[2])) {
        println!("{}", msg);
        return;
    }

    if let Err(msg) = poller.run(Duration::new(30, 0)) {
        println!("{}", msg);
    }
}
