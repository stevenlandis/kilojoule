use super::rule_type::RuleType;
use super::token::Token;

pub struct Rule<'a> {
    pub rule_type: RuleType,
    pub token: Token,
    pub steps: &'a [Token],
}

pub static RULES: &[Rule] = &[
    Rule {
        rule_type: RuleType::main__expr_END,
        token: Token::main,
        steps: &[Token::expr,Token::END],
    },
    Rule {
        rule_type: RuleType::expr__assignExpr,
        token: Token::expr,
        steps: &[Token::assignExpr],
    },
    Rule {
        rule_type: RuleType::assignExpr__opPipeExpr,
        token: Token::assignExpr,
        steps: &[Token::opPipeExpr],
    },
    Rule {
        rule_type: RuleType::assignExpr__LET_IDENTIFIER_EQUAL_opCoalesceExpr_PIPE_assignExpr,
        token: Token::assignExpr,
        steps: &[Token::LET,Token::IDENTIFIER,Token::EQUAL,Token::opCoalesceExpr,Token::PIPE,Token::assignExpr],
    },
    Rule {
        rule_type: RuleType::assignExpr__opPipeExpr_PIPE_LET_IDENTIFIER_EQUAL_opCoalesceExpr_PIPE_assignExpr,
        token: Token::assignExpr,
        steps: &[Token::opPipeExpr,Token::PIPE,Token::LET,Token::IDENTIFIER,Token::EQUAL,Token::opCoalesceExpr,Token::PIPE,Token::assignExpr],
    },
    Rule {
        rule_type: RuleType::opPipeExpr__opCoalesceExpr,
        token: Token::opPipeExpr,
        steps: &[Token::opCoalesceExpr],
    },
    Rule {
        rule_type: RuleType::opPipeExpr__opPipeExpr_PIPE_opCoalesceExpr,
        token: Token::opPipeExpr,
        steps: &[Token::opPipeExpr,Token::PIPE,Token::opCoalesceExpr],
    },
    Rule {
        rule_type: RuleType::opCoalesceExpr__opOrExpr,
        token: Token::opCoalesceExpr,
        steps: &[Token::opOrExpr],
    },
    Rule {
        rule_type: RuleType::opCoalesceExpr__opCoalesceExpr_DOUBLE_QUESTION_opOrExpr,
        token: Token::opCoalesceExpr,
        steps: &[Token::opCoalesceExpr,Token::DOUBLE_QUESTION,Token::opOrExpr],
    },
    Rule {
        rule_type: RuleType::opOrExpr__opAndExpr,
        token: Token::opOrExpr,
        steps: &[Token::opAndExpr],
    },
    Rule {
        rule_type: RuleType::opOrExpr__opOrExpr_OR_opAndExpr,
        token: Token::opOrExpr,
        steps: &[Token::opOrExpr,Token::OR,Token::opAndExpr],
    },
    Rule {
        rule_type: RuleType::opAndExpr__opEqualityExpr,
        token: Token::opAndExpr,
        steps: &[Token::opEqualityExpr],
    },
    Rule {
        rule_type: RuleType::opAndExpr__opAndExpr_AND_opEqualityExpr,
        token: Token::opAndExpr,
        steps: &[Token::opAndExpr,Token::AND,Token::opEqualityExpr],
    },
    Rule {
        rule_type: RuleType::equalityOperator__DOUBLE_EQUALS,
        token: Token::equalityOperator,
        steps: &[Token::DOUBLE_EQUALS],
    },
    Rule {
        rule_type: RuleType::equalityOperator__NOT_EQUALS,
        token: Token::equalityOperator,
        steps: &[Token::NOT_EQUALS],
    },
    Rule {
        rule_type: RuleType::equalityOperator__LESS_THAN,
        token: Token::equalityOperator,
        steps: &[Token::LESS_THAN],
    },
    Rule {
        rule_type: RuleType::equalityOperator__LESS_THAN_OR_EQUAL,
        token: Token::equalityOperator,
        steps: &[Token::LESS_THAN_OR_EQUAL],
    },
    Rule {
        rule_type: RuleType::equalityOperator__GREATER_THAN,
        token: Token::equalityOperator,
        steps: &[Token::GREATER_THAN],
    },
    Rule {
        rule_type: RuleType::equalityOperator__GREATER_THAN_OR_EQUAL,
        token: Token::equalityOperator,
        steps: &[Token::GREATER_THAN_OR_EQUAL],
    },
    Rule {
        rule_type: RuleType::opEqualityExpr__opAddExpr,
        token: Token::opEqualityExpr,
        steps: &[Token::opAddExpr],
    },
    Rule {
        rule_type: RuleType::opEqualityExpr__opAddExpr_equalityOperator_opAddExpr,
        token: Token::opEqualityExpr,
        steps: &[Token::opAddExpr,Token::equalityOperator,Token::opAddExpr],
    },
    Rule {
        rule_type: RuleType::opAddOperator__PLUS,
        token: Token::opAddOperator,
        steps: &[Token::PLUS],
    },
    Rule {
        rule_type: RuleType::opAddOperator__MINUS,
        token: Token::opAddOperator,
        steps: &[Token::MINUS],
    },
    Rule {
        rule_type: RuleType::opAddExpr__opMulExpr,
        token: Token::opAddExpr,
        steps: &[Token::opMulExpr],
    },
    Rule {
        rule_type: RuleType::opAddExpr__opAddExpr_opAddOperator_opMulExpr,
        token: Token::opAddExpr,
        steps: &[Token::opAddExpr,Token::opAddOperator,Token::opMulExpr],
    },
    Rule {
        rule_type: RuleType::opMulOperator__ASTERISK,
        token: Token::opMulOperator,
        steps: &[Token::ASTERISK],
    },
    Rule {
        rule_type: RuleType::opMulOperator__FORWARD_SLASH,
        token: Token::opMulOperator,
        steps: &[Token::FORWARD_SLASH],
    },
    Rule {
        rule_type: RuleType::opMulExpr__opAccessExpr,
        token: Token::opMulExpr,
        steps: &[Token::opAccessExpr],
    },
    Rule {
        rule_type: RuleType::opMulExpr__opMulExpr_opMulOperator_opAccessExpr,
        token: Token::opMulExpr,
        steps: &[Token::opMulExpr,Token::opMulOperator,Token::opAccessExpr],
    },
    Rule {
        rule_type: RuleType::opAccessExpr__baseExpr,
        token: Token::opAccessExpr,
        steps: &[Token::baseExpr],
    },
    Rule {
        rule_type: RuleType::opAccessExpr__opAccessExpr_DOT_IDENTIFIER,
        token: Token::opAccessExpr,
        steps: &[Token::opAccessExpr,Token::DOT,Token::IDENTIFIER],
    },
    Rule {
        rule_type: RuleType::opAccessExpr__opAccessExpr_LEFT_BRACKET_listAccessExpr_RIGHT_BRACKET,
        token: Token::opAccessExpr,
        steps: &[Token::opAccessExpr,Token::LEFT_BRACKET,Token::listAccessExpr,Token::RIGHT_BRACKET],
    },
    Rule {
        rule_type: RuleType::baseExpr__baseDotExpr,
        token: Token::baseExpr,
        steps: &[Token::baseDotExpr],
    },
    Rule {
        rule_type: RuleType::baseExpr__baseDotAccess,
        token: Token::baseExpr,
        steps: &[Token::baseDotAccess],
    },
    Rule {
        rule_type: RuleType::baseExpr__mapExpr,
        token: Token::baseExpr,
        steps: &[Token::mapExpr],
    },
    Rule {
        rule_type: RuleType::baseExpr__listExpr,
        token: Token::baseExpr,
        steps: &[Token::listExpr],
    },
    Rule {
        rule_type: RuleType::baseExpr__LEFT_PAREN_expr_RIGHT_PAREN,
        token: Token::baseExpr,
        steps: &[Token::LEFT_PAREN,Token::expr,Token::RIGHT_PAREN],
    },
    Rule {
        rule_type: RuleType::baseExpr__INTEGER,
        token: Token::baseExpr,
        steps: &[Token::INTEGER],
    },
    Rule {
        rule_type: RuleType::baseExpr__FLOAT,
        token: Token::baseExpr,
        steps: &[Token::FLOAT],
    },
    Rule {
        rule_type: RuleType::baseExpr__stringLiteral,
        token: Token::baseExpr,
        steps: &[Token::stringLiteral],
    },
    Rule {
        rule_type: RuleType::baseExpr__TRUE,
        token: Token::baseExpr,
        steps: &[Token::TRUE],
    },
    Rule {
        rule_type: RuleType::baseExpr__FALSE,
        token: Token::baseExpr,
        steps: &[Token::FALSE],
    },
    Rule {
        rule_type: RuleType::baseExpr__NULL,
        token: Token::baseExpr,
        steps: &[Token::NULL],
    },
    Rule {
        rule_type: RuleType::baseExpr__fcnCallExpr,
        token: Token::baseExpr,
        steps: &[Token::fcnCallExpr],
    },
    Rule {
        rule_type: RuleType::baseExpr__IDENTIFIER,
        token: Token::baseExpr,
        steps: &[Token::IDENTIFIER],
    },
    Rule {
        rule_type: RuleType::baseDotExpr__DOT,
        token: Token::baseDotExpr,
        steps: &[Token::DOT],
    },
    Rule {
        rule_type: RuleType::baseDotAccess__DOT_IDENTIFIER,
        token: Token::baseDotAccess,
        steps: &[Token::DOT,Token::IDENTIFIER],
    },
    Rule {
        rule_type: RuleType::listAccessIdx__expr,
        token: Token::listAccessIdx,
        steps: &[Token::expr],
    },
    Rule {
        rule_type: RuleType::listAccessIdx__FORWARD_SLASH_expr,
        token: Token::listAccessIdx,
        steps: &[Token::FORWARD_SLASH,Token::expr],
    },
    Rule {
        rule_type: RuleType::listAccessExpr__listAccessIdx,
        token: Token::listAccessExpr,
        steps: &[Token::listAccessIdx],
    },
    Rule {
        rule_type: RuleType::listAccessExpr__listAccessIdx_COLON,
        token: Token::listAccessExpr,
        steps: &[Token::listAccessIdx,Token::COLON],
    },
    Rule {
        rule_type: RuleType::listAccessExpr__COLON_listAccessIdx,
        token: Token::listAccessExpr,
        steps: &[Token::COLON,Token::listAccessIdx],
    },
    Rule {
        rule_type: RuleType::listAccessExpr__listAccessIdx_COLON_listAccessIdx,
        token: Token::listAccessExpr,
        steps: &[Token::listAccessIdx,Token::COLON,Token::listAccessIdx],
    },
    Rule {
        rule_type: RuleType::listAccessExpr__COLON,
        token: Token::listAccessExpr,
        steps: &[Token::COLON],
    },
    Rule {
        rule_type: RuleType::mapExpr__LEFT_BRACE_RIGHT_BRACE,
        token: Token::mapExpr,
        steps: &[Token::LEFT_BRACE,Token::RIGHT_BRACE],
    },
    Rule {
        rule_type: RuleType::mapExpr__LEFT_BRACE_mapContents_RIGHT_BRACE,
        token: Token::mapExpr,
        steps: &[Token::LEFT_BRACE,Token::mapContents,Token::RIGHT_BRACE],
    },
    Rule {
        rule_type: RuleType::mapExpr__LEFT_BRACE_mapContents_COMMA_RIGHT_BRACE,
        token: Token::mapExpr,
        steps: &[Token::LEFT_BRACE,Token::mapContents,Token::COMMA,Token::RIGHT_BRACE],
    },
    Rule {
        rule_type: RuleType::mapContents__mapContentsElem,
        token: Token::mapContents,
        steps: &[Token::mapContentsElem],
    },
    Rule {
        rule_type: RuleType::mapContents__mapContents_COMMA_mapContentsElem,
        token: Token::mapContents,
        steps: &[Token::mapContents,Token::COMMA,Token::mapContentsElem],
    },
    Rule {
        rule_type: RuleType::mapContentsElem__IDENTIFIER_COLON_expr,
        token: Token::mapContentsElem,
        steps: &[Token::IDENTIFIER,Token::COLON,Token::expr],
    },
    Rule {
        rule_type: RuleType::listExpr__LEFT_BRACKET_RIGHT_BRACKET,
        token: Token::listExpr,
        steps: &[Token::LEFT_BRACKET,Token::RIGHT_BRACKET],
    },
    Rule {
        rule_type: RuleType::listExpr__LEFT_BRACKET_listExprContents_RIGHT_BRACKET,
        token: Token::listExpr,
        steps: &[Token::LEFT_BRACKET,Token::listExprContents,Token::RIGHT_BRACKET],
    },
    Rule {
        rule_type: RuleType::listExpr__LEFT_BRACKET_listExprContents_COMMA_RIGHT_BRACKET,
        token: Token::listExpr,
        steps: &[Token::LEFT_BRACKET,Token::listExprContents,Token::COMMA,Token::RIGHT_BRACKET],
    },
    Rule {
        rule_type: RuleType::listExprContents__listElem,
        token: Token::listExprContents,
        steps: &[Token::listElem],
    },
    Rule {
        rule_type: RuleType::listExprContents__listExprContents_COMMA_listElem,
        token: Token::listExprContents,
        steps: &[Token::listExprContents,Token::COMMA,Token::listElem],
    },
    Rule {
        rule_type: RuleType::listElem__expr,
        token: Token::listElem,
        steps: &[Token::expr],
    },
    Rule {
        rule_type: RuleType::stringLiteral__STRING_SINGLE_QUOTE,
        token: Token::stringLiteral,
        steps: &[Token::STRING_SINGLE_QUOTE],
    },
    Rule {
        rule_type: RuleType::stringLiteral__F_STRING_SINGLE_QUOTE_LEFT_innerFormatStringSingleQuote_F_STRING_SINGLE_QUOTE_RIGHT,
        token: Token::stringLiteral,
        steps: &[Token::F_STRING_SINGLE_QUOTE_LEFT,Token::innerFormatStringSingleQuote,Token::F_STRING_SINGLE_QUOTE_RIGHT],
    },
    Rule {
        rule_type: RuleType::innerFormatStringSingleQuote__expr,
        token: Token::innerFormatStringSingleQuote,
        steps: &[Token::expr],
    },
    Rule {
        rule_type: RuleType::innerFormatStringSingleQuote__innerFormatStringSingleQuote_F_STRING_SINGLE_QUOTE_MIDDLE_expr,
        token: Token::innerFormatStringSingleQuote,
        steps: &[Token::innerFormatStringSingleQuote,Token::F_STRING_SINGLE_QUOTE_MIDDLE,Token::expr],
    },
    Rule {
        rule_type: RuleType::stringLiteral__STRING_DOUBLE_QUOTE,
        token: Token::stringLiteral,
        steps: &[Token::STRING_DOUBLE_QUOTE],
    },
    Rule {
        rule_type: RuleType::stringLiteral__F_STRING_DOUBLE_QUOTE_LEFT_innerFormatStringDoubleQuote_F_STRING_DOUBLE_QUOTE_RIGHT,
        token: Token::stringLiteral,
        steps: &[Token::F_STRING_DOUBLE_QUOTE_LEFT,Token::innerFormatStringDoubleQuote,Token::F_STRING_DOUBLE_QUOTE_RIGHT],
    },
    Rule {
        rule_type: RuleType::innerFormatStringDoubleQuote__expr,
        token: Token::innerFormatStringDoubleQuote,
        steps: &[Token::expr],
    },
    Rule {
        rule_type: RuleType::innerFormatStringDoubleQuote__innerFormatStringDoubleQuote_F_STRING_DOUBLE_QUOTE_MIDDLE_expr,
        token: Token::innerFormatStringDoubleQuote,
        steps: &[Token::innerFormatStringDoubleQuote,Token::F_STRING_DOUBLE_QUOTE_MIDDLE,Token::expr],
    },
    Rule {
        rule_type: RuleType::fcnCallExpr__IDENTIFIER_LEFT_PAREN_RIGHT_PAREN,
        token: Token::fcnCallExpr,
        steps: &[Token::IDENTIFIER,Token::LEFT_PAREN,Token::RIGHT_PAREN],
    },
    Rule {
        rule_type: RuleType::fcnCallExpr__IDENTIFIER_LEFT_PAREN_fcnCallArgs_RIGHT_PAREN,
        token: Token::fcnCallExpr,
        steps: &[Token::IDENTIFIER,Token::LEFT_PAREN,Token::fcnCallArgs,Token::RIGHT_PAREN],
    },
    Rule {
        rule_type: RuleType::fcnCallExpr__IDENTIFIER_LEFT_PAREN_fcnCallArgs_COMMA_RIGHT_PAREN,
        token: Token::fcnCallExpr,
        steps: &[Token::IDENTIFIER,Token::LEFT_PAREN,Token::fcnCallArgs,Token::COMMA,Token::RIGHT_PAREN],
    },
    Rule {
        rule_type: RuleType::fcnCallArgs__expr,
        token: Token::fcnCallArgs,
        steps: &[Token::expr],
    },
    Rule {
        rule_type: RuleType::fcnCallArgs__fcnCallArgs_COMMA_expr,
        token: Token::fcnCallArgs,
        steps: &[Token::fcnCallArgs,Token::COMMA,Token::expr],
    },
];
