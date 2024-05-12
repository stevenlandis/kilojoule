use super::ast_node::AstNode;
use super::rule_type::RuleType;
use super::token::Token;
use std::rc::Rc;

pub fn get_reduced_token<'a>(token: Token, text: &'a str) -> AstNode<'a> {
    return match token {
        Token::DOT => AstNode::None,
        Token::IDENTIFIER => AstNode::StringLiteral(text),
        Token::END => AstNode::None,
        _ => {
            panic!("Unimplemented token reduce {:?}", token);
        }
    };
}

pub fn get_reduced_rule(rule: RuleType, elems: Vec<Rc<AstNode>>) -> Rc<AstNode> {
    return match rule {
        RuleType::main__expr_END => elems[0].clone(),
        RuleType::expr__opBaseExpr => elems[0].clone(),
        RuleType::opBaseExpr__baseDotExpr => elems[0].clone(),
        RuleType::opBaseExpr__baseDotAccess => elems[0].clone(),
        RuleType::baseDotExpr__DOT => Rc::new(AstNode::Echo),
        RuleType::baseDotAccess__DOT_IDENTIFIER => Rc::new(AstNode::Access(elems[1].clone())),
    };
}
