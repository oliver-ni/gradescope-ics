use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::Router;
use color_eyre::eyre::OptionExt;

use crate::config::Config;
use crate::gradescope::course::Assignment;
use crate::gradescope::home::Course;

type AppState = (Config, HashMap<u32, (Course, Vec<Assignment>)>);

pub async fn start(courses: AppState) {
    let app = Router::new().route("/courses/:id", get(root)).with_state(courses);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[axum::debug_handler]
async fn root(Path(id): Path<u32>, State((config, courses)): State<AppState>) -> String {
    // TODO: Fix unwraps

    let (course, assignments) =
        courses.get(&id).ok_or_eyre("Could not find course with that ID").unwrap();

    config.course_to_ical_calendar(course, assignments).unwrap().to_string()
}
