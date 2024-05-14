use super::ast_node::AstNode;
use super::rule_type::RuleType;
use super::token::Token;
use std::rc::Rc;

pub fn get_reduced_token<'a>(token: Token, text: &'a str) -> AstNode {
    return match token {
        Token::IDENTIFIER => AstNode::StringLiteral(text.to_string()),
        Token::INTEGER => AstNode::F64Literal(text.parse::<f64>().unwrap()),
        Token::FLOAT => AstNode::F64Literal(text.parse::<f64>().unwrap()),
        Token::STRING_SINGLE_QUOTE => AstNode::StringLiteral(escape_string_literal(text)),
        Token::STRING_DOUBLE_QUOTE => AstNode::StringLiteral(escape_string_literal(text)),
        _ => AstNode::None,
    };
}

pub fn get_reduced_rule(rule: RuleType, elems: Vec<Rc<AstNode>>) -> Rc<AstNode> {
    return match rule {
        RuleType::opPipeExpr__opPipeExpr_PIPE_baseExpr => {
            Rc::new(AstNode::Pipe(elems[0].clone(), elems[2].clone()))
        }
        RuleType::baseExpr__LEFT_PAREN_expr_RIGHT_PAREN => elems[1].clone(),
        RuleType::baseDotExpr__DOT => Rc::new(AstNode::Echo),
        RuleType::baseDotAccess__DOT_IDENTIFIER => Rc::new(AstNode::Access(elems[1].clone())),
        RuleType::baseDotBracketAccess__DOT_LEFT_BRACKET_expr_RIGHT_BRACKET => {
            Rc::new(AstNode::Access(elems[2].clone()))
        }

        // Map
        RuleType::mapExpr__LEFT_BRACE_RIGHT_BRACE => Rc::new(AstNode::MapLiteral(None)),
        RuleType::mapExpr__LEFT_BRACE_mapContents_RIGHT_BRACE => {
            Rc::new(AstNode::MapLiteral(Some(elems[1].clone())))
        }
        RuleType::mapExpr__LEFT_BRACE_mapContents_COMMA_RIGHT_BRACE => {
            Rc::new(AstNode::MapLiteral(Some(elems[1].clone())))
        }
        RuleType::mapContents__mapContents_COMMA_mapContentsElem => {
            Rc::new(AstNode::MapElemListNode(elems[0].clone(), elems[2].clone()))
        }
        RuleType::mapContentsElem__IDENTIFIER_COLON_expr => {
            Rc::new(AstNode::MapKeyValPair(elems[0].clone(), elems[2].clone()))
        }

        // List
        RuleType::listExpr__LEFT_BRACKET_RIGHT_BRACKET => Rc::new(AstNode::ListLiteral(None)),
        RuleType::listExpr__LEFT_BRACKET_listExprContents_RIGHT_BRACKET => {
            Rc::new(AstNode::ListLiteral(Some(elems[1].clone())))
        }
        RuleType::listExpr__LEFT_BRACKET_listExprContents_COMMA_RIGHT_BRACKET => {
            Rc::new(AstNode::ListLiteral(Some(elems[1].clone())))
        }
        RuleType::listExprContents__listExprContents_COMMA_listElem => Rc::new(
            AstNode::ListElemListNode(elems[0].clone(), elems[2].clone()),
        ),
        _ => elems[0].clone(),
    };
}

fn escape_string_literal(input: &str) -> String {
    let bytes = input.as_bytes();
    assert!(bytes.len() >= 2);
    let trimmed = &bytes[1..bytes.len() - 1];
    let input = std::str::from_utf8(trimmed).unwrap();
    return input.to_string();
}
