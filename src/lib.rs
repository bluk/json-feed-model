// Copyright 2022 Bryant Luk
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

use core::str;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{
    collections::BTreeSet,
    string::{String, ToString},
    vec::Vec,
};
#[cfg(feature = "std")]
use std::{
    collections::BTreeSet,
    string::{String, ToString},
    vec::Vec,
};

use serde_json::{Map, Value};

/// Version 1 identifier (for 1.0 feeds)
pub const VERSION_1: &str = "https://jsonfeed.org/version/1";

/// Version 1.1 identifier
pub const VERSION_1_1: &str = "https://jsonfeed.org/version/1.1";

/// A JSON Feed spec version identifier
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Version<'a> {
    /// https://jsonfeed.org/version/1
    Version1,
    /// https://jsonfeed.org/version/1.1
    Version1_1,
    /// An unknown version
    Unknown(&'a str),
}

impl<'a> AsRef<str> for Version<'a> {
    fn as_ref(&self) -> &str {
        match self {
            Version::Version1 => VERSION_1,
            Version::Version1_1 => VERSION_1_1,
            Version::Unknown(v) => v,
        }
    }
}

impl<'a> From<&'a str> for Version<'a> {
    fn from(value: &'a str) -> Self {
        match value {
            VERSION_1 => Version::Version1,
            VERSION_1_1 => Version::Version1_1,
            _ => Version::Unknown(value),
        }
    }
}

impl<'a> core::fmt::Display for Version<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

/// All of the possible crate errors.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// If the JSON value is an unexpected type.
    ///
    /// For instance, if a JSON string is expected but the actual value is a JSON object, then
    /// `UnexpectedType` would be returned as an error.
    UnexpectedType,
    /// If there is an error decoding the JSON.
    SerdeJson(serde_json::Error),
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::SerdeJson(error)
    }
}

macro_rules! get_set_rm_str {
    ($key_expr:expr, $getter:ident, $getter_doc:expr, $setter:ident, $setter_doc:expr, $remover:ident, $remover_doc:expr) => {
        get_set_rm_str!($key_expr, $getter, $getter_doc, $setter, $setter_doc);

        #[doc=$remover_doc]
        pub fn $remover(&mut self) -> Option<Value> {
            self.value.remove($key_expr)
        }
    };

    ($key_expr:expr, $getter:ident, $getter_doc:expr, $setter:ident, $setter_doc:expr) => {
        get_set_rm_str!($key_expr, $getter, $getter_doc);

        #[doc=$setter_doc]
        pub fn $setter<T>(&mut self, value: T) -> Option<Value>
        where
            T: ToString,
        {
            self.value
                .insert(String::from($key_expr), Value::String(value.to_string()))
        }
    };

    ($key_expr:expr, $getter:ident, $getter_doc:expr) => {
        #[doc=$getter_doc]
        pub fn $getter(&self) -> Result<Option<&str>, Error> {
            self.value.get($key_expr).map_or_else(
                || Ok(None),
                |value| match value {
                    Value::String(s) => Ok(Some(s.as_str())),
                    _ => Err(Error::UnexpectedType),
                },
            )
        }
    };
}

macro_rules! get_set_rm_str_array {
    ($key_expr:expr, $getter:ident, $getter_doc:expr, $setter:ident, $setter_doc:expr, $remover:ident, $remover_doc:expr) => {
        get_set_rm_str_array!($key_expr, $getter, $getter_doc, $setter, $setter_doc);

        #[doc=$remover_doc]
        pub fn $remover(&mut self) -> Option<Value> {
            self.value.remove($key_expr)
        }
    };

    ($key_expr:expr, $getter:ident, $getter_doc:expr, $setter:ident, $setter_doc:expr) => {
        get_set_rm_str_array!($key_expr, $getter, $getter_doc);

        #[doc=$setter_doc]
        pub fn $setter<I>(&mut self, values: I) -> Option<Value>
        where
            I: IntoIterator<Item = String>,
        {
            let values: Value = Value::Array(values.into_iter().map(Value::String).collect());
            self.value.insert(String::from($key_expr), values)
        }
    };

    ($key_expr:expr, $getter:ident, $getter_doc:expr) => {
        #[doc=$getter_doc]
        pub fn $getter(&self) -> Result<Option<Vec<&str>>, Error> {
            self.value.get($key_expr).map_or_else(
                || Ok(None),
                |value| match value {
                    Value::Array(arr) => arr
                        .iter()
                        .map(|value| match value {
                            Value::String(s) => Ok(s.as_str()),
                            _ => Err(Error::UnexpectedType),
                        })
                        .collect::<Result<Vec<&str>, Error>>()
                        .map(Some),
                    _ => Err(Error::UnexpectedType),
                },
            )
        }
    };
}

macro_rules! get_set_rm_bool {
    ($key_expr:expr, $getter:ident, $getter_doc:expr, $setter:ident, $setter_doc:expr, $remover:ident, $remover_doc:expr) => {
        get_set_rm_bool!($key_expr, $getter, $getter_doc, $setter, $setter_doc);

        #[doc=$remover_doc]
        pub fn $remover(&mut self) -> Option<Value> {
            self.value.remove($key_expr)
        }
    };

    ($key_expr:expr, $getter:ident, $getter_doc:expr, $setter:ident, $setter_doc:expr) => {
        get_set_rm_bool!($key_expr, $getter, $getter_doc);

        #[doc=$setter_doc]
        pub fn $setter<T>(&mut self, value: bool) -> Option<Value> {
            self.value
                .insert(String::from($key_expr), Value::Bool(value))
        }
    };

    ($key_expr:expr, $getter:ident, $getter_doc:expr) => {
        #[doc=$getter_doc]
        pub fn $getter(&self) -> Result<Option<bool>, Error> {
            self.value.get($key_expr).map_or_else(
                || Ok(None),
                |value| match value {
                    Value::Bool(b) => Ok(Some(*b)),
                    _ => Err(Error::UnexpectedType),
                },
            )
        }
    };
}

macro_rules! get_set_rm_u64 {
    ($key_expr:expr, $getter:ident, $getter_doc:expr, $setter:ident, $setter_doc:expr, $remover:ident, $remover_doc:expr) => {
        get_set_rm_u64!($key_expr, $getter, $getter_doc, $setter, $setter_doc);

        #[doc=$remover_doc]
        pub fn $remover<T>(&mut self) -> Option<Value>
        where
            T: ToString,
        {
            self.value.remove($key_expr)
        }
    };

    ($key_expr:expr, $getter:ident, $getter_doc:expr, $setter:ident, $setter_doc:expr) => {
        get_set_rm_u64!($key_expr, $getter, $getter_doc);

        #[doc=$setter_doc]
        pub fn $setter<T>(&mut self, value: u64) -> Option<Value> {
            self.value.insert(
                String::from($key_expr),
                Value::Number(serde_json::Number::from(value)),
            )
        }
    };

    ($key_expr:expr, $getter:ident, $getter_doc:expr) => {
        #[doc=$getter_doc]
        pub fn $getter(&self) -> Result<Option<u64>, Error> {
            self.value.get($key_expr).map_or_else(
                || Ok(None),
                |value| match value {
                    Value::Number(n) => {
                        if let Some(n) = n.as_u64() {
                            Ok(Some(n))
                        } else {
                            Err(Error::UnexpectedType)
                        }
                    }
                    _ => Err(Error::UnexpectedType),
                },
            )
        }
    };
}

