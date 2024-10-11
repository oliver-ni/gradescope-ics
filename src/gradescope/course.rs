use std::fmt::Display;

use color_eyre::eyre::{Context, OptionExt, Result};
use eos::fmt::format_spec;
use eos::{DateTime, UtcOffset};
use scraper::{ElementRef, Html};

use super::select;
use crate::config::Config;

#[derive(Debug, Clone)]
pub enum SubmissionStatus {
    Score(String),
    Warning(String),
    Complete(String),
}

impl Display for SubmissionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Score(s) => write!(f, "{}", s),
            Self::Warning(s) => write!(f, "\u{26A0} {}", s),
            Self::Complete(s) => write!(f, "\u{2713} {}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub name: String,
    pub status: SubmissionStatus,
    pub release_date: DateTime<UtcOffset>,
    pub due_date: DateTime<UtcOffset>,
}

fn parse_datetime(s: &str) -> Result<DateTime<UtcOffset>, eos::fmt::ParseError> {
    DateTime::parse_from_spec(s, format_spec!("%Y-%m-%d %H:%M:%S %z"))
}

impl Config {
    fn parse_assignment(&self, assignment: ElementRef<'_>) -> Result<Assignment> {
        let name = select!(assignment, ".table--primaryLink")
            .next()
            .and_then(|elt| elt.text().next())
            .map(str::to_owned)
            .ok_or_eyre("Failed to find assignment name")?;

        let status = None
            .or(select!(assignment, ".submissionStatus-complete > .submissionStatus--text")
                .map(|elt| SubmissionStatus::Complete(elt.inner_html()))
                .next())
            .or(select!(assignment, ".submissionStatus-warning > .submissionStatus--text")
                .map(|elt| SubmissionStatus::Warning(elt.inner_html()))
                .next())
            .or(select!(assignment, ".submissionStatus--score")
                .map(|elt| SubmissionStatus::Score(elt.inner_html()))
                .next())
            .ok_or_eyre("Failed to find assignment status")?;

        let release_date = parse_datetime(
            select!(assignment, ".submissionTimeChart--releaseDate")
                .next()
                .and_then(|elt| elt.attr("datetime"))
                .ok_or_eyre("Failed to find assignment release date")?,
        )
        .wrap_err("Failed to parse assignment due date")?;

        let due_date = parse_datetime(
            select!(assignment, ".submissionTimeChart--dueDate")
                .next()
                .and_then(|elt| elt.attr("datetime"))
                .ok_or_eyre("Failed to find assignment due date")?,
        )
        .wrap_err("Failed to parse assignment due date")?;

        Ok(Assignment { name, status, release_date, due_date })
    }

    fn parse_assignment_table(&self, assignment_table: ElementRef<'_>) -> Result<Vec<Assignment>> {
        select!(assignment_table, "tbody > tr").map(|elt| self.parse_assignment(elt)).collect()
    }

    pub fn parse_course_page(&self, page: Html) -> Result<Vec<Assignment>> {
        let table = select!(page, "#assignments-student-table")
            .next()
            .ok_or_eyre("Failed to find assignment table")?;

        self.parse_assignment_table(table)
    }
}
