# Terminal News Reader

Terminal News Reader is a Rust application that fetches news articles using the NewsAPI and displays them in a terminal-based interface using the `ratatui` library. It allows you to browse through the latest news headlines, read the content of selected articles, and mark articles as read or unread.

## Features

- Fetches news articles using the NewsAPI.
- Displays headlines in a terminal-based interface.
- Allows navigation through the list of headlines.
- Fetches and displays the full content of selected articles.
- Marks articles as read or unread.

## Dependencies

- `chrono`: Date and time library.
- `color-eyre`: Error handling library.
- `crossterm`: Library for handling terminal input/output.
- `dotenv`: Library for loading environment variables from a `.env` file.
- `log`: Logging library.
- `newsapi`: Library for interacting with the NewsAPI.
- `ratatui`: Terminal UI library.
- `reqwest`: HTTP client library.
- `scraper`: HTML parsing library.
- `serde`: Serialization/deserialization library.
- `serde_derive`: Macros for `serde`.
- `serde_json`: JSON serialization/deserialization.
- `simplelog`: Simple logging library.
- `tokio`: Asynchronous runtime.
- `url`: Library for URL parsing and manipulation.
- `readability`: Library for extracting readable content from web pages.

## Installation

1. Clone the repository:
    ```sh
    git clone https://github.com/yourusername/terminal-news-reader.git
    cd terminal-news-reader
    ```

2. Create a `.env` file in the project root and add your [NewsAPI key](https://newsdata.io/):
    ```env
    NEWSAPI_KEY=your_newsapi_key
    ```

3. Build the project:
    ```sh
    cargo build
    ```

4. Run the project:
    ```sh
    cargo run --bin article
    ```

## Usage

- Use the arrow keys to navigate through the list of headlines.
- Press Enter or Right to mark an article as read/unread.
- Press Esc or `q` to exit the application.

## How It Works

- The application initializes logging and the terminal interface.
- It fetches the latest news articles from the NewsAPI.
- The articles are displayed in a scrollable list in the terminal.
- Users can navigate through the list, read full articles, and mark them as read or unread.
- The application uses `ratatui` for the terminal UI, `reqwest` for making HTTP requests, `scraper` for parsing HTML, and `readability` for extracting the main content from articles.

## Contribution

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## License

This project is licensed under the MIT License.
