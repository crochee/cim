use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

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

#[derive(Debug, Default, Deserialize, Validate)]
pub struct Pagination {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    #[validate(custom = "check_sort")]
    pub sort: Option<String>,
}

impl Pagination {
    const DEFAULT_LIMIT: u64 = 20;
    const DEFAULT_OFFSET: u64 = 0;

    pub fn check(&mut self) {
        if None == self.sort {
            self.sort = Some("`created_at` desc".to_string());
        };
        self.limit = match self.limit {
            Some(mut temp_limit) => {
                if temp_limit == 0 {
                    temp_limit = Self::DEFAULT_LIMIT;
                }
                Some(temp_limit)
            }
            None => Some(Self::DEFAULT_LIMIT),
        };
        self.offset = match self.offset {
            Some(mut temp_offset) => {
                if temp_offset == 0 {
                    temp_offset = Self::DEFAULT_OFFSET;
                }
                Some(temp_offset)
            }
            None => Some(Self::DEFAULT_OFFSET),
        };
    }
}

impl ToString for Pagination {
    fn to_string(&self) -> String {
        let mut wheres = String::from("");
        if let Some(sort) = &self.sort {
            wheres.push_str(format!(" ORDER BY {}", sort).as_str());
        }
        if let Some(limit) = self.limit {
            if limit > 0 {
                wheres.push_str(format!(" LIMIT {}", limit).as_str());
            }
        }
        if let Some(offset) = self.offset {
            if offset > 0 {
                wheres.push_str(format!(" OFFSET {}", offset).as_str());
            }
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
