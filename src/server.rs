use std::cmp::Ordering;
use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::Router;
use color_eyre::eyre::OptionExt;

use crate::config::Config;
use crate::gradescope::course::Assignment;
use crate::gradescope::home::CourseInfo;

type AppState = (Config, HashMap<u32, (CourseInfo, Vec<Assignment>)>);

pub async fn start(courses: AppState) {
    let app = Router::new().route("/courses/:id.ics", get(course_ics)).with_state(courses);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn course_ics(Path(id): Path<u32>, State((config, courses)): State<AppState>) -> String {
    // TODO: Fix unwraps

    let (course, assignments) =
        courses.get(&id).ok_or_eyre("Could not find course with that ID").unwrap();

    config.course_to_ical_calendar(course, assignments).unwrap().to_string()
}

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

// async fn print_things(State((_config, courses)): State<AppState>) -> String {
//     // println!("Term: {}", term.name);
//     // println!("====================");

//     for (course, assignments) in courses.values() {
//         println!();
//         println!("Course: {} - {}", course.shortname, course.name);
//         println!("----------------------------------------");

//         let body = client.get(course.url).send().await?.text().await?;
//         let document = scraper::Html::parse_document(&body);

//         for assignment in assignments {
//             print!(
//                 "{}\n    {}\n        Released: {}\n        Due:      {}",
//                 assignment.name,
//                 assignment.status,
//                 eos::format_dt!("%a, %b %#d, %Y, %#I:%M %p", assignment.release_date),
//                 eos::format_dt!("%a, %b %#d, %Y, %#I:%M %p", assignment.due_date),
//             );

//             if let SubmissionStatus::Warning(_) = assignment.status {
//                 print!(" ({})", pretty_print_interval(assignment.due_date - now));
//             }

//             println!();
//         }
//     }

//     Ok(())
// }
