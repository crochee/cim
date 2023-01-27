pub mod condition;
pub mod policy;
pub mod req;
pub mod tag;
pub mod user;

use serde::{Deserialize, Serialize};
use validator::Validate;

use cim_core::se::from_str;

use crate::pkg::valid::field::check_sort;

#[derive(Debug, Serialize)]
pub struct List<T> {
    pub data: Vec<T>,
    pub limit: u64,
    pub offset: u64,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct ID {
    pub id: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Validate)]
pub struct Pagination {
    #[serde(deserialize_with = "from_str")]
    pub limit: u64,
    #[serde(deserialize_with = "from_str")]
    pub offset: u64,
    #[validate(custom = "check_sort")]
    pub sort: Option<String>,
}

impl Pagination {
    const DEFAULT_LIMIT: u64 = 20;
    const DEFAULT_OFFSET: u64 = 0;

    pub fn check(&mut self) {
        if self.sort.is_none() {
            self.sort = Some("`created_at` DESC".to_string())
        };
        if self.limit == 0 {
            self.limit = Self::DEFAULT_LIMIT
        };
        if self.offset == 0 {
            self.offset = Self::DEFAULT_OFFSET
        };
    }
}

impl ToString for Pagination {
    fn to_string(&self) -> String {
        let mut wheres = String::from("");
        if let Some(sort) = &self.sort {
            wheres.push_str(format!(" ORDER BY {}", sort).as_str());
        }
        if self.limit > 0 {
            wheres.push_str(format!(" LIMIT {}", self.limit).as_str());
        }
        if self.offset > 0 {
            wheres.push_str(format!(" OFFSET {}", self.offset).as_str());
        }
        wheres
    }
}
