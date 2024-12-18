use color_eyre::eyre::{Context, OptionExt, Result};
use reqwest::Url;
use scraper::{ElementRef, Html, Selector};

use super::course::Assignment;
use super::select;
use crate::config::{Config, WhichTerms};

#[derive(Debug, Clone)]
pub struct CourseInfo {
    pub id: u32,
    pub name: String,
    pub shortname: String,
    pub url: Url,
}

#[derive(Debug, Clone)]
pub struct Course {
    pub info: CourseInfo,
    pub assignments: Vec<Assignment>,
}

#[derive(Debug, Clone)]
pub struct TermInfo {
    pub name: String,
    pub courses: Vec<CourseInfo>,
}

impl Config {
    fn parse_course_box(&self, course: ElementRef<'_>) -> Result<CourseInfo> {
        let shortname = select!(course, ".courseBox--shortname")
            .next()
            .ok_or_eyre("Failed to find course shortname")?
            .inner_html();

        let name = select!(course, ".courseBox--name")
            .next()
            .ok_or_eyre("Failed to find course name")?
            .inner_html();

        let url = Url::options()
            .base_url(Some(&self.gradescope_base_url))
            .parse(course.attr("href").ok_or_eyre("Failed to find course URL")?)
            .wrap_err("Failed to parse course URL")?;

        let id = url
            .path()
            .rsplit_once("/")
            .ok_or_eyre("Failed to find course ID in URL")?
            .1
            .parse()
            .wrap_err("Failed to parse course ID from URL")?;

        Ok(CourseInfo { id, name, shortname, url })
    }

    fn parse_courses_for_term(&self, courses_for_term: ElementRef<'_>) -> Result<Vec<CourseInfo>> {
        select!(courses_for_term, ".courseBox:not(.courseBox-new)")
            .map(|elt| self.parse_course_box(elt))
            .collect()
    }

    fn parse_term(&self, courses_for_term: ElementRef<'_>) -> Result<TermInfo> {
        let name = courses_for_term
            .prev_sibling()
            .and_then(ElementRef::wrap)
            .ok_or_eyre("Failed to find term name")?
            .inner_html();

        let courses = self.parse_courses_for_term(courses_for_term)?;

        Ok(TermInfo { name, courses })
    }

    fn parse_course_list(&self, course_list: ElementRef<'_>) -> Vec<Result<TermInfo>> {
        select!(course_list, ".courseList--term + .courseList--coursesForTerm")
            .map(|elt| self.parse_term(elt))
            .collect()
    }

    pub fn parse_home_page(&self, page: Html) -> Result<Vec<TermInfo>> {
        let selector = Selector::parse(".pageHeading + .courseList").unwrap();

        let mut terms = page
            .select(&selector)
            .filter_map(|list| {
                list.prev_sibling()
                    .and_then(ElementRef::wrap)
                    .map(|heading| heading.inner_html() != "Instructor Courses")
                    .and_then(|do_include| do_include.then_some(list))
            })
            .flat_map(|elt| self.parse_course_list(elt));

        match &self.which_terms {
            WhichTerms::All => terms.collect(),
            WhichTerms::MostRecentOnly => terms.next().into_iter().collect(),
            WhichTerms::These(term_names) => terms
                .filter(|term| term.as_ref().is_ok_and(|term| term_names.contains(&term.name)))
                .collect(),
        }
    }
}
