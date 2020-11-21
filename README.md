# JSON Feed Model

[JSON Feed][jsonfeed] Model provides types which can be used to manipulate JSON Feed
data.

The crate is basically a [newtype][newtype] wrapper around [Serde JSON][serde_json]'s `Map` type and provides methods
to JSON Feed properties.

* [Latest API Documentation][api_docs]

## Installation

By default, features which depend on the Rust `std` library are included.

```toml
[dependencies]
json-feed-model = "0.1"
```

### Alloc Only

If the host environment has an allocator but does not have access to the Rust `std` library:

```toml
[dependencies]
json-feed-model = { version = "0.1", default-features = false, features = ["alloc"]}
```

## Examples

The following example shows how to read properties.

```rust
use json_feed_model::{Feed, ItemRef, Version};

let json = serde_json::json!({
    "version": "https://jsonfeed.org/version/1.1",
    "title": "Lorem ipsum dolor sit amet.",
    "home_page_url": "https://example.org/",
    "feed_url": "https://example.org/feed.json",
    "items": [
        {
            "id": "cd7f0673-8e81-4e13-b273-4bd1b83967d0",
            "content_text": "Aenean tristique dictum mauris, et.",
            "url": "https://example.org/aenean-tristique"
        },
        {
            "id": "2bcb497d-c40b-4493-b5ae-bc63c74b48fa",
            "content_html": "Vestibulum non magna vitae tortor.",
            "url": "https://example.org/vestibulum-non"
        }
    ]
});

let feed = json_feed_model::from_value(json)?;

assert!(feed.is_valid(&Version::Version1_1));

assert_eq!(feed.version()?, Some(json_feed_model::VERSION_1_1));
assert_eq!(feed.title()?, Some("Lorem ipsum dolor sit amet."));
assert_eq!(feed.home_page_url()?, Some("https://example.org/"));
assert_eq!(feed.feed_url()?, Some("https://example.org/feed.json"));

let items: Option<Vec<ItemRef>> = feed.items()?;
assert!(items.is_some());
let items: Vec<ItemRef> = items.unwrap();
assert_eq!(items.len(), 2);

assert_eq!(items[0].id()?, Some("cd7f0673-8e81-4e13-b273-4bd1b83967d0"));
assert_eq!(
    items[0].content_text()?,
    Some("Aenean tristique dictum mauris, et.")
);
assert_eq!(
    items[0].url()?,
    Some("https://example.org/aenean-tristique")
);

assert_eq!(items[1].id()?, Some("2bcb497d-c40b-4493-b5ae-bc63c74b48fa"));
assert_eq!(
    items[1].content_html()?,
    Some("Vestibulum non magna vitae tortor.")
);
assert_eq!(items[1].url()?, Some("https://example.org/vestibulum-non"));
```

### Custom Extension

The following example uses a custom trait to write and then read a custom extension.
It also shows a simple way to use `serde_json` to write the JSON Feed. See
`serde_json` for other serialization methods.

```rust
use json_feed_model::{Feed, Item, Version};
use serde_json::Value;

trait ExampleExtension {
    fn example(&self) -> Result<Option<&str>, json_feed_model::Error>;

    fn set_example<T>(&mut self, value: T) -> Option<Value>
    where
        T: ToString;
}

impl ExampleExtension for Feed {
    fn example(&self) -> Result<Option<&str>, json_feed_model::Error> {
        self.as_map().get("_example").map_or_else(
            || Ok(None),
            |value| match value {
                Value::String(s) => Ok(Some(s.as_str())),
                _ => Err(json_feed_model::Error::UnexpectedType),
            },
        )
    }

    fn set_example<T>(&mut self, value: T) -> Option<Value>
    where
        T: ToString,
    {
        self.as_map_mut()
            .insert(String::from("_example"), Value::String(value.to_string()))
    }
}

let mut feed = Feed::new();
feed.set_version(Version::Version1_1);
feed.set_title("Lorem ipsum dolor sit amet.");

feed.set_example("123456");

let mut item = Item::new();
item.set_id("2bcb497d-c40b-4493-b5ae-bc63c74b48fa");
item.set_content_text("Vestibulum non magna vitae tortor.");
item.set_url("https://example.org/vestibulum-non");

feed.set_items(vec![item]);

assert!(feed.is_valid(&Version::Version1_1));

let expected_json = serde_json::json!({
    "version": "https://jsonfeed.org/version/1.1",
    "title": "Lorem ipsum dolor sit amet.",
    "_example": "123456",
    "items": [
        {
            "id": "2bcb497d-c40b-4493-b5ae-bc63c74b48fa",
            "content_text": "Vestibulum non magna vitae tortor.",
            "url": "https://example.org/vestibulum-non",
        }
    ]
});
assert_eq!(feed, json_feed_model::from_value(expected_json)?);

assert_eq!(feed.example()?, Some("123456"));

let output = serde_json::to_string(&feed);
assert!(output.is_ok());
```

## License

Licensed under either of [Apache License, Version 2.0][license_apache] or [MIT
License][license_mit] at your option.

### Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[license_apache]: LICENSE-APACHE
[license_mit]: LICENSE-MIT
[jsonfeed]: https://jsonfeed.org/
[newtype]: https://doc.rust-lang.org/rust-by-example/generics/new_types.html
[serde_json]: https://github.com/serde-rs/json
[api_docs]: https://docs.rs/json-feed-model/
