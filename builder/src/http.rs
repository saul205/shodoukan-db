use std::time::Duration;
use reqwest::blocking::Client;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const READ_TIMEOUT: Duration = Duration::from_secs(600);
const RETRIES: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_secs(10);

fn client() -> Client {
    Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(READ_TIMEOUT)
        .user_agent("shodoukan-db-builder")
        .build()
        .expect("failed to build HTTP client")
}

/// Download `url`, retrying up to 3 times. Panics if all attempts fail.
pub fn fetch_bytes(url: &str) -> Vec<u8> {
    let client = client();
    for attempt in 1..=RETRIES {
        match client.get(url).send().and_then(|r| r.error_for_status()).and_then(|r| r.bytes()) {
            Ok(b) => return b.to_vec(),
            Err(e) if attempt < RETRIES => {
                eprintln!(
                    "  Download attempt {}/{} failed: {} — retrying in {}s...",
                    attempt, RETRIES, e, RETRY_DELAY.as_secs()
                );
                std::thread::sleep(RETRY_DELAY);
            }
            Err(e) => panic!("failed to download {url}: {e}"),
        }
    }
    unreachable!()
}

/// Download `url`, returning `None` on 404 or any permanent failure.
/// Used for optional per-language Tatoeba files that may not exist.
pub fn try_fetch_bytes(url: &str) -> Option<Vec<u8>> {
    let client = client();
    for attempt in 1..=RETRIES {
        match client.get(url).send().and_then(|r| r.error_for_status()).and_then(|r| r.bytes()) {
            Ok(b) => return Some(b.to_vec()),
            Err(e) if e.status().map_or(false, |s| s.as_u16() == 404) => return None,
            Err(e) if attempt < RETRIES => {
                eprintln!(
                    "  Download attempt {}/{} failed: {} — retrying in {}s...",
                    attempt, RETRIES, e, RETRY_DELAY.as_secs()
                );
                std::thread::sleep(RETRY_DELAY);
            }
            Err(_) => return None,
        }
    }
    unreachable!()
}
