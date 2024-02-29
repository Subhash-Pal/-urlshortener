use actix_web::{web, App, HttpServer, HttpResponse,HttpRequest};
use serde::{Serialize};
use std::sync::{Mutex};
use std::collections::HashMap;

// Define a structure to hold the URL data sent by the client
#[derive(Debug, serde::Deserialize)]
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
    // You can implement your own logic here to generate a short URL
    // For simplicity, let's just return the original URL
    original_url.to_string()
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

// Define a structure to hold the response data for the metrics API
#[derive(Debug, Serialize)]
struct MetricsData {
    domain: String,
    count: u32,
}

// Handler function for the metrics API
async fn get_metrics() -> HttpResponse {
    // Lock the storage to access the shortened URLs and their request counts
    let storage = SHORTENED_URLS.lock().unwrap();

    // Create a HashMap to store the counts for each domain
    let mut domain_counts: HashMap<String, u32> = HashMap::new();

    // Iterate through the storage and count the occurrences of each domain
    for (url, entry) in storage.iter() {
        let (_shortened_url, count) = entry;
        // Parse the domain from the URL
        let domain = match url.split_once("://") {
            Some((_, remainder)) => {
                match remainder.split_once("/") {
                    Some((domain, _)) => domain.to_string(),
                    None => continue, // Skip URLs without a domain
                }
            }
            None => continue, // Skip invalid URLs
        };

        // Update the count for the domain
        *domain_counts.entry(domain).or_insert(0) += *count;
    }

    // Sort the domain counts by their counts in descending order
    let mut sorted_counts: Vec<_> = domain_counts.iter().collect();
    sorted_counts.sort_by_key(|&(_, count)| std::cmp::Reverse(*count));

    // Get the top 3 domain names with the highest counts
    let top_domains: Vec<_> = sorted_counts.iter().take(3).map(|&(domain, &count)| MetricsData {
        domain: domain.clone(),
        count,
    }).collect();

    // Return the top domains as JSON response
    HttpResponse::Ok().json(top_domains)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            // Define a route for handling POST requests
            .route("/send-url", web::post().to(receive_url))
            // Define a route for redirection
            .route("/redirect/{short_url}", web::get().to(redirect_to_original))
            // Define a route for the metrics API
            .route("/metrics", web::get().to(get_metrics))
    })
    .bind("127.0.0.1:8080")? // Bind the server to the local address and port 8080
    .run() // Start the server
    .await // Await server termination
}
