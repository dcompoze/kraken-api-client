//! Custom serde helpers for Kraken's quirky serialization formats.
//!
//! Kraken's API uses various non-standard serialization formats that require
//! custom helpers. These modules provide reusable serde helpers.

use std::collections::BTreeSet;
use std::fmt::{self, Display};
use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serializer};

/// Serialize/deserialize a `BTreeSet<T>` as a comma-separated string.
///
/// # Example
///
/// ```rust
/// use std::collections::BTreeSet;
/// use serde::{Serialize, Deserialize};
/// use kraken_api_client::types::serde_helpers::comma_separated;
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq)]
/// struct Request {
///     #[serde(with = "comma_separated")]
///     flags: BTreeSet<String>,
/// }
///
/// let request = Request {
///     flags: ["post", "nompp"].iter().map(|s| s.to_string()).collect(),
/// };
///
/// let json = serde_json::to_string(&request).unwrap();
/// assert_eq!(json, r#"{"flags":"nompp,post"}"#); // BTreeSet sorts alphabetically
/// ```
pub mod comma_separated {
    use super::*;

    /// Serialize a BTreeSet as a comma-separated string.
    pub fn serialize<T, S>(set: &BTreeSet<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        let s = set
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",");
        serializer.serialize_str(&s)
    }

    /// Deserialize a comma-separated string into a BTreeSet.
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<BTreeSet<T>, D::Error>
    where
        T: FromStr + Ord,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            return Ok(BTreeSet::new());
        }
        s.split(',')
            .map(|part| part.trim().parse().map_err(de::Error::custom))
            .collect()
    }
}

/// Serialize/deserialize a type using its Display/FromStr implementations.
///
/// This is useful for types that Kraken wants as strings (e.g., booleans as "true"/"false").
///
/// # Example
///
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use kraken_api_client::types::serde_helpers::display_fromstr;
///
/// #[derive(Serialize, Deserialize, Debug)]
/// struct Request {
///     #[serde(with = "display_fromstr")]
///     validate: bool, // Serializes as "true"/"false" string
/// }
///
/// let request = Request { validate: true };
/// let json = serde_json::to_string(&request).unwrap();
/// assert_eq!(json, r#"{"validate":"true"}"#);
/// ```
pub mod display_fromstr {
    use super::*;

    /// Serialize using Display trait.
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    /// Deserialize using FromStr trait.
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(de::Error::custom)
    }
}

/// Deserialize to `None` instead of failing on invalid/unexpected data.
///
/// This is useful for fields that Kraken sometimes returns with unexpected types or formats.
///
/// # Example
///
/// ```rust
/// use serde::Deserialize;
/// use rust_decimal::Decimal;
/// use kraken_api_client::types::serde_helpers::default_on_error;
///
/// #[derive(Deserialize, Debug)]
/// struct Response {
///     #[serde(deserialize_with = "default_on_error::deserialize", default)]
///     leverage: Option<Decimal>,
/// }
///
/// // Even with invalid data, deserialization succeeds with None
/// let json = r#"{"leverage":"invalid"}"#;
/// let response: Response = serde_json::from_str(json).unwrap();
/// assert!(response.leverage.is_none());
/// ```
pub mod default_on_error {
    use super::*;

    /// Deserialize a value, returning None if deserialization fails.
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        Ok(T::deserialize(deserializer).ok())
    }
}

/// Helper for deserializing Kraken's deposit/withdrawal limit field.
///
/// Kraken returns either `"limit": false` or `"limit": "100.0"`.
///
/// # Example
///
/// ```rust
/// use serde::Deserialize;
/// use rust_decimal::Decimal;
/// use kraken_api_client::types::serde_helpers::maybe_decimal;
///
/// #[derive(Deserialize, Debug)]
/// struct DepositMethod {
///     #[serde(deserialize_with = "maybe_decimal::deserialize", default)]
///     limit: Option<Decimal>,
/// }
///
/// // With false
/// let json = r#"{"limit":false}"#;
/// let method: DepositMethod = serde_json::from_str(json).unwrap();
/// assert!(method.limit.is_none());
///
/// // With string decimal
/// let json = r#"{"limit":"100.0"}"#;
/// let method: DepositMethod = serde_json::from_str(json).unwrap();
/// assert_eq!(method.limit.unwrap().to_string(), "100.0");
/// ```
pub mod maybe_decimal {
    use super::*;
    use rust_decimal::Decimal;

    /// Deserialize a value that may be `false` or a decimal string.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MaybeDecimalVisitor;

        impl<'de> de::Visitor<'de> for MaybeDecimalVisitor {
            type Value = Option<Decimal>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a decimal string or false")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if v {
                    Err(de::Error::custom("expected false or decimal string"))
                } else {
                    Ok(None)
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                v.parse().map(Some).map_err(de::Error::custom)
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&v)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(None)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(None)
            }
        }

        deserializer.deserialize_any(MaybeDecimalVisitor)
    }
}

