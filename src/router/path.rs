// Copied from `apollo-router/src/json_ext.rs` (commit: d9336e43f)

use std::fmt;

use once_cell::sync::Lazy;
use regex::Captures;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;

const FRAGMENT_PREFIX: &str = "... on ";

static TYPE_CONDITIONS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\|\[(?<condition>.+?)?\]")
        .expect("this regex to check for type conditions is valid")
});

/// Extract the condition list from the regex captures.
fn extract_matched_conditions(caps: &Captures) -> TypeConditions {
    caps.name("condition")
        .map(|c| c.as_str().split(',').map(|s| s.to_string()).collect())
        .unwrap_or_default()
}

fn split_path_element_and_type_conditions(s: &str) -> (String, Option<TypeConditions>) {
    let mut type_conditions = None;
    let path_element = TYPE_CONDITIONS_REGEX.replace(s, |caps: &Captures| {
        type_conditions = Some(extract_matched_conditions(caps));
        ""
    });
    (path_element.to_string(), type_conditions)
}

/// A GraphQL path element that is composes of strings or numbers.
/// e.g `/book/3/name`
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
#[serde(untagged)]
pub enum PathElement {
    /// A path element that given an array will flatmap the content.
    #[serde(
        deserialize_with = "deserialize_flatten",
        serialize_with = "serialize_flatten"
    )]
    Flatten(Option<TypeConditions>),

    /// An index path element.
    Index(usize),

    /// A fragment application
    #[serde(
        deserialize_with = "deserialize_fragment",
        serialize_with = "serialize_fragment"
    )]
    Fragment(String),

    /// A key path element.
    #[serde(deserialize_with = "deserialize_key", serialize_with = "serialize_key")]
    Key(String, Option<TypeConditions>),
}

type TypeConditions = Vec<String>;

fn deserialize_flatten<'de, D>(deserializer: D) -> Result<Option<TypeConditions>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_str(FlattenVisitor)
}

struct FlattenVisitor;

impl serde::de::Visitor<'_> for FlattenVisitor {
    type Value = Option<TypeConditions>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a string that is '@', potentially followed by type conditions"
        )
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let (path_element, type_conditions) = split_path_element_and_type_conditions(s);
        if path_element == "@" {
            Ok(type_conditions)
        } else {
            Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(s),
                &self,
            ))
        }
    }
}

fn serialize_flatten<S>(
    type_conditions: &Option<TypeConditions>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let tc_string = if let Some(c) = type_conditions {
        format!("|[{}]", c.join(","))
    } else {
        "".to_string()
    };
    let res = format!("@{}", tc_string);
    serializer.serialize_str(res.as_str())
}

fn deserialize_key<'de, D>(deserializer: D) -> Result<(String, Option<TypeConditions>), D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_str(KeyVisitor)
}

struct KeyVisitor;

impl serde::de::Visitor<'_> for KeyVisitor {
    type Value = (String, Option<TypeConditions>);

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a string, potentially followed by type conditions"
        )
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(split_path_element_and_type_conditions(s))
    }
}

fn serialize_key<S>(
    key: &String,
    type_conditions: &Option<TypeConditions>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let tc_string = if let Some(c) = type_conditions {
        format!("|[{}]", c.join(","))
    } else {
        "".to_string()
    };
    let res = format!("{}{}", key, tc_string);
    serializer.serialize_str(res.as_str())
}

fn deserialize_fragment<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_str(FragmentVisitor)
}

struct FragmentVisitor;

impl serde::de::Visitor<'_> for FragmentVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string that begins with '... on '")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        s.strip_prefix(FRAGMENT_PREFIX)
            .map(|v| v.to_string())
            .ok_or_else(|| serde::de::Error::invalid_value(serde::de::Unexpected::Str(s), &self))
    }
}

fn serialize_fragment<S>(name: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(format!("{FRAGMENT_PREFIX}{name}").as_str())
}

/// A path into the result document.
///
/// This can be composed of strings and numbers
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default, Hash)]
#[serde(transparent)]
pub struct Path(pub Vec<PathElement>);

impl Path {
    pub fn iter(&self) -> impl Iterator<Item = &PathElement> {
        self.0.iter()
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for element in self.iter() {
            write!(f, "/")?;
            match element {
                PathElement::Index(index) => write!(f, "{index}")?,
                PathElement::Key(key, type_conditions) => {
                    write!(f, "{key}")?;
                    if let Some(c) = type_conditions {
                        if !c.is_empty() {
                            write!(f, "|[{}]", c.join(","))?;
                        }
                    };
                }
                PathElement::Flatten(type_conditions) => {
                    write!(f, "@")?;
                    if let Some(c) = type_conditions {
                        if !c.is_empty() {
                            write!(f, "|[{}]", c.join(","))?;
                        }
                    };
                }
                PathElement::Fragment(name) => {
                    write!(f, "{FRAGMENT_PREFIX}{name}")?;
                }
            }
        }
        Ok(())
    }
}
