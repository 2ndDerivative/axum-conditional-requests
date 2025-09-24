# axum-conditional-requests
This small library provides combined extractors and Response wrappers for axum to validate the If-Modified-Since header
and produce an appropriate response.

To maintain the safety of this combination of types, the respective headers should not be overridden after returning.

Where appropriate, the headers are forced to be ignored, which is why IfModifiedSince only implements `OptionalFromRequestParts`

The provided times for last_modified have of course to accurately represent the state of the tracked resource to be useful.

```rust
async fn axum_handler(if_modified_since: Option<IfModifiedSince>) -> MaybeModified<Json<Foo>> {
    let (last_modified, foo) = last_modified_foo();
    // will return 304 if last_modified is before the requested time
    MaybeModified::from_header(if_modified_since, last_modified, foo)
}

fn last_modified_foo() -> (DateTime<Utc>, Foo) {
    unimplemented!()
}

```

This library is WIP and getting updated as my personal requirements for it grow or with external contributions.