use chrono::prelude::*;
use chrono::Duration;
use newsapi::api::NewsAPIClient;
use newsapi::constants::{Category, Language, SortMethod};
use newsapi::payload::article::Articles;

use dotenv::dotenv;
use std::env;

fn main() {
    // Load environment variables from the .env file
    dotenv().ok();

    // Retrieve environment variables
    let key = env::var("NEWSAPI_KEY").expect("API_KEY must be set");

    let start_timestamp = Utc::now() - Duration::days(10);
    let end_timestamp = Utc::now();

    // create a client
    let mut c = NewsAPIClient::new(key);

    c
        // Search German news sources for articles
        .language(Language::German)
        // In the last ten days
        .from(&start_timestamp)
        .to(&end_timestamp)
        // For articles that contain 'Trump' and 'America'
        .query("Trump America")
        // And are categorized as 'general'
        .category(Category::General)
        // Sort by the most popular articles
        .sort_by(SortMethod::Popularity)
        // get articles from the everything endpoint - this builds the url
        .everything();

    // debug print the current client status - you can see the URL that will be sent to the API
    println!("{:?}", c);

    // fire off a request to the endpoint and deserialize the results into an Article struct
    let articles = c.send_sync::<Articles>().unwrap();

    // print the results to the terminal
    println!("{:?}", articles);

    // access article status
    let status = articles.status;
    println!("{}", status);
}