pub fn quote_to(buf: &mut String, s: &str) {
    let mut under_quoted = false;
    let mut self_quoted = false;
    let mut continuous_backtick = 0;
    let mut shift_delimiter = 0;
    for v in s.bytes() {
        match v {
            b'`' => {
                continuous_backtick += 1;
                if continuous_backtick == 2 {
                    buf.push_str("``");
                    continuous_backtick = 0;
                }
            }
            b'.' => {
                if continuous_backtick > 0 || !self_quoted {
                    shift_delimiter = 0;
                    under_quoted = false;
                    continuous_backtick = 0;
                    buf.push('`');
                };
                buf.push('.');
                continue;
            }
            _ => {
                if shift_delimiter - continuous_backtick <= 0 && !under_quoted {
                    buf.push('`');
                    under_quoted = true;
                    self_quoted = continuous_backtick > 0;
                    if self_quoted {
                        continuous_backtick -= 1;
                    }
                }
                while continuous_backtick > 0 {
                    continuous_backtick -= 1;
                    buf.push_str("``");
                }
                buf.push(v as char);
            }
        }
        shift_delimiter += 1;
    }

    if continuous_backtick > 0 && !self_quoted {
        buf.push_str("``");
    }
    buf.push('`');
}

pub fn convert_param(buf: &mut String, s: &str) {
    buf.push('\'');
    let mut last_end = 0;
    for (start, part) in s.match_indices('\'') {
        buf.push_str(unsafe { s.get_unchecked(last_end..start) });
        buf.push_str("\\'");
        last_end = start + part.len();
    }
    buf.push_str(unsafe { s.get_unchecked(last_end..s.len()) });
    buf.push('\'');
}

pub fn convert_field(s: &str) -> String {
    let mut buf = String::new();
    convert_param(&mut buf, s);
    buf
}

pub fn convert_option_field(s: &Option<String>) -> Option<String> {
    s.as_ref().map(|s| convert_field(s))
}

pub fn update_set_param(buf: &mut String, name: &str, value: &Option<String>) {
    if let Some(val) = value {
        if !buf.is_empty() {
            buf.push_str(" , ");
        }
        buf.push_str(name);
        convert_param(buf, val);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_to() {
        let mut writer = String::new();

        writer.clear();
        quote_to(&mut writer, "datadase.tableUser");
        assert_eq!(writer, "`datadase`.`tableUser`");

        writer.clear();
        quote_to(&mut writer, "datadase.table`User");
        assert_eq!(writer, "`datadase`.`table``User`");

        writer.clear();
        quote_to(&mut writer, "`a`.`b`");
        assert_eq!(writer, "`a`.`b`");

        writer.clear();
        quote_to(&mut writer, "`a`.b`");
        assert_eq!(writer, "`a`.`b```");

        writer.clear();
        quote_to(&mut writer, "a.`b`");
        assert_eq!(writer, "`a`.`b`");

        writer.clear();
        quote_to(&mut writer, "`a`.b`c");
        assert_eq!(writer, "`a`.`b``c`");

        writer.clear();
        quote_to(&mut writer, "`a`.`b`c`");
        assert_eq!(writer, "`a`.`b``c`");

        writer.clear();
        quote_to(&mut writer, "`a`.b");
        assert_eq!(writer, "`a`.`b`");

        writer.clear();
        quote_to(&mut writer, "`ab`");
        assert_eq!(writer, "`ab`");

        writer.clear();
        quote_to(&mut writer, "`a``b`");
        assert_eq!(writer, "`a``b`");

        writer.clear();
        quote_to(&mut writer, "`a```b`");
        assert_eq!(writer, "`a````b`");

        writer.clear();
        quote_to(&mut writer, "a`b");
        assert_eq!(writer, "`a``b`");

        writer.clear();
        quote_to(&mut writer, "ab");
        assert_eq!(writer, "`ab`");

        writer.clear();
        quote_to(&mut writer, "`a.b`");
        assert_eq!(writer, "`a.b`");

        writer.clear();
        quote_to(&mut writer, "a.b");
        assert_eq!(writer, "`a`.`b`");
    }

    #[test]
    fn test_convert_param() {
        let mut writer = String::new();
        writer.clear();
        convert_param(&mut writer, "gs");
        assert_eq!(writer, "'gs'");

        writer.clear();
        convert_param(&mut writer, "g's");
        assert_eq!(writer, "'g\\'s'");

        writer.clear();
        convert_param(&mut writer, "g'sgs a'op");
        assert_eq!(writer, "'g\\'sgs a\\'op'");
    }
}
