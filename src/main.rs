mod config;
mod gradescope;

use std::cmp::Ordering;

use color_eyre::eyre::Result;
use gradescope::course::SubmissionStatus;
use reqwest::header;

fn pretty_print_interval(interval: eos::Interval) -> String {
    let (prefix, interval, suffix) = match interval.cmp(&eos::Interval::ZERO) {
        Ordering::Equal => ("", interval, ""),
        Ordering::Greater => ("in ", interval, ""),
        Ordering::Less => ("", -interval, " ago"),
    };

    let fmt = match (interval.months(), interval.days(), interval.hours(), interval.minutes()) {
        (0, 0, 0, 0) => "now".to_owned(),
        (0, 0, 0, minutes) => format!("{} minutes", minutes),
        (0, 0, hours, 0) => format!("{} hours", hours),
        (0, 0, hours, minutes) => format!("{} hours, {} minutes", hours, minutes),
        (0, days, 0, _) => format!("{} days", days),
        (0, days, hours, _) => format!("{} days, {} hours", days, hours),
        (months, 0, _, _) => format!("{} months", months),
        (months, days, _, _) => format!("{} months, {} days", months, days),
    };

    format!("{}{}{}", prefix, fmt, suffix)
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::read_config()?;

    let mut headers = header::HeaderMap::new();
    headers.insert(header::COOKIE, header::HeaderValue::from_str(&config.gradescope_cookie)?);
    let client = reqwest::Client::builder().default_headers(headers).build()?;

    let body = client.get(config.gradescope_base_url.clone()).send().await?.text().await?;
    let document = scraper::Html::parse_document(&body);

    let now = eos::DateTime::utc_now();

    for term in config.parse_home_page(document)? {
        println!("Term: {}", term.name);
        println!("====================");

        for course in term.courses {
            println!();
            println!("Course: {} - {}", course.shortname, course.name);
            println!("----------------------------------------");

            let body = client.get(course.url).send().await?.text().await?;
            let document = scraper::Html::parse_document(&body);
            let assignments = config.parse_course_page(document)?;

            for assignment in assignments {
                print!(
                    "{}\n    {}\n        Released: {}\n        Due:      {}",
                    assignment.name,
                    assignment.status,
                    eos::format_dt!("%a, %b %#d, %Y, %#I:%M %p", assignment.release_date),
                    eos::format_dt!("%a, %b %#d, %Y, %#I:%M %p", assignment.due_date),
                );

                if let SubmissionStatus::Warning(_) = assignment.status {
                    print!(" ({})", pretty_print_interval(assignment.due_date - now));
                }

                println!();
            }
        }
    }

    Ok(())
}
