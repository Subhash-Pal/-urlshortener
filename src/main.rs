use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex};
use std::collections::HashMap;
use tiny_keccak::{Hasher, Sha3};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::io::BufRead;


// Define a structure to hold the URL data sent by the client
//#[derive(Debug, Deserialize)]
#[derive(Debug, Deserialize, Serialize)] // Add Serialize here
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

    // Clone the storage before acquiring the lock
    let cloned_storage = {
        let storage = SHORTENED_URLS.lock().unwrap();
        storage.clone() // Clone the storage
    };

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

    // Save the updated storage to the file
    if let Err(err) = save_shortened_urls(&cloned_storage) {
        eprintln!("Failed to save shortened URLs data: {}", err);
    }

    // Construct the response data with both original and shortened URLs
    let response_data = ResponseData {
        original_url: original_url.clone(),
        shortened_url: shortened_url.clone(), // Use the shortened URL from storage
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

// Function to save the shortened URLs data to a file
fn save_shortened_urls(storage: &HashMap<String, (String, u32)>) -> std::io::Result<()> {
    let file = File::create("shortened_urls.txt")?;
    let mut writer = BufWriter::new(file);

    for (url, (shortened_url, count)) in storage.iter() {
        writeln!(writer, "{}:{}:{}", url, shortened_url, count)?;
    }

    Ok(())
}

// Function to load the shortened URLs data from a file
// Function to load the shortened URLs data from a file
fn load_shortened_urls() -> std::io::Result<HashMap<String, (String, u32)>> {
    let file = match File::open("shortened_urls.txt") {
        Ok(file) => file,
        Err(_) => return Ok(HashMap::new()), // Return an empty HashMap if the file doesn't exist
    };
    let reader = BufReader::new(file);

    let mut storage = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<_> = line.split(':').collect();
        if parts.len() == 3 {
            let url = parts[0].to_string();
            let shortened_url = parts[1].to_string();
            let count = parts[2].parse().unwrap_or(0);
            storage.insert(url, (shortened_url, count));
        }
    }

    Ok(storage)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load the shortened URLs data from the file
    let initial_storage = load_shortened_urls().unwrap_or_else(|err| {
        eprintln!("Failed to load shortened URLs data: {}", err);
        HashMap::new()
    });

    // Initialize the global storage with the loaded data
    *SHORTENED_URLS.lock().unwrap() = initial_storage;

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

/*#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shorten_url() {
        // Test with a sample original URL
        let original_url = "https://www.example.com/some/long/url/to/test";
        let shortened_url = shorten_url(original_url);
        // Assert that the shortened URL is not empty
        assert!(!shortened_url.is_empty());
    }
}

*/

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    use actix_web::http::StatusCode;

    #[test]
    async  fn test_shorten_url() {
        // Test with a sample original URL
        let original_url = "https://www.example.com/some/long/url/to/test";
        let shortened_url = shorten_url(original_url);
        // Assert that the shortened URL is not empty
        assert!(!shortened_url.is_empty());
    }

    #[actix_rt::test]
    async fn test_send_url() {
        // Create a test server
        let mut app = test::init_service(
            App::new()
                .route("/send-url", web::post().to(receive_url))
        )
        .await;

        // Send a POST request with JSON data to the /send-url endpoint
        let req = test::TestRequest::post()
            .uri("/send-url")
            .set_json(&UrlData { url: "https://www.example.com".to_string() })
            .to_request();
        let response = test::call_service(&mut app, req).await;

        // Check if the response is successful (HTTP status code 200 OK)
        assert_eq!(response.status(), StatusCode::OK);
    }
}
