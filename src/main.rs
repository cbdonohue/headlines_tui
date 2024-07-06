use std::{error::Error, io};

use crossterm::event::KeyEvent;
use ratatui::{
    backend::Backend,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{
        palette::tailwind::{BLUE, GREEN, SLATE},
        Color, Modifier, Style, Stylize,
    },
    symbols,
    terminal::Terminal,
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
};

use chrono::prelude::*;
use chrono::Duration;
use dotenv::dotenv;
use newsapi::api::NewsAPIClient;
use newsapi::constants::{Category, Language, SortMethod};
use newsapi::payload::article::Articles;
use std::env;

use url::Url;
use readability::extractor;
use reqwest;

#[macro_use] extern crate log;
extern crate simplelog;

use simplelog::*;

use std::fs::File;

const HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;
const COMPLETED_TEXT_FG_COLOR: Color = GREEN.c500;

fn main() -> Result<(), Box<dyn Error>> {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create("my_rust_binary.log").unwrap()),
        ]
    ).unwrap();

    tui::init_error_hooks()?;
    let terminal = tui::init_terminal()?;

    let mut app = App::default();
    app.add_articles(&fetch_news_articles()?.articles);
    app.run(terminal)?;

    tui::restore_terminal()?;
    Ok(())
}

fn fetch_full_article(article: &newsapi::payload::article::Article) -> Result<String, Box<dyn std::error::Error>> {
    let article_html = reqwest::blocking::get(&article.url)?.text()?;

    let article_url = Url::parse(&article.url)?;
    info!("Article URL: {:?}", article_url);

    let readability = extractor::extract(&mut article_html.as_bytes(), &article_url)?;

    Ok(readability.text)
}

fn fetch_news_articles() -> Result<Articles, Box<dyn std::error::Error>> {
    // Load environment variables from the .env file
    dotenv().ok();

    // Retrieve environment variables
    let key = env::var("NEWSAPI_KEY")?;

    let start_timestamp = Utc::now() - Duration::days(10);
    let end_timestamp = Utc::now();

    // Create a client
    let mut c = NewsAPIClient::new(key);

    c.language(Language::German)
        .from(&start_timestamp)
        .to(&end_timestamp)
        .query("Trump America")
        .category(Category::General)
        .sort_by(SortMethod::Popularity)
        .language(Language::English)
        .everything();

    // Debug print the current client status - you can see the URL that will be sent to the API
    info!("{:?}", c);

    // Fire off a request to the endpoint and deserialize the results into an Article struct
    let articles = c.send_sync::<Articles>()?;

    info!("{:?}", articles);

    // Access article status
    let status = &articles.status;
    info!("{}", status);

    Ok(articles)
}

/// This struct holds the current state of the app. In particular, it has the `article_list` field
/// which is a wrapper around `ListState`. Keeping track of the state lets us render the
/// associated widget with its state and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events. Check
/// the drawing logic for items on how to specify the highlighting style for selected items.
struct App {
    should_exit: bool,
    article_list: ArticleList,
}

struct ArticleList {
    items: Vec<ArticleItem>,
    state: ListState,
}

#[derive(Debug)]
struct ArticleItem {
    headline: String,
    info: String,
    status: Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Status {
    Unread,
    Completed,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_exit: false,
            article_list: ArticleList::from_iter([
                (Status::Unread, "Loading...", "Loading..."),
            ]),
        }
    }
}

impl FromIterator<(Status, &'static str, &'static str)> for ArticleList {
    fn from_iter<I: IntoIterator<Item = (Status, &'static str, &'static str)>>(iter: I) -> Self {
        let items = iter
            .into_iter()
            .map(|(status, headline, info)| ArticleItem::new(status, headline, info))
            .collect();
        let state = ListState::default();
        Self { items, state }
    }
}

impl ArticleItem {
    fn new(status: Status, headline: &str, info: &str) -> Self {
        Self {
            status,
            headline: headline.to_string(),
            info: info.to_string(),
        }
    }
}

