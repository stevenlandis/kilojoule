use super::rule_type::RuleType;
use super::token::Token;

pub struct Rule<'a> {
    pub rule_type: RuleType,
    pub token: Token,
    pub steps: &'a [Token],
}

pub static RULES: &[Rule] = &[
    Rule {
        rule_type: RuleType::Main__Expr_END,
        token: Token::Main,
        steps: &[Token::Expr, Token::END],
    },
    Rule {
        rule_type: RuleType::Expr__AddExpr,
        token: Token::Expr,
        steps: &[Token::AddExpr],
    },
    Rule {
        rule_type: RuleType::AddExpr__MulExpr,
        token: Token::AddExpr,
        steps: &[Token::MulExpr],
    },
    Rule {
        rule_type: RuleType::AddExpr__AddExpr_PLUS_MulExpr,
        token: Token::AddExpr,
        steps: &[Token::AddExpr, Token::PLUS, Token::MulExpr],
    },
    Rule {
        rule_type: RuleType::MulExpr__INTEGER,
        token: Token::MulExpr,
        steps: &[Token::INTEGER],
    },
    Rule {
        rule_type: RuleType::MulExpr__MulExpr_ASTERISK_INTEGER,
        token: Token::MulExpr,
        steps: &[Token::MulExpr, Token::ASTERISK, Token::INTEGER],
    },
];
