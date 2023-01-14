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