impl App {
    fn run(&mut self, mut terminal: Terminal<impl Backend>) -> io::Result<()> {
        while !self.should_exit {
            terminal.draw(|f| f.render_widget(&mut *self, f.size()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(())
    }

    fn add_article(&mut self, article: &newsapi::payload::article::Article) {
        // Fetch the full article content
        let content = fetch_full_article(article).unwrap_or_else(|_| "No description available".to_string());

        // Push the new item to the article list
        self.article_list.items.push(ArticleItem::new(
            Status::Unread,
            &article.title,
            &content,
        ));
    }

    fn add_articles(&mut self, articles: &Vec<newsapi::payload::article::Article>) {
        self.article_list.items.clear();
        for article in articles.iter().take(10) {
            self.add_article(article);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            KeyCode::Char('h') | KeyCode::Left => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                self.toggle_status();
            }
            _ => {}
        }
    }

    fn select_none(&mut self) {
        self.article_list.state.select(None);
    }

    fn select_next(&mut self) {
        self.article_list.state.select_next();
    }
    fn select_previous(&mut self) {
        self.article_list.state.select_previous();
    }

    fn select_first(&mut self) {
        self.article_list.state.select_first();
    }

    fn select_last(&mut self) {
        self.article_list.state.select_last();
    }

    /// Changes the status of the selected list item
    fn toggle_status(&mut self) {
        if let Some(i) = self.article_list.state.selected() {
            self.article_list.items[i].status = match self.article_list.items[i].status {
                Status::Completed => Status::Unread,
                Status::Unread => Status::Completed,
            }
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [list_area, item_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

        App::render_header(header_area, buf);
        App::render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        self.render_selected_item(item_area, buf);
    }
}

/// Rendering logic for the app
impl App {
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Terminal News Reader")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom.")
            .centered()
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Headlines").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .article_list
            .items
            .iter()
            .enumerate()
            .map(|(i, article_item)| {
                let color = alternate_colors(i);
                ListItem::from(article_item).bg(color)
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // We need to disambiguate this trait method as both `Widget` and `StatefulWidget` share the
        // same method name `render`.
        StatefulWidget::render(list, area, buf, &mut self.article_list.state);
    }

    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        // We get the info depending on the item's state.
        let info = if let Some(i) = self.article_list.state.selected() {
            match self.article_list.items[i].status {
                Status::Completed => format!("✓ {}", self.article_list.items[i].info),
                Status::Unread => format!("☐ {}", self.article_list.items[i].info),
            }
        } else {
            "Nothing selected...".to_string()
        };

        // We show the list item's info under the list in this paragraph
        let block = Block::new()
            .title(Line::raw("Ariticle").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(HEADER_STYLE)
            .bg(NORMAL_ROW_BG)
            .padding(Padding::horizontal(1));

        // We can now render the item info
        Paragraph::new(info)
            .block(block)
            .fg(TEXT_FG_COLOR)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}

impl From<&ArticleItem> for ListItem<'_> {
    fn from(value: &ArticleItem) -> Self {
        let line = match value.status {
            Status::Unread => Line::styled(format!(" ☐ {}", value.headline), TEXT_FG_COLOR),
            Status::Completed => {
                Line::styled(format!(" ✓ {}", value.headline), COMPLETED_TEXT_FG_COLOR)
            }
        };
        ListItem::new(line)
    }
}

mod tui {
    use std::{io, io::stdout};

    use color_eyre::config::HookBuilder;
    use ratatui::{
        backend::{Backend, CrosstermBackend},
        crossterm::{
            terminal::{
                disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
            },
            ExecutableCommand,
        },
        terminal::Terminal,
    };

    pub fn init_error_hooks() -> color_eyre::Result<()> {
        let (panic, error) = HookBuilder::default().into_hooks();
        let panic = panic.into_panic_hook();
        let error = error.into_eyre_hook();
        color_eyre::eyre::set_hook(Box::new(move |e| {
            let _ = restore_terminal();
            error(e)
        }))?;
        std::panic::set_hook(Box::new(move |info| {
            let _ = restore_terminal();
            panic(info);
        }));
        Ok(())
    }

    pub fn init_terminal() -> io::Result<Terminal<impl Backend>> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        Terminal::new(CrosstermBackend::new(stdout()))
    }

    pub fn restore_terminal() -> io::Result<()> {
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()
    }
}