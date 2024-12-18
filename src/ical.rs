use color_eyre::eyre::{OptionExt, Result};
use eos::UtcOffset;
use icalendar::{Calendar, CalendarDateTime, Component, Event, EventLike};

use crate::config::{Config, CreateEventsFor};
use crate::gradescope::course::Assignment;
use crate::gradescope::home::CourseInfo;

fn eos_datetime_to_ical(dt: eos::DateTime<UtcOffset>) -> Result<CalendarDateTime> {
    Ok(chrono::DateTime::from_timestamp(dt.timestamp().as_seconds(), dt.nanosecond())
        .ok_or_eyre("Failed to convert timestamp to iCal format")?
        .into())
}

impl Config {
    pub fn assignment_to_ical_events(&self, assignment: &Assignment) -> Result<Vec<Event>> {
        let rel_date = eos_datetime_to_ical(assignment.release_date)?;
        let due_date = eos_datetime_to_ical(assignment.due_date)?;
        let rel_date = || rel_date.clone();
        let due_date = || due_date.clone();

        let event = |starts, ends, summary: String| {
            Event::new()
                .summary(&summary)
                .description(&assignment.status.to_string())
                .starts(starts)
                .ends(ends)
                .done()
        };

        let rel_event = || event(rel_date(), rel_date(), format!("{} Released", assignment.name));
        let due_event = || event(due_date(), due_date(), format!("{} Due", assignment.name));
        let active_event = || event(rel_date(), due_date(), assignment.name.clone());

        Ok(match self.create_events_for {
            CreateEventsFor::DueDateOnly => vec![due_event()],
            CreateEventsFor::ReleaseAndDueDates => vec![rel_event(), due_event()],
            CreateEventsFor::DurationAssignmentIsActive => vec![active_event()],
        })
    }

    pub fn course_to_ical_calendar(
        &self,
        course: &CourseInfo,
        assignments: &[Assignment],
    ) -> Result<Calendar> {
        let mut cal = Calendar::new();

        for assignment in assignments {
            for event in self.assignment_to_ical_events(assignment)? {
                cal.push(event);
            }
        }

        Ok(cal.name(&course.shortname).description(&course.name).done())
    }
}
