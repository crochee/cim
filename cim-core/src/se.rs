use std::str::FromStr;

use serde::de::Error;

pub fn from_str<'de, D, S>(deserializer: D) -> Result<S, D::Error>
where
    D: serde::Deserializer<'de>,
    S: FromStr,
{
    let s = <&str as serde::Deserialize>::deserialize(deserializer)?;
    S::from_str(s)
        .map_err(|_| D::Error::custom(format!("could not parse string {}", s)))
}

pub fn from_str_option<'de, D, S>(
    deserializer: D,
) -> Result<Option<S>, D::Error>
where
    D: serde::Deserializer<'de>,
    S: FromStr,
{
    let s = <Option<&str> as serde::Deserialize>::deserialize(deserializer)?;
    match s {
        Some(value) => {
            let v = S::from_str(value).map_err(|_| {
                D::Error::custom(format!("could not parse string {}", value))
            })?;
            Ok(Some(v))
        }
        None => Ok(None),
    }
}
