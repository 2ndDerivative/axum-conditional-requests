use std::fmt::Display;

use chrono::{MappedLocalTime, TimeZone, Utc};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
/// A functionally equivalent timezone to UTC [as per RFC 7231](https://datatracker.ietf.org/doc/html/rfc7231#section-7.1.1.1)
///
/// The GMT timezone in general isn't like this, as the name isn't even clearly defined,
/// but this is mostly a wrapper for UTC to print as "GMT"
pub struct HttpGmt;
impl TimeZone for HttpGmt {
    type Offset = Utc;

    fn from_offset(_s: &Self::Offset) -> Self {
        Self
    }

    fn offset_from_local_date(&self, _local: &chrono::NaiveDate) -> chrono::MappedLocalTime<Self::Offset> {
        MappedLocalTime::Single(Utc)
    }

    fn offset_from_local_datetime(&self, _local: &chrono::NaiveDateTime) -> chrono::MappedLocalTime<Self::Offset> {
        MappedLocalTime::Single(Utc)
    }

    fn offset_from_utc_date(&self, _utc: &chrono::NaiveDate) -> Self::Offset {
        Utc
    }

    fn offset_from_utc_datetime(&self, _utc: &chrono::NaiveDateTime) -> Self::Offset {
        Utc
    }
}
impl Display for HttpGmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GMT")
    }
}
impl From<Utc> for HttpGmt {
    fn from(_: Utc) -> Self {
        Self
    }
}
impl From<HttpGmt> for Utc {
    fn from(_: HttpGmt) -> Self {
        Utc
    }
}
