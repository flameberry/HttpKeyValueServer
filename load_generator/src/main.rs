use reqwest::blocking::Client;

fn main() {
    println!("Welcome to the load generator");

    let baseurl = "http://127.0.0.1:3000/kv";
    const NUM_THREADS: usize = 10;
    const NUM_REQUESTS_PER_THREAD: usize = 10000;

    let handles: [std::thread::JoinHandle<usize>; NUM_THREADS] = [(); NUM_THREADS].map(|_| {
        std::thread::spawn(move || {
            let client = Client::new();
            let mut num_errors = 0;

            for _ in 0..NUM_REQUESTS_PER_THREAD {
                let key = "Aditya";
                match client.get(format!("{}/{}", baseurl, key)).send() {
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
