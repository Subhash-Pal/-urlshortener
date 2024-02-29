use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex};
use std::collections::HashMap;
use tiny_keccak::{Hasher, Sha3};

// Define a structure to hold the URL data sent by the client
#[derive(Debug, Deserialize)]
struct UrlData {
    url: String,
}

// Define a structure to hold the response data with both original and shortened URLs
#[derive(Debug, Serialize)]
struct ResponseData {
    original_url: String,
    shortened_url: String,
    request_count: u32,
}

// Global storage for shortened URLs and their request counts (in-memory)
lazy_static::lazy_static! {
    static ref SHORTENED_URLS: Mutex<HashMap<String, (String, u32)>> = Mutex::new(HashMap::new());
}

// Handler function for receiving URL data from the client, shortening it if necessary,
// and dispatching both URLs back
async fn receive_url(req_body: web::Json<UrlData>) -> HttpResponse {
    // Extract the URL from the JSON data
    let original_url = &req_body.url;

    // Check if the original URL already exists in storage
    let mut storage = SHORTENED_URLS.lock().unwrap();
    let entry = storage.entry(original_url.clone());
    let (shortened_url, request_count) = entry.or_insert_with(|| {
        // Generate a short URL identifier using the original URL
        let shortened_url = shorten_url(original_url);
        (shortened_url.clone(), 0)
    });

    // Increment the request count
    *request_count += 1;

    // Construct the response data with both original and shortened URLs
    let response_data = ResponseData {
        original_url: original_url.clone(),
        shortened_url: shortened_url.clone(),
        request_count: *request_count,
    };

    // Return the response with the original and shortened URLs
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

// Handler function for redirecting to the original URL
async fn redirect_to_original(req: HttpRequest) -> HttpResponse {
    let short_url = req.match_info().get("short_url").unwrap_or("");

    // Look up the original URL corresponding to the short URL
    let storage = SHORTENED_URLS.lock().unwrap();
    if let Some((_original_url, request_count)) = storage.get(short_url) {
        return HttpResponse::Ok().json(ResponseData {
            original_url: "".to_string(),
            shortened_url: short_url.to_string(),
            request_count: *request_count,
        });
    }

    HttpResponse::NotFound().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            // Define a route for handling POST requests
            .route("/send-url", web::post().to(receive_url))
            // Define a route for redirection
            .route("/{short_url}", web::get().to(redirect_to_original))
    })
    .bind("127.0.0.1:8080")? // Bind the server to the local address and port 8080
    .run() // Start the server
    .await // Await server termination
}