/// Helper for empty strings that should be deserialized as None.
///
/// Some Kraken fields return `""` instead of null.
///
/// # Example
///
/// ```rust
/// use serde::Deserialize;
/// use kraken_api_client::types::serde_helpers::empty_string_as_none;
///
/// #[derive(Deserialize, Debug)]
/// struct Response {
///     #[serde(deserialize_with = "empty_string_as_none::deserialize", default)]
///     refid: Option<String>,
/// }
///
/// let json = r#"{"refid":""}"#;
/// let response: Response = serde_json::from_str(json).unwrap();
/// assert!(response.refid.is_none());
///
/// let json = r#"{"refid":"ABC123"}"#;
/// let response: Response = serde_json::from_str(json).unwrap();
/// assert_eq!(response.refid.unwrap(), "ABC123");
/// ```
pub mod empty_string_as_none {
    use super::*;

    /// Deserialize a string, returning None if empty.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = Option::<String>::deserialize(deserializer)?;
        Ok(s.filter(|s| !s.is_empty()))
    }
}

/// Optional comma-separated helper for Option<BTreeSet<T>>.
///
/// Uses with comma_separated but handles the Option wrapper.
pub mod optional_comma_separated {
    use super::*;

    /// Serialize an Option<BTreeSet> as a comma-separated string or skip if None.
    pub fn serialize<T, S>(set: &Option<BTreeSet<T>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        match set {
            Some(set) => comma_separated::serialize(set, serializer),
            None => serializer.serialize_none(),
        }
    }

    /// Deserialize an optional comma-separated string.
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<BTreeSet<T>>, D::Error>
    where
        T: FromStr + Ord,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) if !s.is_empty() => {
                let set: Result<BTreeSet<T>, _> = s
                    .split(',')
                    .map(|part| part.trim().parse().map_err(de::Error::custom))
                    .collect();
                set.map(Some)
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;

    #[test]
    fn test_comma_separated_serialize() {
        #[derive(Serialize)]
        struct Test {
            #[serde(with = "comma_separated")]
            flags: BTreeSet<String>,
        }

        let test = Test {
            flags: ["a", "b", "c"].iter().map(|s| s.to_string()).collect(),
        };
        let json = serde_json::to_string(&test).unwrap();
        assert_eq!(json, r#"{"flags":"a,b,c"}"#);
    }

    #[test]
    fn test_comma_separated_deserialize() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Test {
            #[serde(with = "comma_separated")]
            flags: BTreeSet<String>,
        }

        let json = r#"{"flags":"a,b,c"}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.flags.len(), 3);
        assert!(test.flags.contains("a"));
        assert!(test.flags.contains("b"));
        assert!(test.flags.contains("c"));
    }

    #[test]
    fn test_comma_separated_empty() {
        #[derive(Deserialize, Debug)]
        struct Test {
            #[serde(with = "comma_separated")]
            flags: BTreeSet<String>,
        }

        let json = r#"{"flags":""}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert!(test.flags.is_empty());
    }

    #[test]
    fn test_display_fromstr_serialize() {
        #[derive(Serialize)]
        struct Test {
            #[serde(with = "display_fromstr")]
            validate: bool,
        }

        let test = Test { validate: true };
        let json = serde_json::to_string(&test).unwrap();
        assert_eq!(json, r#"{"validate":"true"}"#);
    }

    #[test]
    fn test_display_fromstr_deserialize() {
        #[derive(Deserialize, Debug)]
        struct Test {
            #[serde(with = "display_fromstr")]
            validate: bool,
        }

        let json = r#"{"validate":"true"}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert!(test.validate);

        let json = r#"{"validate":"false"}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert!(!test.validate);
    }

    #[test]
    fn test_default_on_error_invalid() {
        #[derive(Deserialize, Debug)]
        struct Test {
            #[serde(deserialize_with = "default_on_error::deserialize", default)]
            value: Option<i32>,
        }

        let json = r#"{"value":"not_a_number"}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert!(test.value.is_none());
    }

    #[test]
    fn test_default_on_error_valid() {
        #[derive(Deserialize, Debug)]
        struct Test {
            #[serde(deserialize_with = "default_on_error::deserialize", default)]
            value: Option<i32>,
        }

        let json = r#"{"value":42}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.value, Some(42));
    }

    #[test]
    fn test_maybe_decimal_false() {
        #[derive(Deserialize, Debug)]
        struct Test {
            #[serde(deserialize_with = "maybe_decimal::deserialize", default)]
            limit: Option<Decimal>,
        }

        let json = r#"{"limit":false}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert!(test.limit.is_none());
    }

    #[test]
    fn test_maybe_decimal_string() {
        #[derive(Deserialize, Debug)]
        struct Test {
            #[serde(deserialize_with = "maybe_decimal::deserialize", default)]
            limit: Option<Decimal>,
        }

        let json = r#"{"limit":"100.50"}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.limit.unwrap(), Decimal::from_str("100.50").unwrap());
    }

    #[test]
    fn test_empty_string_as_none() {
        #[derive(Deserialize, Debug)]
        struct Test {
            #[serde(deserialize_with = "empty_string_as_none::deserialize", default)]
            refid: Option<String>,
        }

        let json = r#"{"refid":""}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert!(test.refid.is_none());

        let json = r#"{"refid":"ABC123"}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.refid.unwrap(), "ABC123");
    }
}