macro_rules! get_ref_get_ref_mut_set_rm_obj {
    ($key_expr:expr, $getter_ref:ident, $getter_ref_type:ty, $getter_ref_new:expr, $getter_ref_doc:expr,
        $getter_ref_mut:ident, $getter_ref_mut_type:ty, $getter_ref_mut_new:expr, $getter_ref_mut_doc:expr,
        $setter:ident, $setter_type:ty, $setter_doc:expr,
        $remover:ident, $remover_doc:expr
    ) => {
        get_ref_get_ref_mut_set_rm_obj!(
            $key_expr,
            $getter_ref,
            $getter_ref_type,
            $getter_ref_new,
            $getter_ref_doc
        );

        #[doc=$getter_ref_mut_doc]
        pub fn $getter_ref_mut(&mut self) -> Result<Option<$getter_ref_mut_type>, Error> {
            self.value.get_mut($key_expr).map_or_else(
                || Ok(None),
                |value| match value {
                    Value::Object(obj) => Ok(Some($getter_ref_mut_new(obj))),
                    _ => Err(Error::UnexpectedType),
                },
            )
        }

        #[doc=$setter_doc]
        pub fn $setter(&mut self, value: $setter_type) -> Option<Value> {
            self.value
                .insert(String::from($key_expr), Value::Object(value.value))
        }

        #[doc=$remover_doc]
        pub fn $remover(&mut self) -> Option<Value> {
            self.value.remove($key_expr)
        }
    };
    ($key_expr:expr, $getter_ref:ident, $getter_ref_type:ty, $getter_ref_new:expr, $getter_ref_doc:expr) => {
        #[doc=$getter_ref_doc]
        pub fn $getter_ref(&self) -> Result<Option<$getter_ref_type>, Error> {
            self.value.get($key_expr).map_or_else(
                || Ok(None),
                |value| match value {
                    Value::Object(obj) => Ok(Some($getter_ref_new(obj))),
                    _ => Err(Error::UnexpectedType),
                },
            )
        }
    };
}

macro_rules! get_ref_get_ref_mut_set_rm_obj_array {
    ($key_expr:expr, $getter_ref:ident, $getter_ref_type:ty, $getter_ref_new:expr, $getter_ref_doc:expr,
        $getter_ref_mut:ident, $getter_ref_mut_type:ty, $getter_ref_mut_new:expr, $getter_ref_mut_doc:expr,
        $setter:ident, $setter_type:ty, $setter_doc:expr,
        $remover:ident, $remover_doc:expr
    ) => {
        get_ref_get_ref_mut_set_rm_obj_array!(
            $key_expr,
            $getter_ref,
            $getter_ref_type,
            $getter_ref_new,
            $getter_ref_doc
        );

        #[doc=$getter_ref_mut_doc]
        pub fn $getter_ref_mut(&mut self) -> Result<Option<Vec<$getter_ref_mut_type>>, Error> {
            self.value.get_mut($key_expr).map_or_else(
                || Ok(None),
                |value| match value {
                    Value::Array(arr) => arr
                        .iter_mut()
                        .map(|value| match value {
                            Value::Object(obj) => Ok($getter_ref_mut_new(obj)),
                            _ => Err(Error::UnexpectedType),
                        })
                        .collect::<Result<Vec<$getter_ref_mut_type>, Error>>()
                        .map(Some),
                    _ => Err(Error::UnexpectedType),
                },
            )
        }

        #[doc=$setter_doc]
        pub fn $setter<I>(&mut self, items: I) -> Option<Value>
        where
            I: IntoIterator<Item = $setter_type>,
        {
            let items: Value =
                Value::Array(items.into_iter().map(|a| Value::Object(a.value)).collect());
            self.value.insert(String::from($key_expr), items)
        }

        #[doc=$remover_doc]
        pub fn $remover(&mut self) -> Option<Value> {
            self.value.remove($key_expr)
        }
    };
    ($key_expr:expr, $getter_ref:ident, $getter_ref_type:ty, $getter_ref_new:expr, $getter_ref_doc:expr) => {
        #[doc=$getter_ref_doc]
        pub fn $getter_ref(&self) -> Result<Option<Vec<$getter_ref_type>>, Error> {
            self.value.get($key_expr).map_or_else(
                || Ok(None),
                |value| match value {
                    Value::Array(arr) => arr
                        .iter()
                        .map(|value| match value {
                            Value::Object(obj) => Ok($getter_ref_new(obj)),
                            _ => Err(Error::UnexpectedType),
                        })
                        .collect::<Result<Vec<$getter_ref_type>, Error>>()
                        .map(Some),
                    _ => Err(Error::UnexpectedType),
                },
            )
        }
    };
}

macro_rules! json_feed_prop_decl {
    () => {};
    ([str_prop, $key_expr:expr, $getter:ident, $getter_doc:expr, $setter:ident, $setter_doc:expr, $remover:ident, $remover_doc:expr] $(,$rest:tt)*) => {
        get_set_rm_str!($key_expr, $getter, $getter_doc, $setter, $setter_doc, $remover, $remover_doc);
        json_feed_prop_decl!($($rest),*);
    };
    ([str_array_prop, $key_expr:expr, $getter:ident, $getter_doc:expr, $setter:ident, $setter_doc:expr, $remover:ident, $remover_doc:expr] $(,$rest:tt)*) => {
        get_set_rm_str_array!($key_expr, $getter, $getter_doc, $setter, $setter_doc, $remover, $remover_doc);
        json_feed_prop_decl!($($rest),*);
    };
    ([u64_prop, $key_expr:expr, $getter:ident, $getter_doc:expr, $setter:ident, $setter_doc:expr, $remover:ident, $remover_doc:expr] $(,$rest:tt)*) => {
        get_set_rm_u64!($key_expr, $getter, $getter_doc, $setter, $setter_doc, $remover, $remover_doc);
        json_feed_prop_decl!($($rest),*);
    };
    ([bool_prop, $key_expr:expr, $getter:ident, $getter_doc:expr, $setter:ident, $setter_doc:expr, $remover:ident, $remover_doc:expr] $(,$rest:tt)*) => {
        get_set_rm_bool!($key_expr, $getter, $getter_doc, $setter, $setter_doc, $remover, $remover_doc);
        json_feed_prop_decl!($($rest),*);
    };
    ([obj_prop, $key_expr:expr, $getter_ref:ident, $getter_ref_type:ty, $getter_ref_new:expr, $getter_ref_doc:expr, $getter_ref_mut:ident, $getter_ref_mut_type:ty, $getter_ref_mut_new:expr, $getter_ref_mut_doc:expr, $setter:ident, $setter_type:ty, $setter_doc:expr, $remover:ident, $remover_doc:expr] $(,$rest:tt)*) => {
        get_ref_get_ref_mut_set_rm_obj!($key_expr, $getter_ref, $getter_ref_type, $getter_ref_new, $getter_ref_doc, $getter_ref_mut, $getter_ref_mut_type, $getter_ref_mut_new, $getter_ref_mut_doc, $setter, $setter_type, $setter_doc, $remover, $remover_doc);
        json_feed_prop_decl!($($rest),*);
    };
    ([obj_array_prop, $key_expr:expr, $getter_ref:ident, $getter_ref_type:ty, $getter_ref_new:expr, $getter_ref_doc:expr, $getter_ref_mut:ident, $getter_ref_mut_type:ty, $getter_ref_mut_new:expr, $getter_ref_mut_doc:expr, $setter:ident, $setter_type:ty, $setter_doc:expr, $remover:ident, $remover_doc:expr] $(,$rest:tt)*) => {
        get_ref_get_ref_mut_set_rm_obj_array!($key_expr, $getter_ref, $getter_ref_type, $getter_ref_new, $getter_ref_doc, $getter_ref_mut, $getter_ref_mut_type, $getter_ref_mut_new, $getter_ref_mut_doc, $setter, $setter_type, $setter_doc, $remover, $remover_doc);
        json_feed_prop_decl!($($rest),*);
    };
}

