use reqwest::Client as NonBlockingClient;
use reqwest::blocking::Client as BlockingClient;
use uuid::Uuid;

const BASE_URL: &str = "http://127.0.0.1:3000/kv";

fn workload_get() {}

async fn populate_keys(num_keys: u32) -> Vec<String> {
    let client = NonBlockingClient::new();
    let mut keys: Vec<String> = Vec::new();

    for _ in 0..num_keys {
        let uuid: String = Uuid::new_v4().to_string();
        match client.put(format!("{}/{}", BASE_URL, uuid)).send().await {
            Ok(response) => {
                let text = response.text().await.unwrap();
                keys.push(uuid);
                println!("Response: {}", text);
            }
            Err(err) => {
                eprintln!("{}", err);
            }
        }
    }
    keys
}

fn workload_put() {}

fn workload_mixed() {}

fn main() {
    println!("Welcome to the load generator");

    const NUM_THREADS: usize = 10;
    const NUM_REQUESTS_PER_THREAD: usize = 10000;

    let handles: [std::thread::JoinHandle<usize>; NUM_THREADS] = [(); NUM_THREADS].map(|_| {
        std::thread::spawn(move || {
            let client = BlockingClient::new();
            let mut num_errors = 0;

            for _ in 0..NUM_REQUESTS_PER_THREAD {
                let key = "Aditya";
                match client.get(format!("{}/{}", BASE_URL, key)).send() {
                    Ok(response) => {
                        let text = response.text().unwrap();
                        println!("Response: {}", text);
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                        num_errors += 1;
                    }
                }
            }
            return num_errors;
        })
    });

    let mut num_errors = 0;
    for handle in handles {
        num_errors += handle.join().unwrap();
    }
    println!(
        "Total errors: {}/{} i.e. {} %",
        num_errors,
        NUM_THREADS * NUM_REQUESTS_PER_THREAD,
        100.0 * num_errors as f32 / (NUM_THREADS * NUM_REQUESTS_PER_THREAD) as f32
    );
}
