use chrono::{DateTime, TimeZone, Utc};
use std::{str::FromStr, convert::TryFrom};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(transparent))]
pub struct Schedule {
    /// List of [`cron::Schedule`]
    #[cfg_attr(feature = "serde", serde(with = "parsing"))]
    schedules: Vec<cron::Schedule>,
}

impl TryFrom<Vec<String>> for Schedule {
    type Error = cron::error::Error;

    fn try_from(vec: Vec<String>) -> Result<Self, Self::Error> {
        let schedules = vec
            .iter()
            .map(|s| cron::Schedule::from_str(s))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { schedules })
    }
}

impl ToString for Schedule {
    fn to_string(&self) -> String {
        self.schedules
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

impl Schedule {
    /// Create a new schedules with the given list of schedules
    pub fn new(schedules: Vec<cron::Schedule>) -> Self {
        Self { schedules }
    }

    fn next_after<Z: TimeZone>(&self, after: &DateTime<Z>) -> Option<DateTime<Z>> {
        self.schedules
            .iter()
            .filter_map(|s| s.after(after).next())
            .min()
    }

    fn prev_from<Z: TimeZone>(&self, from: &DateTime<Z>) -> Option<DateTime<Z>> {
        self.schedules
            .iter()
            .filter_map(|s| s.after(from).next_back())
            .max()
    }

    pub fn to_strings(&self) -> Vec<String> {
        self.schedules
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    }
}

impl FromStr for Schedule {
    type Err = cron::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let schedules = s
            .split('|')
            .map(|s| cron::Schedule::from_str(s.trim()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { schedules })
    }
}

impl TryFrom<&str> for Schedule {
    type Error = cron::error::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

// Code below is exactly the same as schedule.rs from cron crate
// Last updated: v0.12.0

impl Schedule {
    /// Provides an iterator which will return each DateTime that matches the schedule starting with
    /// the current time if applicable.
    pub fn upcoming<Z>(&self, timezone: Z) -> ScheduleIterator<'_, Z>
    where
        Z: TimeZone,
    {
        self.after(&timezone.from_utc_datetime(&Utc::now().naive_utc()))
    }

    /// The same, but with an iterator with a static ownership
    pub fn upcoming_owned<Z: TimeZone>(&self, timezone: Z) -> OwnedScheduleIterator<Z> {
        self.after_owned(timezone.from_utc_datetime(&Utc::now().naive_utc()))
    }

    /// Like the `upcoming` method, but allows you to specify a start time other than the present.
    pub fn after<Z>(&self, after: &DateTime<Z>) -> ScheduleIterator<'_, Z>
    where
        Z: TimeZone,
    {
        ScheduleIterator::new(self, after)
    }

    /// The same, but with a static ownership.
    pub fn after_owned<Z: TimeZone>(&self, after: DateTime<Z>) -> OwnedScheduleIterator<Z> {
        OwnedScheduleIterator::new(self.clone(), after)
    }

}

pub struct ScheduleIterator<'a, Z>
where
    Z: TimeZone,
{
    schedule: &'a Schedule,
    previous_datetime: Option<DateTime<Z>>,
}
//TODO: Cutoff datetime?

impl<'a, Z> ScheduleIterator<'a, Z>
where
    Z: TimeZone,
{
    fn new(schedule: &'a Schedule, starting_datetime: &DateTime<Z>) -> Self {
        ScheduleIterator {
            schedule,
            previous_datetime: Some(starting_datetime.clone()),
        }
    }
}

impl<'a, Z> Iterator for ScheduleIterator<'a, Z>
where
    Z: TimeZone,
{
    type Item = DateTime<Z>;

    fn next(&mut self) -> Option<DateTime<Z>> {
        let previous = self.previous_datetime.take()?;

        if let Some(next) = self.schedule.next_after(&previous) {
            self.previous_datetime = Some(next.clone());
            Some(next)
        } else {
            None
        }
    }
}

impl<'a, Z> DoubleEndedIterator for ScheduleIterator<'a, Z>
where
    Z: TimeZone,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let previous = self.previous_datetime.take()?;

        if let Some(prev) = self.schedule.prev_from(&previous) {
            self.previous_datetime = Some(prev.clone());
            Some(prev)
        } else {
            None
        }
    }
}

/// A `ScheduleIterator` with a static lifetime.
pub struct OwnedScheduleIterator<Z> where Z: TimeZone {
    schedule: Schedule,
    previous_datetime: Option<DateTime<Z>>
}

impl<Z> OwnedScheduleIterator<Z> where Z: TimeZone {
    pub fn new(schedule: Schedule, starting_datetime: DateTime<Z>) -> Self {
        Self { schedule, previous_datetime: Some(starting_datetime) }
    }
}

impl<Z> Iterator for OwnedScheduleIterator<Z> where Z: TimeZone {
    type Item = DateTime<Z>;

    fn next(&mut self) -> Option<DateTime<Z>> {
        let previous = self.previous_datetime.take()?;

        if let Some(next) = self.schedule.next_after(&previous) {
            self.previous_datetime = Some(next.clone());
            Some(next)
        } else {
            None
        }
    }
}

impl<Z: TimeZone> DoubleEndedIterator for OwnedScheduleIterator<Z> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let previous = self.previous_datetime.take()?;

        if let Some(prev) = self.schedule.prev_from(&previous) {
            self.previous_datetime = Some(prev.clone());
            Some(prev)
        } else {
            None
        }
    }
}