macro_rules! json_feed_prop_read_only_decl {
    () => {};
    ([str_prop, $key_expr:expr, $getter:ident, $getter_doc:expr, $setter:ident, $setter_doc:expr, $remover:ident, $remover_doc:expr] $(,$rest:tt)*) => {
        get_set_rm_str!($key_expr, $getter, $getter_doc);
        json_feed_prop_read_only_decl!($($rest),*);
    };
    ([str_array_prop, $key_expr:expr, $getter:ident, $getter_doc:expr, $setter:ident, $setter_doc:expr, $remover:ident, $remover_doc:expr] $(,$rest:tt)*) => {
        get_set_rm_str_array!($key_expr, $getter, $getter_doc);
        json_feed_prop_read_only_decl!($($rest),*);
    };
    ([u64_prop, $key_expr:expr, $getter:ident, $getter_doc:expr, $setter:ident, $setter_doc:expr, $remover:ident, $remover_doc:expr] $(,$rest:tt)*) => {
        get_set_rm_u64!($key_expr, $getter, $getter_doc);
        json_feed_prop_read_only_decl!($($rest),*);
    };
    ([bool_prop, $key_expr:expr, $getter:ident, $getter_doc:expr, $setter:ident, $setter_doc:expr, $remover:ident, $remover_doc:expr] $(,$rest:tt)*) => {
        get_set_rm_bool!($key_expr, $getter, $getter_doc);
        json_feed_prop_read_only_decl!($($rest),*);
    };
    ([obj_prop, $key_expr:expr, $getter_ref:ident, $getter_ref_type:ty, $getter_ref_new:expr, $getter_ref_doc:expr, $getter_ref_mut:ident, $getter_ref_mut_type:ty, $getter_ref_mut_new:expr, $getter_ref_mut_doc:expr, $setter:ident, $setter_type:ty, $setter_doc:expr, $remover:ident, $remover_doc:expr] $(,$rest:tt)*) => {
        get_ref_get_ref_mut_set_rm_obj!($key_expr, $getter_ref, $getter_ref_type, $getter_ref_new, $getter_ref_doc);
        json_feed_prop_read_only_decl!($($rest),*);
    };
    ([obj_array_prop, $key_expr:expr, $getter_ref:ident, $getter_ref_type:ty, $getter_ref_new:expr, $getter_ref_doc:expr, $getter_ref_mut:ident, $getter_ref_mut_type:ty, $getter_ref_mut_new:expr, $getter_ref_mut_do:expr, $setter:ident, $setter_type:ty, $setter_doc:expr, $remover:ident, $remover_doc:expr] $(,$rest:tt)*) => {
        get_ref_get_ref_mut_set_rm_obj_array!($key_expr, $getter_ref, $getter_ref_type, $getter_ref_new, $getter_ref_doc);
        json_feed_prop_read_only_decl!($($rest),*);
    };
}

macro_rules! trait_for_borrowed_type {
    ($name:ident) => {
        impl<'a> $name<'a> {
            /// Returns the inner `Map` as a reference.
            #[must_use]
            pub fn as_map(&self) -> &Map<String, Value> {
                self.value
            }
        }

        impl<'a> AsRef<Map<String, Value>> for $name<'a> {
            fn as_ref(&self) -> &Map<String, Value> {
                self.value
            }
        }

        impl<'a> core::fmt::Debug for $name<'a> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("value", &self.value)
                    .finish()
            }
        }

        impl<'a> Eq for $name<'a> {}

        impl<'a> From<&'a mut Map<String, Value>> for $name<'a> {
            fn from(value: &'a mut Map<String, Value>) -> Self {
                Self { value }
            }
        }

        impl<'a> PartialEq<Map<String, Value>> for $name<'a> {
            fn eq(&self, other: &Map<String, Value>) -> bool {
                self.value.eq(&other)
            }
        }

        impl<'a> PartialEq<$name<'a>> for $name<'a> {
            fn eq(&self, other: &$name<'_>) -> bool {
                self.value.eq(&other.value)
            }
        }
    };
}

