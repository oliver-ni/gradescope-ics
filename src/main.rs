mod config;
mod gradescope;
mod ical;
mod server;

use std::collections::HashMap;

use color_eyre::eyre::Result;
use reqwest::header;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::read_config()?;

    let mut headers = header::HeaderMap::new();
    headers.insert(header::COOKIE, header::HeaderValue::from_str(&config.gradescope_cookie)?);
    let client = reqwest::Client::builder().default_headers(headers).build()?;

    let body = client.get(config.gradescope_base_url.clone()).send().await?.text().await?;
    let document = scraper::Html::parse_document(&body);

    let mut courses = HashMap::new();

    for term in config.parse_home_page(document)? {
        for course in term.courses {
            let body = client.get(course.url.clone()).send().await?.text().await?;
            let document = scraper::Html::parse_document(&body);
            let assignments = config.parse_course_page(document)?;

            courses.insert(course.id, (course, assignments));
        }
    }

    server::start((config, courses)).await;

    Ok(())
}
