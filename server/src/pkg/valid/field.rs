use regex::Regex;
use validator::ValidationError;

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

pub fn check_sex(sex: &str) -> Result<(), ValidationError> {
    if sex == "male" || sex == "female" {
        return Ok(());
    }
    Err(ValidationError::new("invalid sex"))
}
