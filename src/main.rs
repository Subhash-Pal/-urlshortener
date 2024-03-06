//use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::{ BufRead};
use std::io::{BufReader, BufWriter, Read, Write};
use std::sync::Mutex;
use std::collections::HashMap;
use actix_web::{web, HttpResponse};
use std::fs::OpenOptions;

use tiny_keccak::{Hasher, Sha3};


#[derive(Debug, Deserialize, Serialize)]
struct UrlData {
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ResponseData {
    original_url_received: String,
    shortened_url: String,
    original_url_retrieved: String,
    original_url_matches: bool,
    received_count: u32,
}

lazy_static::lazy_static! {
    static ref SHORTENED_URLS: Mutex<HashMap<String, (String, u32)>> = Mutex::new(HashMap::new());
}

fn save_top_urls(urls: Vec<(String, u32)>) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)  // Truncate the file before writing
        .create(true)
        .open("top_urls.txt")?;
    let mut writer = BufWriter::new(file);

    for (url, count) in urls {
        writeln!(writer, "{}:{}", url, count)?;
    }

    Ok(())
}

fn generate_shortened_url_key(original_url: &str) -> String {
    let mut hasher = Sha3::v256();
    hasher.update(original_url.as_bytes());
    let mut result = [0u8; 32];
    hasher.finalize(&mut result);

    let mut shortened_url = String::new();
    for byte in result.iter().take(6) {
        shortened_url.push_str(&format!("{:02x}", byte));
    }

    shortened_url
}


async fn shorten_and_retrieve_url(req_body: web::Json<UrlData>) -> HttpResponse {
    let original_url_received = req_body.url.clone();
    let mut storage = SHORTENED_URLS.lock().unwrap();

    /* let shortened_url_key = {
        let mut hasher = Sha3::v256();
        hasher.update(original_url_received.as_bytes());
        let mut result = [0u8; 32];
        hasher.finalize(&mut result);

        let mut shortened_url = String::new();
        for byte in result.iter().take(6) {
            shortened_url.push_str(&format!("{:02x}", byte));
        }
        shortened_url
    };
 */
    let shortened_url_key=generate_shortened_url_key(&original_url_received);
    if let Some((_, request_count)) = storage.get_mut(&shortened_url_key) {
        *request_count += 1;  

        println!("URL already exists. Count: {}", *request_count); // Debug output

        return HttpResponse::Ok().json(ResponseData {
            original_url_received: original_url_received.clone(),
            shortened_url: shortened_url_key.clone(),
            original_url_retrieved: original_url_received.clone(),
            original_url_matches: true,
            received_count: *request_count,
        });
    }

    storage.insert(shortened_url_key.clone(), (original_url_received.clone(), 1));

    println!("New URL inserted. Count: 1"); // Debug output

    HttpResponse::Ok().json(ResponseData {
        original_url_received: original_url_received.clone(),
        shortened_url: shortened_url_key.clone(),
        original_url_retrieved: original_url_received.clone(),
        original_url_matches: true,
        received_count: 1, // This value is changed to *request_count
    })
}

async fn retrieve_original_url(req_body: web::Json<UrlData>) -> HttpResponse {
    let shortened_url_received = req_body.url.clone();
    let mut storage = SHORTENED_URLS.lock().unwrap();

    println!("Stored shortened URLs: {:?}", storage.keys()); // Debug output

    // Check if the shortened URL exists in the storage
    if let Some((_original_url, count)) = storage.get_mut(&shortened_url_received) {
        println!("Found shortened URL in storage: {:?}", shortened_url_received);
        // Increment the request count
        *count += 1;
        println!("Incremented count: {}", *count); // Debug output

        // Retrieve the count after updating
        let request_count = *count;

        let (original_url, _) = storage.get(&shortened_url_received).unwrap();
        return HttpResponse::Ok().json(ResponseData {
            original_url_received: original_url.clone(),
            shortened_url: shortened_url_received.clone(),
            original_url_retrieved: original_url.clone(),
            original_url_matches: true,
            received_count: request_count,
        });
    } else {
        println!("Shortened URL not found: {:?}", shortened_url_received);
        // Insert the shortened URL with an initial count of 1
        storage.insert(shortened_url_received.clone(), (req_body.url.clone(), 1));

        // Retrieve the count after insertion
        let request_count = 1;

        let (original_url, _) = storage.get(&shortened_url_received).unwrap();
        return HttpResponse::Ok().json(ResponseData {
            original_url_received: original_url.clone(),
            shortened_url: shortened_url_received.clone(),
            original_url_retrieved: original_url.clone(),
            original_url_matches: true,
            received_count: request_count,
        });
    }
}

fn load_top_urls() -> std::io::Result<()> {
    let file = File::open("top_urls.txt")?;
    let reader = BufReader::new(file);
    let mut storage = SHORTENED_URLS.lock().unwrap();

    for line in reader.lines() {
        let line = line?;
        if let Some(index) = line.rfind(':') {
            let (url, count_str) = line.split_at(index);
            let url = url.trim();
            let count_str = &count_str[1..].trim(); // Skip the colon and trim whitespace
            println!("URL: '{}', Count: '{}'", url, count_str); // Debug output
            let count = count_str.parse().unwrap_or(0);
            println!("Parsed count: {}", count); // Debug output
            storage.insert(url.to_string(), (url.to_string(), count)); // Store the URL along with its count
        }
    }

    Ok(())
}


fn get_top_urls(storage: &Mutex<HashMap<String, (String, u32)>>) -> Vec<(String, u32)> {
    let storage = storage.lock().unwrap();
    let mut url_counts: Vec<_> = storage.iter()
        .map(|(_, (original_url, count))| (original_url.clone(), *count))
        .collect();

    url_counts.sort_by_key(|&(_, count)| std::cmp::Reverse(count)); // Sort by count in descending order

    println!("URL Counts: {:?}", url_counts); // Debug output

    url_counts.into_iter().take(3).collect()
}


async fn top_urls() -> HttpResponse {
    let top_urls = get_top_urls(&SHORTENED_URLS);
    println!("Top URLs: {:?}", top_urls); // Debug output

    if let Err(err) = save_top_urls(top_urls.clone()) {
        eprintln!("Failed to save top URLs: {}", err);
    }

    HttpResponse::Ok().json(top_urls)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
     // Load top URLs from file when program starts
     load_top_urls()?;


    actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .route("/shorten-and-retrieve-url", web::post().to(shorten_and_retrieve_url))
            .route("/retrieve-original-url", web::post().to(retrieve_original_url))
            .route("/top-urls", web::get().to(top_urls))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App,test};

    #[actix_rt::test]
    async fn test_shorten_and_retrieve_url() {
        // Create a test app
        let mut app = test::init_service(
            App::new()
                .route("/shorten-and-retrieve-url", web::post().to(shorten_and_retrieve_url))
        )
        .await;

        // Make a POST request to the endpoint
        let req_body = UrlData { url: "https://coderprog.com".to_string() };
        let req = test::TestRequest::post()
            .uri("/shorten-and-retrieve-url")
            .set_json(&req_body)
            .to_request();
        let resp = test::call_service(&mut app, req).await;

        // Check if the response is successful
        assert!(resp.status().is_success());

        // Parse the response body and verify its contents
        let body = test::read_body(resp).await;
        let response_data: ResponseData = serde_json::from_slice(&body).unwrap();
        assert_eq!(response_data.original_url_received, "https://coderprog.com");
        // Add more assertions as needed
    }
}
