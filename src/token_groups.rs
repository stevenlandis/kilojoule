use super::token::Token;

pub struct TokenGroup<'a> {
    pub tokens: &'a [Token],
}

pub static TOKEN_GROUPS: &[TokenGroup] = &[
    TokenGroup {
        tokens: &[Token::DOT, Token::FALSE, Token::FLOAT, Token::F_STRING_DOUBLE_QUOTE_LEFT, Token::F_STRING_SINGLE_QUOTE_LEFT, Token::IDENTIFIER, Token::INTEGER, Token::LEFT_BRACE, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::LET, Token::NULL, Token::STRING_DOUBLE_QUOTE, Token::STRING_SINGLE_QUOTE, Token::TRUE],
    },
    TokenGroup {
        tokens: &[Token::END],
    },
    TokenGroup {
        tokens: &[],
    },
    TokenGroup {
        tokens: &[Token::IDENTIFIER],
    },
    TokenGroup {
        tokens: &[Token::EQUAL],
    },
    TokenGroup {
        tokens: &[Token::DOT, Token::FALSE, Token::FLOAT, Token::F_STRING_DOUBLE_QUOTE_LEFT, Token::F_STRING_SINGLE_QUOTE_LEFT, Token::IDENTIFIER, Token::INTEGER, Token::LEFT_BRACE, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::NULL, Token::STRING_DOUBLE_QUOTE, Token::STRING_SINGLE_QUOTE, Token::TRUE],
    },
    TokenGroup {
        tokens: &[Token::PIPE],
    },
    TokenGroup {
        tokens: &[Token::END, Token::PIPE],
    },
    TokenGroup {
        tokens: &[Token::DOUBLE_QUESTION, Token::END, Token::PIPE],
    },
    TokenGroup {
        tokens: &[Token::DOUBLE_QUESTION],
    },
    TokenGroup {
        tokens: &[Token::DOUBLE_QUESTION, Token::END, Token::OR, Token::PIPE],
    },
    TokenGroup {
        tokens: &[Token::OR],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::DOUBLE_QUESTION, Token::END, Token::OR, Token::PIPE],
    },
    TokenGroup {
        tokens: &[Token::AND],
    },
    TokenGroup {
        tokens: &[Token::DOUBLE_EQUALS, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::NOT_EQUALS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::DOUBLE_QUESTION, Token::END, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::MINUS, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOUBLE_QUESTION, Token::END, Token::FORWARD_SLASH, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::ASTERISK, Token::FORWARD_SLASH],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_QUESTION, Token::END, Token::FORWARD_SLASH, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::DOT],
    },
    TokenGroup {
        tokens: &[Token::LEFT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::COLON, Token::DOT, Token::FALSE, Token::FLOAT, Token::FORWARD_SLASH, Token::F_STRING_DOUBLE_QUOTE_LEFT, Token::F_STRING_SINGLE_QUOTE_LEFT, Token::IDENTIFIER, Token::INTEGER, Token::LEFT_BRACE, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::LET, Token::NULL, Token::STRING_DOUBLE_QUOTE, Token::STRING_SINGLE_QUOTE, Token::TRUE],
    },
    TokenGroup {
        tokens: &[Token::COLON, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::COLON, Token::PIPE, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::COLON, Token::DOUBLE_QUESTION, Token::PIPE, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::COLON, Token::DOUBLE_QUESTION, Token::OR, Token::PIPE, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::COLON, Token::DOUBLE_QUESTION, Token::OR, Token::PIPE, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::COLON, Token::DOUBLE_QUESTION, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COLON, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COLON, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::COLON, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COLON, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COLON, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::PIPE, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::DOUBLE_QUESTION, Token::PIPE, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::DOUBLE_QUESTION, Token::OR, Token::PIPE, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::DOUBLE_QUESTION, Token::OR, Token::PIPE, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::DOUBLE_QUESTION, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COLON, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::DOT, Token::FALSE, Token::FLOAT, Token::F_STRING_DOUBLE_QUOTE_LEFT, Token::F_STRING_SINGLE_QUOTE_LEFT, Token::IDENTIFIER, Token::INTEGER, Token::LEFT_BRACE, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::LET, Token::NULL, Token::RIGHT_PAREN, Token::STRING_DOUBLE_QUOTE, Token::STRING_SINGLE_QUOTE, Token::TRUE],
    },
    TokenGroup {
        tokens: &[Token::COMMA, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::COMMA, Token::PIPE, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::COMMA, Token::DOUBLE_QUESTION, Token::PIPE, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::COMMA, Token::DOUBLE_QUESTION, Token::OR, Token::PIPE, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::COMMA, Token::DOUBLE_QUESTION, Token::OR, Token::PIPE, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::COMMA, Token::DOUBLE_QUESTION, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COLON, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::IDENTIFIER, Token::RIGHT_BRACE],
    },
    TokenGroup {
        tokens: &[Token::RIGHT_BRACE],
    },
    TokenGroup {
        tokens: &[Token::COMMA],
    },
    TokenGroup {
        tokens: &[Token::COMMA, Token::RIGHT_BRACE],
    },
    TokenGroup {
        tokens: &[Token::COLON],
    },
    TokenGroup {
        tokens: &[Token::COMMA, Token::PIPE, Token::RIGHT_BRACE],
    },
    TokenGroup {
        tokens: &[Token::COMMA, Token::DOUBLE_QUESTION, Token::PIPE, Token::RIGHT_BRACE],
    },
    TokenGroup {
        tokens: &[Token::COMMA, Token::DOUBLE_QUESTION, Token::OR, Token::PIPE, Token::RIGHT_BRACE],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::COMMA, Token::DOUBLE_QUESTION, Token::OR, Token::PIPE, Token::RIGHT_BRACE],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::COMMA, Token::DOUBLE_QUESTION, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACE],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACE],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACE],
    },
    TokenGroup {
        tokens: &[Token::DOT, Token::FALSE, Token::FLOAT, Token::FORWARD_SLASH, Token::F_STRING_DOUBLE_QUOTE_LEFT, Token::F_STRING_SINGLE_QUOTE_LEFT, Token::IDENTIFIER, Token::INTEGER, Token::LEFT_BRACE, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::LET, Token::NULL, Token::RIGHT_BRACKET, Token::STRING_DOUBLE_QUOTE, Token::STRING_SINGLE_QUOTE, Token::TRUE],
    },
    TokenGroup {
        tokens: &[Token::PIPE, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::DOUBLE_QUESTION, Token::PIPE, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::DOUBLE_QUESTION, Token::OR, Token::PIPE, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::DOUBLE_QUESTION, Token::OR, Token::PIPE, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::DOUBLE_QUESTION, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::DOT, Token::FALSE, Token::FLOAT, Token::F_STRING_DOUBLE_QUOTE_LEFT, Token::F_STRING_SINGLE_QUOTE_LEFT, Token::IDENTIFIER, Token::INTEGER, Token::LEFT_BRACE, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::LET, Token::NULL, Token::RIGHT_BRACKET, Token::STRING_DOUBLE_QUOTE, Token::STRING_SINGLE_QUOTE, Token::TRUE],
    },
    TokenGroup {
        tokens: &[Token::COMMA, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::COMMA, Token::PIPE, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::COMMA, Token::DOUBLE_QUESTION, Token::PIPE, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::COMMA, Token::DOUBLE_QUESTION, Token::OR, Token::PIPE, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::COMMA, Token::DOUBLE_QUESTION, Token::OR, Token::PIPE, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::COMMA, Token::DOUBLE_QUESTION, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::F_STRING_SINGLE_QUOTE_MIDDLE, Token::F_STRING_SINGLE_QUOTE_RIGHT],
    },
    TokenGroup {
        tokens: &[Token::F_STRING_SINGLE_QUOTE_MIDDLE, Token::F_STRING_SINGLE_QUOTE_RIGHT, Token::PIPE],
    },
    TokenGroup {
        tokens: &[Token::DOUBLE_QUESTION, Token::F_STRING_SINGLE_QUOTE_MIDDLE, Token::F_STRING_SINGLE_QUOTE_RIGHT, Token::PIPE],
    },
    TokenGroup {
        tokens: &[Token::DOUBLE_QUESTION, Token::F_STRING_SINGLE_QUOTE_MIDDLE, Token::F_STRING_SINGLE_QUOTE_RIGHT, Token::OR, Token::PIPE],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::DOUBLE_QUESTION, Token::F_STRING_SINGLE_QUOTE_MIDDLE, Token::F_STRING_SINGLE_QUOTE_RIGHT, Token::OR, Token::PIPE],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::DOUBLE_QUESTION, Token::F_STRING_SINGLE_QUOTE_MIDDLE, Token::F_STRING_SINGLE_QUOTE_RIGHT, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::F_STRING_SINGLE_QUOTE_MIDDLE, Token::F_STRING_SINGLE_QUOTE_RIGHT, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::F_STRING_SINGLE_QUOTE_MIDDLE, Token::F_STRING_SINGLE_QUOTE_RIGHT, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::F_STRING_DOUBLE_QUOTE_MIDDLE, Token::F_STRING_DOUBLE_QUOTE_RIGHT],
    },
    TokenGroup {
        tokens: &[Token::F_STRING_DOUBLE_QUOTE_MIDDLE, Token::F_STRING_DOUBLE_QUOTE_RIGHT, Token::PIPE],
    },
    TokenGroup {
        tokens: &[Token::DOUBLE_QUESTION, Token::F_STRING_DOUBLE_QUOTE_MIDDLE, Token::F_STRING_DOUBLE_QUOTE_RIGHT, Token::PIPE],
    },
    TokenGroup {
        tokens: &[Token::DOUBLE_QUESTION, Token::F_STRING_DOUBLE_QUOTE_MIDDLE, Token::F_STRING_DOUBLE_QUOTE_RIGHT, Token::OR, Token::PIPE],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::DOUBLE_QUESTION, Token::F_STRING_DOUBLE_QUOTE_MIDDLE, Token::F_STRING_DOUBLE_QUOTE_RIGHT, Token::OR, Token::PIPE],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::DOUBLE_QUESTION, Token::F_STRING_DOUBLE_QUOTE_MIDDLE, Token::F_STRING_DOUBLE_QUOTE_RIGHT, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::F_STRING_DOUBLE_QUOTE_MIDDLE, Token::F_STRING_DOUBLE_QUOTE_RIGHT, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::F_STRING_DOUBLE_QUOTE_MIDDLE, Token::F_STRING_DOUBLE_QUOTE_RIGHT, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::COMMA, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::COMMA, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::F_STRING_SINGLE_QUOTE_MIDDLE, Token::F_STRING_SINGLE_QUOTE_RIGHT, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::F_STRING_SINGLE_QUOTE_MIDDLE, Token::F_STRING_SINGLE_QUOTE_RIGHT, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::F_STRING_SINGLE_QUOTE_MIDDLE, Token::F_STRING_SINGLE_QUOTE_RIGHT, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::F_STRING_DOUBLE_QUOTE_MIDDLE, Token::F_STRING_DOUBLE_QUOTE_RIGHT, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::F_STRING_DOUBLE_QUOTE_MIDDLE, Token::F_STRING_DOUBLE_QUOTE_RIGHT, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::F_STRING_DOUBLE_QUOTE_MIDDLE, Token::F_STRING_DOUBLE_QUOTE_RIGHT, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::F_STRING_DOUBLE_QUOTE_MIDDLE, Token::F_STRING_DOUBLE_QUOTE_RIGHT, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::F_STRING_SINGLE_QUOTE_MIDDLE, Token::F_STRING_SINGLE_QUOTE_RIGHT, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::F_STRING_DOUBLE_QUOTE_MIDDLE, Token::F_STRING_DOUBLE_QUOTE_RIGHT, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::F_STRING_SINGLE_QUOTE_MIDDLE, Token::F_STRING_SINGLE_QUOTE_RIGHT, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::F_STRING_SINGLE_QUOTE_RIGHT],
    },
    TokenGroup {
        tokens: &[Token::F_STRING_SINGLE_QUOTE_MIDDLE],
    },
    TokenGroup {
        tokens: &[Token::F_STRING_DOUBLE_QUOTE_RIGHT],
    },
    TokenGroup {
        tokens: &[Token::F_STRING_DOUBLE_QUOTE_MIDDLE],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::F_STRING_DOUBLE_QUOTE_MIDDLE, Token::F_STRING_DOUBLE_QUOTE_RIGHT, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::F_STRING_DOUBLE_QUOTE_MIDDLE, Token::F_STRING_DOUBLE_QUOTE_RIGHT, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::DOUBLE_QUESTION, Token::PIPE],
    },
    TokenGroup {
        tokens: &[Token::DOUBLE_QUESTION, Token::OR, Token::PIPE],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::DOUBLE_QUESTION, Token::OR, Token::PIPE],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::DOUBLE_QUESTION, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::F_STRING_SINGLE_QUOTE_MIDDLE, Token::F_STRING_SINGLE_QUOTE_RIGHT, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::F_STRING_SINGLE_QUOTE_MIDDLE, Token::F_STRING_SINGLE_QUOTE_RIGHT, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACE],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACE],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::COMMA, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACE],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACE],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACE],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACE],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACE],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COMMA, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_PAREN],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COLON, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::COLON, Token::DOT, Token::DOUBLE_QUESTION, Token::FORWARD_SLASH, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS, Token::RIGHT_BRACKET],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_QUESTION, Token::END, Token::FORWARD_SLASH, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_QUESTION, Token::END, Token::FORWARD_SLASH, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::MINUS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::END, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::END, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::END, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::END, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::LEFT_BRACKET, Token::LEFT_PAREN, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
    TokenGroup {
        tokens: &[Token::AND, Token::ASTERISK, Token::DOT, Token::DOUBLE_EQUALS, Token::DOUBLE_QUESTION, Token::END, Token::FORWARD_SLASH, Token::GREATER_THAN, Token::GREATER_THAN_OR_EQUAL, Token::IDENTIFIER, Token::LEFT_BRACKET, Token::LESS_THAN, Token::LESS_THAN_OR_EQUAL, Token::MINUS, Token::NOT_EQUALS, Token::OR, Token::PIPE, Token::PLUS],
    },
];
