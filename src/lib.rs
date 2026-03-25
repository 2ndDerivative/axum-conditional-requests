use std::convert::Infallible;

use axum_core::{
    extract::OptionalFromRequestParts,
    response::{AppendHeaders, IntoResponse, IntoResponseParts, ResponseParts},
};
use chrono::{DateTime, TimeZone, Timelike};
use http::{
    HeaderValue, Method, StatusCode,
    header::{IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED},
    request::Parts,
};

use crate::timezone::HttpGmt;
mod timezone;

#[derive(Debug)]
pub struct LastModified(DateTime<HttpGmt>);
impl IntoResponseParts for LastModified {
    type Error = Infallible;

    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        res.headers_mut()
            .insert(LAST_MODIFIED, HeaderValue::from_str(&self.0.to_rfc2822()).unwrap());
        Ok(res)
    }
}
impl IntoResponse for LastModified {
    fn into_response(self) -> axum_core::response::Response {
        AppendHeaders([(LAST_MODIFIED, HeaderValue::from_str(&self.0.to_rfc2822()).unwrap())]).into_response()
    }
}

#[derive(Clone, Copy, Debug)]
/// Only implements OptionalFromRequestParts to force the user
/// to handle the case of ignoring this header,
/// as extracting this header is not supposed to error out, but MUST instead be ignored.
pub struct IfModifiedSince(DateTime<HttpGmt>);
impl<S: Send + Sync> OptionalFromRequestParts<S> for IfModifiedSince {
    type Rejection = Infallible;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Option<Self>, Self::Rejection> {
        if parts.headers.contains_key(IF_NONE_MATCH) {
            return Ok(None);
        }
        if !matches!(parts.method, Method::GET | Method::HEAD) {
            #[cfg(feature = "tracing")]
            tracing::warn!("client sent if-modified-since on a non-GET/HEAD request");
            return Ok(None);
        }
        let Some(header) = parts.headers.get(IF_MODIFIED_SINCE) else {
            return Ok(None);
        };
        let Ok(header_str) = header.to_str() else {
            #[cfg(feature = "tracing")]
            tracing::warn!("Client sent non-utf8 if-modified-since header");
            return Ok(None);
        };
        let Ok(time) = DateTime::parse_from_rfc2822(header_str) else {
            #[cfg(feature = "tracing")]
            tracing::warn!("Client sent invalid date in if-modified-since header");
            return Ok(None);
        };
        Ok(Some(Self(HttpGmt.from_local_datetime(&time.naive_utc()).unwrap())))
    }
}

#[derive(Debug)]
pub struct MaybeUnmodified<T> {
    last_modified: DateTime<HttpGmt>,
    payload: MaybeModifiedPayload<T>,
}
impl<T> MaybeUnmodified<T> {
    pub fn from_header<Tz: TimeZone>(header: Option<IfModifiedSince>, last_modified: DateTime<Tz>, payload: T) -> Self {
        let last_modified: DateTime<HttpGmt> = HttpGmt.from_local_datetime(&last_modified.naive_utc()).unwrap();
        let Some(IfModifiedSince(header_time)) = header else {
            return MaybeUnmodified {
                last_modified,
                payload: MaybeModifiedPayload::New(payload),
            };
        };
        let last_modified = last_modified.with_nanosecond(0).unwrap();
        let header_time = header_time.with_nanosecond(0).unwrap();
        let payload = if last_modified <= header_time {
            MaybeModifiedPayload::NotModified
        } else {
            MaybeModifiedPayload::New(payload)
        };
        MaybeUnmodified { last_modified, payload }
    }
}
impl<T: IntoResponse> IntoResponse for MaybeUnmodified<T> {
    fn into_response(self) -> axum_core::response::Response {
        let header = LastModified(self.last_modified);
        match self.payload {
            MaybeModifiedPayload::NotModified => (StatusCode::NOT_MODIFIED, header).into_response(),
            MaybeModifiedPayload::New(p) => (header, p).into_response(),
        }
    }
}

#[derive(Debug)]
enum MaybeModifiedPayload<T> {
    NotModified,
    New(T),
}
