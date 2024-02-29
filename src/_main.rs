use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Sha3};
use std::sync::{Mutex, Arc};
use std::collections::HashMap;

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

// Handler function for receiving URL data from the client, shortening it, and dispatching both URLs back
async fn receive_url(req_body: web::Json<UrlData>) -> HttpResponse {
    // Extract the URL from the JSON data
    let original_url = &req_body.url;

    // Check if the original URL already exists in storage
    let mut storage = SHORTENED_URLS.lock().unwrap();
    if let Some((shortened_url, request_count)) = storage.get(original_url) {
        return HttpResponse::Ok().json(ResponseData {
            original_url: original_url.clone(),
            shortened_url: shortened_url.clone(),
            request_count: *request_count,
        });
    }

    // Generate a short URL identifier using the original URL
    let shortened_url = shorten_url(original_url);

    // Store the original and shortened URLs along with the request count
    storage.insert(original_url.clone(), (shortened_url.clone(), 0));

    // Construct the response data with both original and shortened URLs
    let response_data = ResponseData {
        original_url: original_url.clone(),
        shortened_url: shortened_url.clone(),
        request_count: 0,
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
    if let Some((original_url, _)) = storage.get(short_url) {
        // Increment the request count
        let mut storage = SHORTENED_URLS.lock().unwrap();
        if let Some((_shortened_url, request_count)) = storage.get_mut(original_url) {
            *request_count += 1;
        }
        
        return HttpResponse::TemporaryRedirect()
            .append_header(("Location", original_url.clone()))
            .finish();
    }

    HttpResponse::NotFound().finish()
}

// Handler function for default URL on GET request
async fn default_url() -> HttpResponse {
    let default_url = "https://example.com".to_string(); // Change this to your default URL
    HttpResponse::Ok().body(default_url)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            // Define a route for handling POST requests
            .route("/send-url", web::post().to(receive_url))
            // Define a route for redirection
            .route("/{short_url}", web::get().to(redirect_to_original))
            // Define a route for the default URL
            .route("/", web::get().to(default_url))
    })
    .bind("127.0.0.1:8080")? // Bind the server to the local address and port 8080
    .run() // Start the server
    .await // Await server termination
}

/*use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};
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
}

// Global storage for shortened URLs (in-memory)
static mut SHORTENED_URLS: Vec<(String, String)> = Vec::new();

// Handler function for receiving URL data from the client, shortening it, and dispatching both URLs back
async fn receive_url(req_body: web::Json<UrlData>) -> HttpResponse {
    // Extract the URL from the JSON data
    let original_url = &req_body.url;

    // Generate a short URL identifier using the original URL
    let shortened_url = shorten_url(original_url);

    // Store the original and shortened URLs
    unsafe {
        SHORTENED_URLS.push((original_url.clone(), shortened_url.clone()));
    }

    // Construct the response data with both original and shortened URLs
    let response_data = ResponseData {
        original_url: original_url.clone(),
        shortened_url: shortened_url.clone(),
    };

    // Return the response with the original and shortened URLs
    HttpResponse::Ok().json(response_data)
}

// Function to generate a short URL identifier from the original URL
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

    // Prepend the hash to the base URL of the server
    format!("https://freedns.afraid.org/{}", short_url)
}


// Handler function for redirecting to the original URL
async fn redirect_to_original(req: HttpRequest) -> HttpResponse {
    let short_url = req.match_info().get("short_url").unwrap_or("");
    
    // Find the original URL corresponding to the short URL
    let original_url = unsafe {
        SHORTENED_URLS.iter()
            .find(|&&(_, ref shortened)| *shortened == short_url)
            .map(|&(ref original, _)| original.clone())
    };

    match original_url {
        Some(url) => HttpResponse::TemporaryRedirect()
            .header("Location", url)
            .finish(),
        None => HttpResponse::NotFound().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            // Define a route for handling POST requests
            .route("/send-url", web::post().to(receive_url))
            // Define a route for handling redirection
            .route("/{short_url}", web::get().to(redirect_to_original))
    })
    .bind("127.0.0.1:8080")? // Bind the server to the local address and port 8080
    .run() // Start the server
    .await // Await server termination
}
*/