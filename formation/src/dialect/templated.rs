use sqlparser::dialect::Dialect;

#[derive(Debug)]
pub struct TemplatedDialect {}

impl Dialect for TemplatedDialect {
    fn is_identifier_start(&self, ch: char) -> bool {
        ('a'..='z').contains(&ch)
            || ('A'..='Z').contains(&ch)
            || ch == '_'
            || ch == '#'
            || ch == '@'
            || ch == '{'
    }

    fn is_identifier_part(&self, ch: char) -> bool {
        ('a'..='z').contains(&ch)
            || ('A'..='Z').contains(&ch)
            || ('0'..='9').contains(&ch)
            || ch == '@'
            || ch == '$'
            || ch == '#'
            || ch == '_'
            || ch == '{'
            || ch == '}'
    }
}
