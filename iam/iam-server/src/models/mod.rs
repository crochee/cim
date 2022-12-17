pub mod condition;
pub mod policy;
pub mod req;
pub mod tag;

use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use cim_core::se::from_str;

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

lazy_static::lazy_static! {
    static ref SORT_REGEX: Regex = Regex::new(
        r"^[a-z][a-z_]{0,30}[a-z](\s(asc|ASC|desc|DESC))?(,[a-z][a-z_]{0,30}[a-z](\s(asc|ASC|desc|DESC))?)*$",
    ).unwrap();

    static ref PASSWORD_REGEX: Regex = Regex::new(
        r"^[a-zA-Z][a-zA-Z0-9_#@\$]{14,254}$",
    ).unwrap();
}

// 以字母开头，需要包含数字，字母，特殊字符（_,#,@,$）之一，长度不少于15位，最大不超过255位
pub fn check_password(password: &str) -> Result<(), ValidationError> {
    if PASSWORD_REGEX.is_match(password) {
        return Ok(());
    }
    Err(ValidationError::new("invalid password"))
}

pub fn check_sort(sort: &str) -> Result<(), ValidationError> {
    if SORT_REGEX.is_match(sort) {
        return Ok(());
    }
    Err(ValidationError::new("invalid sort"))
}