macro_rules! json_feed_map_type {
    ($owned:ident, $owned_doc:expr, $borrowed:ident, $borrowed_doc:expr, $borrowed_mut:ident, $borrowed_mut_doc:expr, $to_owned:ident,
        $($rest:tt),*
    ) => {
        #[doc=$owned_doc]
        pub struct $owned {
            value: Map<String, Value>,
        }

        impl $owned {
            /// Instantiates with an empty JSON object.
            #[must_use]
            pub fn new() -> Self {
                Self { value: Map::new() }
            }

            /// Returns the inner `Map` as a reference.
            #[must_use]
            pub fn as_map(&self) -> &Map<String, Value> {
                &self.value
            }

            /// Returns the inner `Map` as a mutable reference.
            pub fn as_map_mut(&mut self) -> &mut Map<String, Value> {
                &mut self.value
            }

            /// Converts the type into the inner `Map`.
            #[must_use]
            pub fn into_inner(self) -> Map<String, Value> {
                self.value
            }

            json_feed_prop_decl!($($rest),*);
        }

        impl AsRef<Map<String,Value>> for $owned {
            fn as_ref(&self) -> &Map<String, Value> {
                &self.value
            }
        }

        impl AsMut<Map<String,Value>> for $owned {
            fn as_mut(&mut self) -> &mut Map<String, Value> {
                &mut self.value
            }
        }

        impl Clone for $owned {
            fn clone(&self) -> $owned {
                $owned {
                    value: self.value.clone(),
                }
            }
        }

        impl core::fmt::Debug for $owned {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($owned))
                    .field("value", &self.value)
                    .finish()
            }
        }

        impl Default for $owned {
            fn default() -> Self {
                Self::new()
            }
        }

        impl Eq for $owned {}

        impl From<Map<String, Value>> for $owned {
            fn from(value: Map<String, Value>) -> Self {
                Self {
                    value
                }
            }
        }

        impl PartialEq<Map<String, Value>> for $owned {
            fn eq(&self, other: &Map<String, Value>) -> bool {
                self.value.eq(&other)
            }
        }

        impl PartialEq<$owned> for $owned {
            fn eq(&self, other: &$owned) -> bool {
                self.value.eq(&other.value)
            }
        }

        impl serde::Serialize for $owned
        {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                self.value.serialize(serializer)
            }
        }

        impl<'de> serde::de::Deserialize<'de> for $owned {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                let map: Map<String, Value> = Map::deserialize(deserializer)?;
                Ok(Self { value: map })
            }
        }

        #[doc=$borrowed_doc]
        pub struct $borrowed<'a> {
            value: &'a Map<String, Value>,
        }

        trait_for_borrowed_type!($borrowed);

        impl<'a> $borrowed<'a> {
            /// Clones the inner `Map` reference and returns an owned type.
            #[must_use]
            pub fn $to_owned(&self) -> $owned {
                $owned::from(self.value.clone())
            }

            json_feed_prop_read_only_decl!($($rest),*);
        }

        impl<'a> From<&'a Map<String, Value>> for $borrowed<'a> {
            fn from(value: &'a Map<String, Value>) -> Self {
                Self { value }
            }
        }

        impl<'a> serde::Serialize for $borrowed<'a>
        {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                self.value.serialize(serializer)
            }
        }

        #[doc=$borrowed_mut_doc]
        pub struct $borrowed_mut<'a> {
            value: &'a mut Map<String, Value>,
        }

        trait_for_borrowed_type!($borrowed_mut);

        impl<'a> $borrowed_mut<'a> {
            /// Returns the inner `Map` as a mutable reference.
            pub fn as_map_mut(&mut self) -> &mut Map<String, Value> {
                self.value
            }

            /// Clones the inner `Map` reference and returns an owned type.
            #[must_use]
            pub fn $to_owned(&self) -> $owned {
                $owned::from(self.value.clone())
            }

            json_feed_prop_decl!($($rest),*);
        }

        impl<'a> AsMut<Map<String, Value>> for $borrowed_mut<'a> {
            fn as_mut(&mut self) -> &mut Map<String, Value> {
                self.value
            }
        }

        impl<'a> serde::Serialize for $borrowed_mut<'a>
        {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                self.value.serialize(serializer)
            }
        }
    };
}

json_feed_map_type!(
    Author,
    "An author of a feed or an item in the feed.

# Valid Author

An `Author` must have at least one of the `name`, `url`, or `avatar` properties set.
",
    AuthorRef,
    "An `Author` implemented with a borrowed reference to a JSON object.",
    AuthorMut,
    "An `Author` implemented with a borrowed mutable reference to a JSON object.",
    to_author,
    [
        str_prop,
        "name",
        name,
        "The optional author's name.",
        set_name,
        "Sets the name.",
        remove_name,
        "Remove the name."
    ],
    [
        str_prop,
        "url",
        url,
        "An optional URL for a site which represents the author.",
        set_url,
        "Sets the URL.",
        remove_url,
        "Removes the URL."
    ],
    [
        str_prop,
        "avatar",
        avatar,
        "An optional URL for an image which represents the author.",
        set_avatar,
        "Sets the avatar.",
        remove_avatar,
        "Removes the avatar."
    ]
);

json_feed_map_type!(
    Hub,
    "A subscription endpoint which can be used to receive feed update notifications.

# Valid Hub

A `Hub` must have both the `type` and `url` properties set.
",
    HubRef,
    "A `Hub` implemented with a borrowed reference to a JSON object.",
    HubMut,
    "A `Hub` implemented with a borrowed mutable reference to a JSON object.",
    to_hub,
    [
        str_prop,
        "type",
        hub_type,
        "The required protocol which is used to subscribe with.",
        set_hub_type,
        "Sets the type.",
        remove_hub_type,
        "Removes the type."
    ],
    [
        str_prop,
        "url",
        url,
        "A required hub type specific URL which is used to subscribe with.",
        set_url,
        "Sets the URL.",
        remove_url,
        "Removes the URL."
    ]
);

