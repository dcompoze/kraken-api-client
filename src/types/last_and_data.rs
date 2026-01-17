//! Custom deserializer for Kraken's pagination format.
//!
//! Many Kraken endpoints return data with a dynamic key and a "last" field for pagination:
//!
//! ```json
//! {
//!     "XBTUSD": [[...trade data...]],
//!     "last": "1234567890"
//! }
//! ```
//!
//! The `LastAndData<T>` type handles this format by parsing any non-"last" key as the data.

use std::fmt;
use std::marker::PhantomData;

use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};

/// A response containing paginated data with a "last" cursor.
///
/// This type handles Kraken's pagination format where the response contains:
/// - A dynamically-named field with the actual data (e.g., "XBTUSD")
/// - A "last" field containing a cursor for the next request
///
/// # Example
///
/// ```rust
/// use kraken_api_client::types::LastAndData;
///
/// #[derive(Debug, serde::Deserialize)]
/// struct Trade {
///     price: String,
///     volume: String,
/// }
///
/// let json = r#"{"XBTUSD": [{"price": "50000", "volume": "1.0"}], "last": "12345"}"#;
/// let result: LastAndData<Vec<Trade>> = serde_json::from_str(json).unwrap();
///
/// assert_eq!(result.last, "12345");
/// assert_eq!(result.data.len(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct LastAndData<T> {
    /// The pagination cursor for the next request.
    pub last: String,
    /// The actual data returned.
    pub data: T,
}

impl<T> LastAndData<T> {
    /// Create a new LastAndData.
    pub fn new(last: impl Into<String>, data: T) -> Self {
        Self {
            last: last.into(),
            data,
        }
    }

    /// Map the data to a different type.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> LastAndData<U> {
        LastAndData {
            last: self.last,
            data: f(self.data),
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for LastAndData<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LastAndDataVisitor<T>(PhantomData<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for LastAndDataVisitor<T> {
            type Value = LastAndData<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with a 'last' key and one data key")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut last: Option<String> = None;
                let mut data: Option<T> = None;

                while let Some(key) = map.next_key::<String>()? {
                    if key == "last" {
                        // Handle both string and numeric "last" values
                        let value: serde_json::Value = map.next_value()?;
                        last = Some(match value {
                            serde_json::Value::String(s) => s,
                            serde_json::Value::Number(n) => n.to_string(),
                            _ => {
                                return Err(de::Error::custom(
                                    "expected string or number for 'last'",
                                ))
                            }
                        });
                    } else {
                        // Any other key is the data
                        data = Some(map.next_value()?);
                    }
                }

                let last = last.ok_or_else(|| de::Error::missing_field("last"))?;
                let data = data.ok_or_else(|| de::Error::custom("missing data field"))?;

                Ok(LastAndData { last, data })
            }
        }

        deserializer.deserialize_map(LastAndDataVisitor(PhantomData))
    }
}

impl<T: serde::Serialize> serde::Serialize for LastAndData<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("data", &self.data)?;
        map.serialize_entry("last", &self.last)?;
        map.end()
    }
}

/// A variant of LastAndData that also captures the key name.
///
/// This is useful when you need to know which asset pair the data is for.
#[derive(Debug, Clone)]
pub struct LastAndDataWithKey<T> {
    /// The key (e.g., "XBTUSD").
    pub key: String,
    /// The pagination cursor for the next request.
    pub last: String,
    /// The actual data returned.
    pub data: T,
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for LastAndDataWithKey<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor<T>(PhantomData<T>);

        impl<'de, T: Deserialize<'de>> de::Visitor<'de> for Visitor<T> {
            type Value = LastAndDataWithKey<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with a 'last' key and one data key")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut last: Option<String> = None;
                let mut data: Option<T> = None;
                let mut key: Option<String> = None;

                while let Some(k) = map.next_key::<String>()? {
                    if k == "last" {
                        let value: serde_json::Value = map.next_value()?;
                        last = Some(match value {
                            serde_json::Value::String(s) => s,
                            serde_json::Value::Number(n) => n.to_string(),
                            _ => {
                                return Err(de::Error::custom(
                                    "expected string or number for 'last'",
                                ))
                            }
                        });
                    } else {
                        key = Some(k);
                        data = Some(map.next_value()?);
                    }
                }

                let key = key.ok_or_else(|| de::Error::custom("missing data key"))?;
                let last = last.ok_or_else(|| de::Error::missing_field("last"))?;
                let data = data.ok_or_else(|| de::Error::custom("missing data field"))?;

                Ok(LastAndDataWithKey { key, last, data })
            }
        }

        deserializer.deserialize_map(Visitor(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn test_last_and_data_string_last() {
        let json = r#"{"XBTUSD": [1, 2, 3], "last": "12345"}"#;
        let result: LastAndData<Vec<i32>> = serde_json::from_str(json).unwrap();
        assert_eq!(result.last, "12345");
        assert_eq!(result.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_last_and_data_numeric_last() {
        let json = r#"{"ETHUSD": ["a", "b"], "last": 67890}"#;
        let result: LastAndData<Vec<String>> = serde_json::from_str(json).unwrap();
        assert_eq!(result.last, "67890");
        assert_eq!(result.data, vec!["a", "b"]);
    }

    #[test]
    fn test_last_and_data_complex() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Item {
            price: String,
            volume: String,
        }

        let json = r#"{"XBTUSD": [{"price": "50000", "volume": "1.0"}], "last": "123"}"#;
        let result: LastAndData<Vec<Item>> = serde_json::from_str(json).unwrap();
        assert_eq!(result.last, "123");
        assert_eq!(result.data.len(), 1);
        assert_eq!(result.data[0].price, "50000");
    }

    #[test]
    fn test_last_and_data_with_key() {
        let json = r#"{"XBTUSD": [1, 2, 3], "last": "12345"}"#;
        let result: LastAndDataWithKey<Vec<i32>> = serde_json::from_str(json).unwrap();
        assert_eq!(result.key, "XBTUSD");
        assert_eq!(result.last, "12345");
        assert_eq!(result.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_last_and_data_map() {
        let result = LastAndData::new("123", vec![1, 2, 3]);
        let mapped = result.map(|v| v.iter().sum::<i32>());
        assert_eq!(mapped.last, "123");
        assert_eq!(mapped.data, 6);
    }
}
