use std::fmt;

use serde::de::{
    self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor,
};
use serde::Serialize;
use utoipa::ToSchema;
use validator::Validate;

use slo::regexp::check_order_by;

#[derive(Debug, Serialize, ToSchema)]
pub struct List<T> {
    pub data: Vec<T>,
    pub limit: u64,
    pub offset: u64,
    pub total: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ID {
    pub id: String,
}

#[derive(Debug, Default, Validate, ToSchema)]
pub struct Pagination {
    pub limit: u64,
    pub offset: u64,
    #[validate(custom = "check_order_by")]
    pub order_by: Option<String>,
}

impl<'de> Deserialize<'de> for Pagination {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        enum Field {
            Limit,
            Offset,
            OrderBy,
            Ignore,
        }
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(
                        &self,
                        formatter: &mut fmt::Formatter,
                    ) -> fmt::Result {
                        formatter.write_str("field identifier")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "limit" => Ok(Field::Limit),
                            "offset" => Ok(Field::Offset),
                            "order_by" => Ok(Field::OrderBy),
                            _ => Ok(Field::Ignore),
                        }
                    }
                    fn visit_bytes<E>(
                        self,
                        value: &[u8],
                    ) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            b"limit" => Ok(Field::Limit),
                            b"offset" => Ok(Field::Offset),
                            b"order_by" => Ok(Field::OrderBy),
                            _ => Ok(Field::Ignore),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct PaginationVisitor;

        impl<'de> Visitor<'de> for PaginationVisitor {
            type Value = Pagination;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Pagination")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Pagination, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let limit: String = seq
                    .next_element()?
                    .unwrap_or_else(|| Pagination::DEFAULT_LIMIT.to_string());
                let offset: String = seq
                    .next_element()?
                    .unwrap_or_else(|| Pagination::DEFAULT_OFFSET.to_string());
                let order_by = seq.next_element()?.unwrap_or_else(|| {
                    Some(Pagination::DEFAULT_ORDER_BY.to_string())
                });
                Ok(Pagination {
                    limit: limit.parse().map_err(|err| {
                        de::Error::custom(format_args!(
                            "invalid limit: {}",
                            err
                        ))
                    })?,
                    offset: offset.parse().map_err(|err| {
                        de::Error::custom(format_args!(
                            "invalid offset: {}",
                            err
                        ))
                    })?,
                    order_by,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Pagination, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut limit: Option<String> = None;
                let mut offset: Option<String> = None;
                let mut order_by: Option<Option<String>> = None;
                while let Some(key) = MapAccess::next_key::<Field>(&mut map)? {
                    match key {
                        Field::Limit => {
                            if Option::is_some(&limit) {
                                return Err(
                                    <V::Error as de::Error>::duplicate_field(
                                        "limit",
                                    ),
                                );
                            }
                            limit = Some(MapAccess::next_value::<String>(
                                &mut map,
                            )?);
                        }
                        Field::Offset => {
                            if Option::is_some(&offset) {
                                return Err(
                                    <V::Error as de::Error>::duplicate_field(
                                        "offset",
                                    ),
                                );
                            }
                            offset = Some(MapAccess::next_value::<String>(
                                &mut map,
                            )?);
                        }
                        Field::OrderBy => {
                            if Option::is_some(&order_by) {
                                return Err(
                                    <V::Error as de::Error>::duplicate_field(
                                        "order_by",
                                    ),
                                );
                            }
                            order_by =
                                Some(MapAccess::next_value::<Option<String>>(
                                    &mut map,
                                )?);
                        }
                        _ => {
                            let _ = MapAccess::next_value::<de::IgnoredAny>(
                                &mut map,
                            )?;
                        }
                    }
                }
                // 填写默认值
                let limit = limit
                    .unwrap_or_else(|| Pagination::DEFAULT_LIMIT.to_string());
                let offset = offset
                    .unwrap_or_else(|| Pagination::DEFAULT_OFFSET.to_string());
                let order_by = order_by.unwrap_or_else(|| {
                    Some(Pagination::DEFAULT_ORDER_BY.to_string())
                });
                Ok(Pagination {
                    limit: limit.parse().map_err(|err| {
                        de::Error::custom(format_args!(
                            "invalid limit: {}",
                            err
                        ))
                    })?,
                    offset: offset.parse().map_err(|err| {
                        de::Error::custom(format_args!(
                            "invalid offset: {}",
                            err
                        ))
                    })?,
                    order_by,
                })
            }
        }

        const FIELDS: &[&str] = &["limit", "offset", "order_by"];
        deserializer.deserialize_struct("Pagination", FIELDS, PaginationVisitor)
    }
}

impl Pagination {
    const DEFAULT_LIMIT: &'static str = "20";
    const DEFAULT_OFFSET: &'static str = "0";
    const DEFAULT_ORDER_BY: &'static str = "created_at DESC";

    pub fn convert(&self, wheres: &mut String) {
        if let Some(order_by) = &self.order_by {
            wheres.push_str(" ORDER BY ");
            wheres.push_str(order_by);
        }
        if self.limit > 0 {
            wheres.push_str(" LIMIT ");
            wheres.push_str(self.limit.to_string().as_str());
        }
        if self.offset > 0 {
            wheres.push_str(" OFFSET ");
            wheres.push_str(self.offset.to_string().as_str());
        }
    }
}

#[derive(
    Debug, Clone, Validate, serde::Deserialize, Default, Serialize, ToSchema,
)]
pub struct Claim {
    pub sub: String,
    pub groups: Option<Vec<String>>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub preferred_username: Option<String>,
    pub profile: Option<String>,
    pub picture: Option<String>,
    pub website: Option<String>,
    pub gender: Option<String>,
    pub birthdate: Option<String>,
    pub locale: Option<String>,
    pub phone_number: Option<String>,
    pub phone_number_verified: Option<bool>,
    pub address: Option<String>,
}
