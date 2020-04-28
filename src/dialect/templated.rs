use sqlparser::dialect::Dialect;

#[derive(Debug)]
pub struct TemplatedDialect {}

impl Dialect for TemplatedDialect {
    fn is_identifier_start(&self, ch: char) -> bool {
        (ch >= 'a' && ch <= 'z')
            || (ch >= 'A' && ch <= 'Z')
            || ch == '_'
            || ch == '#'
            || ch == '@'
            || ch == '{'
    }

    fn is_identifier_part(&self, ch: char) -> bool {
        (ch >= 'a' && ch <= 'z')
            || (ch >= 'A' && ch <= 'Z')
            || (ch >= '0' && ch <= '9')
            || ch == '@'
            || ch == '$'
            || ch == '#'
            || ch == '_'
            || ch == '{'
            || ch == '}'
    }
}
