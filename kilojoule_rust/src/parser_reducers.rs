use super::ast_node::AstNode;
use super::rule_type::RuleType;
use super::token::Token;
use std::rc::Rc;

pub fn get_reduced_token<'a>(token: Token, text: &'a str) -> AstNode<'a> {
    return match token {
        Token::IDENTIFIER => AstNode::StringLiteral(text),
        Token::INTEGER => AstNode::F64Literal(text.parse::<f64>().unwrap()),
        Token::FLOAT => AstNode::F64Literal(text.parse::<f64>().unwrap()),
        _ => AstNode::None,
    };
}

pub fn get_reduced_rule(rule: RuleType, elems: Vec<Rc<AstNode>>) -> Rc<AstNode> {
    return match rule {
        RuleType::opPipeExpr__opPipeExpr_PIPE_opBaseExpr => {
            Rc::new(AstNode::Pipe(elems[0].clone(), elems[2].clone()))
        }
        RuleType::opBaseExpr__LEFT_PAREN_expr_RIGHT_PAREN => elems[1].clone(),
        RuleType::baseDotExpr__DOT => Rc::new(AstNode::Echo),
        RuleType::baseDotAccess__DOT_IDENTIFIER => Rc::new(AstNode::Access(elems[1].clone())),
        RuleType::dictExpr__LEFT_BRACE_RIGHT_BRACE => Rc::new(AstNode::MapLiteral(None)),
        RuleType::dictExpr__LEFT_BRACE_dictContents_RIGHT_BRACE => {
            Rc::new(AstNode::MapLiteral(Some(elems[1].clone())))
        }
        RuleType::dictExpr__LEFT_BRACE_dictContents_COMMA_RIGHT_BRACE => {
            Rc::new(AstNode::MapLiteral(Some(elems[1].clone())))
        }
        RuleType::dictContents__dictContents_COMMA_dictContentsElem => {
            Rc::new(AstNode::MapElemListNode(elems[0].clone(), elems[2].clone()))
        }
        RuleType::dictContentsElem__IDENTIFIER_COLON_expr => {
            Rc::new(AstNode::MapKeyValPair(elems[0].clone(), elems[2].clone()))
        }
        _ => elems[0].clone(),
    };
}
