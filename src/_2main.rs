use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;
use tiny_keccak::{Hasher, Sha3};

// Define a structure to hold the URL data sent by the client
#[derive(Debug, Deserialize, Serialize)]
struct UrlData {
    url: String,
}

// Define a structure to hold the response data with original and shortened URLs
#[derive(Debug, Serialize)]
struct ResponseData {
    original_url_received: String,
    shortened_url: String,
    original_url_retrieved: Option<String>,
    original_url_matches: Option<bool>,
}

// Global storage for shortened URLs (in-memory)
lazy_static::lazy_static! {
    static ref SHORTENED_URLS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

// Handler function for the API endpoint
async fn shorten_and_retrieve_url(req_body: web::Json<UrlData>) -> HttpResponse {
    // Extract the URL from the JSON data
    let url = &req_body.url;

    // Check if the provided URL is already shortened
    let original_url_received = url.clone();
    let mut original_url_retrieved = None;
    let mut original_url_matches = None;

    let mut shortened_url = if SHORTENED_URLS.lock().unwrap().contains_key(url) {
        // If it's already in the storage, retrieve the original URL
        let original_url = SHORTENED_URLS.lock().unwrap().get(url).unwrap().clone();
        original_url_retrieved = Some(original_url.clone());
        println!("{}, {:?}", original_url, original_url_retrieved); 
        // Output: https://coderprog.com, Some("https://coderprog.com")
        original_url_matches = Some(match &original_url_retrieved {
            Some(received_url) => original_url == *received_url,
            None => false, // Handle the case where original_url_received is None
        });
        original_url
    } else {
        // Otherwise, it's a new URL, so shorten it
        shorten_url(url)
    };

    // If the shortened URL is the same as the original URL received, use it as the shortened URL
    if shortened_url == *url {
        shortened_url = shorten_url(&url);
    }

    // Store the original and shortened URLs in the storage
    SHORTENED_URLS.lock().unwrap().insert(shortened_url.clone(), url.clone());

    // Construct the response data with original and shortened URLs
    let response_data = ResponseData {
        original_url_received,
        shortened_url,
        original_url_retrieved,
        original_url_matches,
    };

    // Return the response with original, shortened, and retrieved URLs
    HttpResponse::Ok().json(response_data)
}

// Function to generate a short URL identifier from the original URL
fn shorten_url(original_url: &str) -> String {
    let mut hasher = Sha3::v256(); // Use the Sha3 hasher
    let mut result = [0u8; 32];
    hasher.update(original_url.as_bytes());
    hasher.finalize(&mut result);

    let mut short_url = String::new();
    for byte in result.iter().take(6) {
        short_url.push_str(&format!("{:02x}", byte));
    }

    short_url
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Start the HTTP server
    actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .route("/shorten-and-retrieve-url", web::post().to(shorten_and_retrieve_url))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
