use reqwest;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::env;
use tokio;
use url::Url;
use readability::extractor;

#[derive(Deserialize, Debug)]
struct Article {
    url: String,
}

#[derive(Deserialize, Debug)]
struct NewsApiResponse {
    articles: Vec<Article>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from the .env file
    dotenv::dotenv().ok();
    
    // Retrieve the News API key from environment variables
    let api_key = env::var("NEWSAPI_KEY")?;
    
    // Build the URL for the News API request
    let url = format!(
        "https://newsapi.org/v2/everything?q=Apple&sortBy=publishedAt&apiKey={}",
        api_key
    );
    
    // Make the request to the News API with the User-Agent header
    let client = reqwest::Client::new();
    let response = client.get(&url)
        .header("User-Agent", "headlines/0.1.0")
        .send()
        .await?
        .text()
        .await?;
    
    // Print the raw JSON response for debugging
    println!("Raw JSON response: {}", response);
    
    // Parse the JSON response
    let news: NewsApiResponse = serde_json::from_str(&response)?;
    
    // Get the first article from the response
    if let Some(first_article) = news.articles.get(9) {
        // Make the request to get the HTML content of the first article
        let mut article_html = reqwest::get(&first_article.url).await?.text().await?;
        
        // Convert the article URL to a Url object
        let article_url = Url::parse(&first_article.url)?;
        
        // Parse the article content using readability
        let readability = extractor::extract(&mut article_html.as_bytes(), &article_url)?;
        
        // Print the article content
        println!("{}", remove_html_tags(&readability.content));
    } else {
        println!("No articles found.");
    }
    
    Ok(())
}

fn remove_html_tags(html_content: &str) -> String {
    // Parse the HTML content
    let document = Html::parse_document(html_content);
    
    // Create a selector for all text nodes
    let selector = Selector::parse("body").unwrap();
    
    // Extract and collect text from the document
    let text = document
        .select(&selector)
        .next()
        .map(|body| body.text().collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|| "".to_string());
    
    text
}