json_feed_map_type!(
    Item,
    "An item is a single object (blog post, story, etc.) in the feed list.

# Valid Item

An `Item` must have an `id` property set and either a `content_html` or `content_text` property set.
",
    ItemRef,
    "An `Item` implemented with a borrowed reference to a JSON object.",
    ItemMut,
    "An `Item` implemented with a borrowed mutable reference to a JSON object.",
    to_item,
    [str_prop, "id", id, "A required unique identifier for an item.

# Important

The ID should be unique across all items which have ever appeared in the feed.
An item with the same exact ID as another item (even if it is no longer in the
current JSON feed `items` array) are considered the same item.

# Version 1.0 Incompatibility

While JSON Feed 1.0 permitted values which could be coerced into JSON strings (e.g. JSON numbers), this model supports only
JSON strings. JSON Feed 1.1 strongly suggests to only use strings. In practice, the vast majority of feeds use strings.

If you wish to support non-String IDs, you can directly access the underlying `Map` with `as_map_mut` or an equivalent method and
read the JSON value.
", set_id, "Sets the ID.", remove_id, "Removes the ID."],
    [str_prop, "url", url, "The optional URL which the item represents.", set_url, "Sets the URL.", remove_url, "Removes the URL."],
    [
        str_prop,
        "external_url",
        external_url,
        "An optional related external URL to the item.",
        set_external_url,
        "Sets the external URL.",
        remove_external_url,
        "Removes the external URL."
    ],
    [
        str_prop,
        "title",
        title,
        "An optional title for the item.",
        set_title,
        "Sets the title.",
        remove_title,
        "Removes the title."
    ],
    [
        str_prop,
        "content_html",
        content_html,
        "An optional HTML string representing the content.",
        set_content_html,
        "Sets the HTML content.",
        remove_content_html,
        "Removes the HTML content."
    ],
    [
        str_prop,
        "content_text",
        content_text,
        "An optional plain text string representing the content.",
        set_content_text,
        "Sets the plain text content.",
        remove_content_text,
        "Removes the plain text content."
    ],
    [
        str_prop,
        "summary",
        summary,
        "An optional summary of the item.",
        set_summary,
        "Sets the summary.",
        remove_summary,
        "Removes the summary."
    ],
    [
        str_prop,
        "image",
        image,
        "An optional URL of an image representing the item.",
        set_image,
        "Sets the image.",
        remove_image,
        "Removes the image."
    ],
    [
        str_prop,
        "banner_image",
        banner_image,
        "An optional URL of a banner image representing the item.",
        set_banner_image,
        "Sets the banner image.",
        remove_banner_image,
        "Removes the banner image."
    ],
    [
        str_prop,
        "date_published",
        date_published,
        "The date which the item was published in [RFC 3339][rfc_3339] format.

[rfc_3339]: https://tools.ietf.org/html/rfc3339
",
        set_date_published,
        "Sets the date published.",
        remove_date_published,
        "Removes the date published."
    ],
    [
        str_prop,
        "date_modified",
        date_modified,
        "The date which the item was modified in [RFC 3339][rfc_3339] format.

[rfc_3339]: https://tools.ietf.org/html/rfc3339
",
        set_date_modified,
        "Sets the date modified.",
        remove_date_modified,
        "Removes the date modified."
    ],
    [
        obj_prop,
        "author",
        author,
        AuthorRef<'_>,
        AuthorRef::from,
        "An optional author.

# Deprecation

The `author` field is deprecated in favor of the `authors` field as of JSON Feed 1.1.
",
        author_mut,
        AuthorMut<'_>,
        AuthorMut::from,
        "An optional author.

# Deprecation

The `author` field is deprecated in favor of the `authors` field as of JSON Feed 1.1.
",
        set_author,
        Author,
        "Sets the author.",
        remove_author,
        "Removes the author."
    ],
    [
        obj_array_prop,
        "authors",
        authors,
        AuthorRef<'_>,
        AuthorRef::from,
        "An optional array of authors.",
        authors_mut,
        AuthorMut<'_>,
        AuthorMut::from,
        "An optional array of authors.",
        set_authors,
        Author,
        "Sets the authors.",
        remove_authors,
        "Removes the authors."
    ],
    [
        str_array_prop,
        "tags",
        tags,
        "An optional array of plain text tags.",
        set_tags,
        "Sets the tags.",
        remove_tags,
        "Removes the tags."
    ],
    [
        str_prop,
        "language",
        language,
        "The optional language which the feed data is written in.

Valid values are from [RFC 5646][rfc_5646].

[rfc_5646]: https://tools.ietf.org/html/rfc5646
",
        set_language,
        "Sets the language.",
        remove_language,
        "Removes the language."
    ],
    [
        obj_array_prop,
        "attachments",
        attachments,
        AttachmentRef<'_>,
        AttachmentRef::from,
        "An optional array of relevant resources for the item.",
        attachments_mut,
        AttachmentMut<'_>,
        AttachmentMut::from,
        "An optional array of relevant resources for the item.",
        set_attachments,
        Attachment,
        "Sets the attachments.",
        remove_attachments,
        "Removes the attachments."
    ]
);

json_feed_map_type!(
    Attachment,
    "A relevant resource for an `Item`.

# Valid Attachment

An `Attachment` must have both the `url` and `mime_type` properties set.
",
    AttachmentRef,
    "An `Attachment` implemented with a borrowed reference to a JSON object.",
    AttachmentMut,
    "An `Attachment` implemented with a borrowed mutable reference to a JSON object.",
    to_attachment,
    [
        str_prop,
        "url",
        url,
        "The required URL for the attachment.",
        set_url,
        "Sets the URL.",
        remove_url,
        "Removes the URL."
    ],
    [
        str_prop,
        "mime_type",
        mime_type,
        "The required [MIME][mime] type (e.g. image/png).

[mime]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types
",
        set_mime_type,
        "Sets the MIME type.",
        remove_mime_type,
        "Removes the MIME type."
    ],
    [
        str_prop,
        "title",
        title,
        "An optional title for the attachment.

# Important

Attachments with the same title are considered to be alternative representations of an attachment.
 ",
        set_title,
        "Sets the title.",
        remove_title,
        "Removes the title."
    ],
    [
        u64_prop,
        "size_in_bytes",
        size_in_bytes,
        "The optional size of the attachment in bytes.",
        set_size_in_bytes,
        "Sets the size in bytes.",
        remove_size_in_bytes,
        "Removes the size in bytes."
    ],
    [
        u64_prop,
        "duration_in_seconds",
        duration_in_seconds,
        "The optional duration of the content in seconds.",
        set_duration_in_seconds,
        "Sets the duration of in seconds.",
        remove_duration_in_seconds,
        "Removes the duration in seconds."
    ]
);

json_feed_map_type!(
    Feed,
    r#"A list of items with associated metadata.

The type provides a view into a JSON object value with accessor methods for the standard properties.
`Feed` owns the underlying JSON object data and provides methods to access the backing object itself
with `as_map`, `as_map_mut`, and `into_inner`.

The underlying data is not guaranteed to be a valid JSON Feed.

# Valid Feed

A `Feed` must have the `version` set to a valid JSON Feed version value, the `title` property set, and the `items`
property set.

# Example

```
use json_feed_model::{Feed};
# fn main() -> Result<(), json_feed_model::Error> {
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
let feed = json_feed_model::from_value(json).unwrap();
assert_eq!(feed.version()?, Some(json_feed_model::VERSION_1_1));
assert_eq!(feed.title()?, Some("Lorem ipsum dolor sit amet."));
assert_eq!(feed.home_page_url()?, Some("https://example.org/"));
assert_eq!(feed.feed_url()?, Some("https://example.org/feed.json"));

let items = feed.items()?;
let items = items.unwrap();
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
# Ok(())
# }
```
    "#,
    FeedRef,
    "A `Feed` implemented with a borrowed reference to a JSON object.",
    FeedMut,
    "A `Feed` implemented with a borrowed mutable reference to a JSON object.",
    to_feed,
    [
        str_prop,
        "version",
        version,
        "The required URL formatted version identifier.

Identifies what version of the spec the feed is suppose to be compliant with.",
        set_version,
        "Sets the version identifier.",
        remove_version,
        "Removes the version identifier."
    ],
    [
        str_prop,
        "title",
        title,
        "The optional name of the feed.",
        set_title,
        "Sets the name of the feed.",
        remove_title,
        "Removes the name of the feed."
    ],
    [
        str_prop,
        "home_page_url",
        home_page_url,
        "The optional URL which the feed is suppose to represent.",
        set_home_page_url,
        "Sets the home page URL.",
        remove_home_page_url,
        "Removes the home page URL."
    ],
    [
        str_prop,
        "feed_url",
        feed_url,
        "The optional URL which this feed can be retrieived from.",
        set_feed_url,
        "Sets the feed URL.",
        remove_feed_url,
        "Removes the feed URL."
    ],
    [
        str_prop,
        "description",
        description,
        "An optional description of the feed.",
        set_description,
        "Sets the description of the feed.",
        remove_description,
        "Removes the description of the feed."
    ],
    [
        str_prop,
        "user_comment",
        user_comment,
        "An optional meta description about the feed only intended to be viewed in the raw JSON form.",
        set_user_comment,
        "Sets the user comment.",
        remove_user_comment,
        "Removes the user comment."
    ],
    [
        str_prop,
        "next_url",
        next_url,
        "An optional pagination URL.",
        set_next_url,
        "Sets the next URL.",
        remove_next_url,
        "Removes the next URL."
    ],
    [str_prop, "icon", icon, "An optional URL to an icon for use in a list of items.", set_icon, "Sets the icon.", remove_icon, "Removes the icon."],
    [
        str_prop,
        "favicon",
        favicon,
        "An optional URL to a favicon suitable for use in a list of feeds.",
        set_favicon,
        "Sets the favicon URL.",
        remove_favicon,
        "Removes the favicon URL."
    ],
    [
        obj_prop,
        "author",
        author,
        AuthorRef<'_>,
        AuthorRef::from,
        "An optional author.

# Deprecation

The `author` field is deprecated in favor of the `authors` field as of JSON Feed 1.1.
",
        author_mut,
        AuthorMut<'_>,
        AuthorMut::from,
        "An optional author.

# Deprecation

The `author` field is deprecated in favor of the `authors` field as of JSON Feed 1.1.
",
        set_author,
        Author,
        "Sets the author.",
        remove_author,
        "Removes the author."
    ],
    [
        obj_array_prop,
        "authors",
        authors,
        AuthorRef<'_>,
        AuthorRef::from,
        "An optional array of authors.",
        authors_mut,
        AuthorMut<'_>,
        AuthorMut::from,
        "An optional array of authors.",
        set_authors,
        Author,
        "Sets the authors.",
        remove_authors,
        "Removes the authors."
    ],
    [
        str_prop,
        "language",
        language,
        "The optional language which the feed data is written in.

Valid values are from [RFC 5646][rfc_5646].

[rfc_5646]: https://tools.ietf.org/html/rfc5646
",
        set_language,
        "Sets the language.",
        remove_language,
        "Removes the language."
    ],
    [
        bool_prop,
        "expired",
        expired,
        "Optionally determines if the feed will be updated in the future.
        
If true, the feed will not be updated in the future. If false or `None`, then the feed may be updated in the future.",
        set_expired,
        "Sets the expired flag.",
        remove_expired,
        "Removes the expired flag."
    ],
    [
        obj_array_prop,
        "hubs",
        hubs,
        HubRef<'_>,
        HubRef::from,
        "Optional subscription endpoints which can be used to received feed update notifications.",
        hubs_mut,
        HubMut<'_>,
        HubMut::from,
        "Subscription endpoints which can be used to received feed update notifications.",
        set_hubs,
        Hub,
        "Sets the hubs.",
        remove_hubs,
        "Removes the hubs."
    ],
    [
        obj_array_prop,
        "items",
        items,
        ItemRef<'_>,
        ItemRef::from,
        "A required array of `Items`.",
        items_mut,
        ItemMut<'_>,
        ItemMut::from,
        "A required array of `Items`.",
        set_items,
        Item,
        "Sets the items.",
        remove_items,
        "Removes the items."
    ]
);

fn is_extension_key(key: &str) -> bool {
    key.as_bytes().iter().next() == Some(&b'_')
}

fn are_keys_valid<'a, I>(keys: I, valid_keys: &BTreeSet<&str>) -> bool
where
    I: IntoIterator<Item = &'a String>,
{
    keys.into_iter()
        .all(|k| valid_keys.contains(k.as_str()) || is_extension_key(k))
}

fn is_valid_attachment(map: &Map<String, Value>, version: &Version<'_>) -> bool {
    match version {
        Version::Unknown(_) => return false,
        Version::Version1 | Version::Version1_1 => {}
    }
    let attachment_ref = AttachmentRef::from(map);
    let mut valid_keys = BTreeSet::new();
    valid_keys.insert("url");
    valid_keys.insert("mime_type");
    valid_keys.insert("title");
    valid_keys.insert("size_in_bytes");
    valid_keys.insert("duration_in_seconds");

    attachment_ref.url().map_or(false, |url| url.is_some())
        && attachment_ref
            .mime_type()
            .map_or(false, |mime_type| mime_type.is_some())
        && attachment_ref.title().is_ok()
        && attachment_ref.size_in_bytes().is_ok()
        && attachment_ref.duration_in_seconds().is_ok()
        && are_keys_valid(map.keys(), &valid_keys)
}

impl Attachment {
    /// Verifies if the JSON data complies with a specific `Version` of the JSON Feed spec.
    #[must_use]
    pub fn is_valid(&self, version: &Version<'_>) -> bool {
        is_valid_attachment(&self.value, version)
    }
}

impl<'a> AttachmentMut<'a> {
    /// Verifies if the JSON data complies with a specific `Version` of the JSON Feed spec.
    #[must_use]
    pub fn is_valid(&self, version: &Version<'_>) -> bool {
        is_valid_attachment(self.value, version)
    }
}

impl<'a> AttachmentRef<'a> {
    /// Verifies if the JSON data complies with a specific `Version` of the JSON Feed spec.
    #[must_use]
    pub fn is_valid(&self, version: &Version<'_>) -> bool {
        is_valid_attachment(self.value, version)
    }
}

fn is_valid_author(map: &Map<String, Value>, version: &Version<'_>) -> bool {
    match version {
        Version::Unknown(_) => return false,
        Version::Version1 | Version::Version1_1 => {}
    }
    let author_ref = AuthorRef::from(map);
    let mut valid_keys = BTreeSet::new();
    valid_keys.insert("name");
    valid_keys.insert("avatar");
    valid_keys.insert("url");

    let name_result = author_ref.name();
    let avatar_result = author_ref.avatar();
    let url_result = author_ref.url();

    name_result.is_ok()
        && avatar_result.is_ok()
        && url_result.is_ok()
        && (name_result.map_or(false, |name| name.is_some())
            || avatar_result.map_or(false, |avatar| avatar.is_some())
            || url_result.map_or(false, |url| url.is_some()))
        && are_keys_valid(map.keys(), &valid_keys)
}

impl Author {
    /// Verifies if the JSON data complies with a specific `Version` of the JSON Feed spec.
    #[must_use]
    pub fn is_valid(&self, version: &Version<'_>) -> bool {
        is_valid_author(&self.value, version)
    }
}

impl<'a> AuthorMut<'a> {
    /// Verifies if the JSON data complies with a specific `Version` of the JSON Feed spec.
    #[must_use]
    pub fn is_valid(&self, version: &Version<'_>) -> bool {
        is_valid_author(self.value, version)
    }
}

impl<'a> AuthorRef<'a> {
    /// Verifies if the JSON data complies with a specific `Version` of the JSON Feed spec.
    #[must_use]
    pub fn is_valid(&self, version: &Version<'_>) -> bool {
        is_valid_author(self.value, version)
    }
}

fn is_valid_feed(map: &Map<String, Value>, version: &Version<'_>) -> bool {
    match version {
        Version::Unknown(_) => return false,
        Version::Version1 | Version::Version1_1 => {}
    }
    let feed_ref = FeedRef::from(map);
    let mut valid_keys = BTreeSet::new();
    valid_keys.insert("version");
    valid_keys.insert("title");
    valid_keys.insert("home_page_url");
    valid_keys.insert("feed_url");
    valid_keys.insert("description");
    valid_keys.insert("user_comment");
    valid_keys.insert("next_url");
    valid_keys.insert("favicon");
    valid_keys.insert("author");
    match version {
        Version::Version1_1 => {
            valid_keys.insert("authors");
            valid_keys.insert("language");
        }
        Version::Version1 | Version::Unknown(_) => {}
    }
    valid_keys.insert("expired");
    valid_keys.insert("hubs");
    valid_keys.insert("items");

    feed_ref.version().map_or(false, |v| {
        v.map_or(false, |v| match Version::from(v) {
            Version::Unknown(_) => false,
            Version::Version1 => match version {
                Version::Version1 | Version::Version1_1 => true,
                Version::Unknown(_) => false,
            },
            Version::Version1_1 => match version {
                Version::Version1 | Version::Unknown(_) => false,
                Version::Version1_1 => true,
            },
        })
    }) && feed_ref
        .title()
        .map_or_else(|_| false, |title| title.is_some())
        && feed_ref.items().map_or(false, |items| {
            items.map_or(false, |items| {
                items.iter().all(|item| item.is_valid(version))
            })
        })
        && feed_ref.hubs().map_or(false, |hubs| {
            hubs.map_or(true, |hubs| hubs.iter().all(|hub| hub.is_valid(version)))
        })
        && feed_ref.home_page_url().is_ok()
        && feed_ref.feed_url().is_ok()
        && feed_ref.description().is_ok()
        && feed_ref.user_comment().is_ok()
        && feed_ref.next_url().is_ok()
        && feed_ref.icon().is_ok()
        && feed_ref.favicon().is_ok()
        && feed_ref.author().is_ok()
        && feed_ref.authors().is_ok()
        && feed_ref.language().is_ok()
        && feed_ref.expired().is_ok()
        && are_keys_valid(map.keys(), &valid_keys)
}

impl Feed {
    /// Verifies if the JSON data complies with a specific `Version` of the JSON Feed spec.
    #[must_use]
    pub fn is_valid(&self, version: &Version<'_>) -> bool {
        is_valid_feed(&self.value, version)
    }
}

impl<'a> FeedMut<'a> {
    /// Verifies if the JSON data complies with a specific `Version` of the JSON Feed spec.
    #[must_use]
    pub fn is_valid(&self, version: &Version<'_>) -> bool {
        is_valid_feed(self.value, version)
    }
}

impl<'a> FeedRef<'a> {
    /// Verifies if the JSON data complies with a specific `Version` of the JSON Feed spec.
    #[must_use]
    pub fn is_valid(&self, version: &Version<'_>) -> bool {
        is_valid_feed(self.value, version)
    }
}

fn is_valid_hub(map: &Map<String, Value>, version: &Version<'_>) -> bool {
    match version {
        Version::Unknown(_) => return false,
        Version::Version1 | Version::Version1_1 => {}
    }
    let hub_ref = HubRef::from(map);
    let mut valid_keys = BTreeSet::new();
    valid_keys.insert("type");
    valid_keys.insert("url");

    hub_ref.url().map_or(false, |url| url.is_some())
        && hub_ref
            .hub_type()
            .map_or(false, |hub_type| hub_type.is_some())
        && are_keys_valid(map.keys(), &valid_keys)
}

impl Hub {
    /// Verifies if the JSON data complies with a specific `Version` of the JSON Feed spec.
    #[must_use]
    pub fn is_valid(&self, version: &Version<'_>) -> bool {
        is_valid_hub(&self.value, version)
    }
}

impl<'a> HubMut<'a> {
    /// Verifies if the JSON data complies with a specific `Version` of the JSON Feed spec.
    #[must_use]
    pub fn is_valid(&self, version: &Version<'_>) -> bool {
        is_valid_hub(self.value, version)
    }
}

impl<'a> HubRef<'a> {
    /// Verifies if the JSON data complies with a specific `Version` of the JSON Feed spec.
    #[must_use]
    pub fn is_valid(&self, version: &Version<'_>) -> bool {
        is_valid_hub(self.value, version)
    }
}

fn is_valid_item(map: &Map<String, Value>, version: &Version<'_>) -> bool {
    match version {
        Version::Unknown(_) => return false,
        Version::Version1 | Version::Version1_1 => {}
    }
    let item_ref = ItemRef::from(map);
    let mut valid_keys = BTreeSet::new();
    valid_keys.insert("id");
    valid_keys.insert("url");
    valid_keys.insert("external_url");
    valid_keys.insert("title");
    valid_keys.insert("content_html");
    valid_keys.insert("content_text");
    valid_keys.insert("summary");
    valid_keys.insert("image");
    valid_keys.insert("banner_image");
    valid_keys.insert("date_published");
    valid_keys.insert("date_modified");
    valid_keys.insert("author");
    match version {
        Version::Version1_1 => {
            valid_keys.insert("authors");
            valid_keys.insert("language");
        }
        Version::Version1 | Version::Unknown(_) => {}
    }
    valid_keys.insert("tags");
    valid_keys.insert("attachments");

    let content_html_result = item_ref.content_html();
    let content_text_result = item_ref.content_text();

    item_ref.id().map_or(false, |id| id.is_some())
        && item_ref.authors().map_or(false, |authors| {
            authors.map_or(true, |authors| {
                authors.iter().all(|author| author.is_valid(version))
            })
        })
        && item_ref.attachments().map_or(false, |attachments| {
            attachments.map_or(true, |attachments| {
                attachments
                    .iter()
                    .all(|attachment| attachment.is_valid(version))
            })
        })
        && item_ref.id().is_ok()
        && item_ref.url().is_ok()
        && item_ref.external_url().is_ok()
        && item_ref.title().is_ok()
        && content_html_result.is_ok()
        && content_text_result.is_ok()
        && (content_text_result.map_or(false, |content| content.is_some())
            || content_html_result.map_or(false, |content| content.is_some()))
        && item_ref.summary().is_ok()
        && item_ref.image().is_ok()
        && item_ref.banner_image().is_ok()
        && item_ref.date_published().is_ok()
        && item_ref.date_modified().is_ok()
        && item_ref.author().is_ok()
        && item_ref.tags().is_ok()
        && item_ref.language().is_ok()
        && are_keys_valid(map.keys(), &valid_keys)
}

impl Item {
    /// Verifies if the JSON data complies with a specific `Version` of the JSON Feed spec.
    #[must_use]
    pub fn is_valid(&self, version: &Version<'_>) -> bool {
        is_valid_item(&self.value, version)
    }
}

impl<'a> ItemMut<'a> {
    /// Verifies if the JSON data complies with a specific `Version` of the JSON Feed spec.
    #[must_use]
    pub fn is_valid(&self, version: &Version<'_>) -> bool {
        is_valid_item(self.value, version)
    }
}

impl<'a> ItemRef<'a> {
    /// Verifies if the JSON data complies with a specific `Version` of the JSON Feed spec.
    #[must_use]
    pub fn is_valid(&self, version: &Version<'_>) -> bool {
        is_valid_item(self.value, version)
    }
}

/// Attempts to JSON decode a `std::io::Read` and return a `Feed`.
///
/// # Errors
///
/// If the data cannot be JSON decoded, then `Error::SerdeJson(serde_json::Error)` is returned.
///
/// If the decoded JSON value is not an Object, then `Error::UnexpectedType` is returned.
#[cfg(feature = "std")]
pub fn from_reader<R>(reader: R) -> Result<Feed, Error>
where
    R: std::io::Read,
{
    let value = serde_json::from_reader(reader)?;
    from_value(value)
}

/// Attempts to JSON decode a `str` and return a `Feed`.
///
/// # Errors
///
/// If the string cannot be JSON decoded, then `Error::SerdeJson(serde_json::Error)` is returned.
///
/// If the decoded JSON value is not an Object, then `Error::UnexpectedType` is returned.
pub fn from_str(s: &str) -> Result<Feed, Error> {
    from_slice(s.as_bytes())
}

/// Attempts to JSON decode a byte slice and return a `Feed`.
///
/// # Errors
///
/// If the byte slice cannot be JSON decoded, then `Error::SerdeJson(serde_json::Error)` is returned.
///
/// If the decoded JSON value is not an Object, then `Error::UnexpectedType` is returned.
pub fn from_slice(v: &[u8]) -> Result<Feed, Error> {
    let value = serde_json::from_slice(v)?;
    from_value(value)
}

/// Attempts to return a `Feed` from a JSON `Value`.
///
/// # Errors
///
/// If the JSON value is not an Object, then `Error::UnexpectedType` is returned.
///
/// # Example
///
/// If the library user wishes to save invalid JSON values, a simple check should be done
/// before calling the function.
///
/// ```
/// let value = serde_json::json!("a JSON String, not an Object");
/// match &value {
///     serde_json::Value::Object(_) => {
///         let feed_result = json_feed_model::from_value(value);
///         assert!(false, "should not have execute this code")
///     }
///     _ => {
///         // handle the invalid JSON value
///     },
/// }
pub fn from_value(value: Value) -> Result<Feed, Error> {
    match value {
        Value::Object(obj) => Ok(Feed { value: obj }),
        _ => Err(Error::UnexpectedType),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    use alloc::vec;

    #[test]
    fn simple_example() -> Result<(), Error> {
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

        let feed = from_value(json)?;

        assert!(feed.is_valid(&Version::Version1_1));

        assert_eq!(feed.version()?, Some(VERSION_1_1));
        assert_eq!(feed.title()?, Some("Lorem ipsum dolor sit amet."));
        assert_eq!(feed.home_page_url()?, Some("https://example.org/"));
        assert_eq!(feed.feed_url()?, Some("https://example.org/feed.json"));

        let items: Option<Vec<ItemRef<'_>>> = feed.items()?;
        assert!(items.is_some());
        let items: Vec<ItemRef<'_>> = items.unwrap();
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

        Ok(())
    }

    #[test]
    fn read_extensions() -> Result<(), Error> {
        let json = serde_json::json!({
            "version": "https://jsonfeed.org/version/1.1",
            "title": "Lorem ipsum dolor sit amet.",
            "_example": {
                "id": "cd7f0673-8e81-4e13-b273-4bd1b83967d0"
            },
            "items": [
                {
                    "id": "2bcb497d-c40b-4493-b5ae-bc63c74b48fa",
                    "content_html": "Vestibulum non magna vitae tortor.",
                    "url": "https://example.org/vestibulum-non",
                    "_extension": 1
                }
            ]
        });
        let feed = from_value(json).unwrap();

        assert!(feed.is_valid(&Version::Version1_1));

        assert_eq!(feed.version()?, Some(VERSION_1_1));
        assert_eq!(feed.title()?, Some("Lorem ipsum dolor sit amet."));

        let example_value = feed.as_map().get("_example");
        assert_eq!(
            example_value,
            Some(&serde_json::json!({ "id": "cd7f0673-8e81-4e13-b273-4bd1b83967d0" }))
        );

        let items = feed.items()?;
        let items = items.unwrap();
        assert_eq!(items.len(), 1);

        assert_eq!(items[0].id()?, Some("2bcb497d-c40b-4493-b5ae-bc63c74b48fa"));
        assert_eq!(
            items[0].content_html()?,
            Some("Vestibulum non magna vitae tortor.")
        );
        assert_eq!(items[0].url()?, Some("https://example.org/vestibulum-non"));

        let extension_value = items[0].as_map().get("_extension");
        assert_eq!(extension_value, Some(&serde_json::json!(1)));

        Ok(())
    }

    #[test]
    fn write_extensions() -> Result<(), Error> {
        let mut feed = Feed::new();
        feed.set_version(Version::Version1_1);
        feed.set_title("Lorem ipsum dolor sit amet.");
        feed.as_map_mut().insert(
            String::from("_example"),
            serde_json::json!({ "id": "cd7f0673-8e81-4e13-b273-4bd1b83967d0" }),
        );

        let mut item = Item::new();
        item.set_id("invalid-id");
        item.set_content_html("Vestibulum non magna vitae tortor.");
        item.set_url("https://example.org/vestibulum-non");
        item.as_map_mut()
            .insert(String::from("_extension"), serde_json::json!(1));

        let items = vec![item];
        feed.set_items(items);

        let item = &mut feed.items_mut()?.unwrap()[0];
        item.set_id("2bcb497d-c40b-4493-b5ae-bc63c74b48fa");

        assert!(feed.is_valid(&Version::Version1_1));

        let expected_json = serde_json::json!({
            "version": "https://jsonfeed.org/version/1.1",
            "title": "Lorem ipsum dolor sit amet.",
            "_example": {
                "id": "cd7f0673-8e81-4e13-b273-4bd1b83967d0"
            },
            "items": [
                {
                    "id": "2bcb497d-c40b-4493-b5ae-bc63c74b48fa",
                    "content_html": "Vestibulum non magna vitae tortor.",
                    "url": "https://example.org/vestibulum-non",
                    "_extension": 1
                }
            ]
        });
        assert_eq!(feed, from_value(expected_json.clone())?);
        assert_eq!(serde_json::to_value(feed.clone())?, expected_json);

        let output = serde_json::to_string(&feed);
        assert!(output.is_ok());

        Ok(())
    }

    #[test]
    fn is_valid_version_forward_compatible() {
        let json = serde_json::json!({
            "version": "https://jsonfeed.org/version/1",
            "title": "Lorem ipsum dolor sit amet.",
            "items": [
                {
                    "id": "2bcb497d-c40b-4493-b5ae-bc63c74b48fa",
                    "content_html": "Vestibulum non magna vitae tortor.",
                    "url": "https://example.org/vestibulum-non",
                }
            ]
        });
        let feed = from_value(json).unwrap();

        assert!(feed.is_valid(&Version::Version1_1));
        assert!(feed.is_valid(&Version::Version1));
    }

    #[test]
    fn is_valid_version_backward_compatible() {
        let json = serde_json::json!({
            "version": "https://jsonfeed.org/version/1.1",
            "title": "Lorem ipsum dolor sit amet.",
            "items": [
                {
                    "id": "2bcb497d-c40b-4493-b5ae-bc63c74b48fa",
                    "content_html": "Vestibulum non magna vitae tortor.",
                    "url": "https://example.org/vestibulum-non",
                }
            ]
        });
        let feed = from_value(json).unwrap();

        assert!(feed.is_valid(&Version::Version1_1));
        assert!(!feed.is_valid(&Version::Version1));
    }

    #[test]
    fn custom_extension_trait() -> Result<(), Error> {
        trait ExampleExtension {
            fn example(&self) -> Result<Option<&str>, Error>;

            fn set_example<T>(&mut self, value: T) -> Option<Value>
            where
                T: ToString;
        }

        impl ExampleExtension for Feed {
            fn example(&self) -> Result<Option<&str>, Error> {
                self.as_map().get("_example").map_or_else(
                    || Ok(None),
                    |value| match value {
                        Value::String(s) => Ok(Some(s.as_str())),
                        _ => Err(Error::UnexpectedType),
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
        assert_eq!(feed, from_value(expected_json)?);

        assert_eq!(feed.example()?, Some("123456"));

        let output = serde_json::to_string(&feed);
        assert!(output.is_ok());

        Ok(())
    }
}
