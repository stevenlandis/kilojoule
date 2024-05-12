use super::token::Token;

pub struct TokenGroup<'a> {
    pub tokens: &'a [Token],
}

pub static TOKEN_GROUPS: &[TokenGroup] = &[
    TokenGroup {
        tokens: &[Token::INTEGER],
    },
    TokenGroup { tokens: &[] },
    TokenGroup {
        tokens: &[Token::ASTERISK, Token::END, Token::PLUS],
    },
];
