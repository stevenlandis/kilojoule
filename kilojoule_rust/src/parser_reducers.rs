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
        Token::F_STRING_SINGLE_QUOTE_LEFT => AstNode::StringLiteral(escape_string_literal(text)),
        Token::F_STRING_SINGLE_QUOTE_MIDDLE => AstNode::StringLiteral(escape_string_literal(text)),
        Token::F_STRING_SINGLE_QUOTE_RIGHT => AstNode::StringLiteral(escape_string_literal(text)),
        Token::STRING_DOUBLE_QUOTE => AstNode::StringLiteral(escape_string_literal(text)),
        Token::TRUE => AstNode::Bool(true),
        Token::FALSE => AstNode::Bool(false),
        Token::NULL => AstNode::Null,
        Token::PLUS => AstNode::PLUS,
        Token::MINUS => AstNode::MINUS,
        Token::DOUBLE_EQUALS => AstNode::DOUBLE_EQUALS,
        Token::NOT_EQUALS => AstNode::NOT_EQUAL,
        Token::LESS_THAN => AstNode::LESS_THAN,
        Token::LESS_THAN_OR_EQUAL => AstNode::LESS_THAN_OR_EQUAL,
        Token::GREATER_THAN => AstNode::GREATER_THAN,
        Token::GREATER_THAN_OR_EQUAL => AstNode::GREATER_THAN_OR_EQUAL,
        Token::ASTERISK => AstNode::ASTERISK,
        Token::FORWARD_SLASH => AstNode::FORWARD_SLASH,
        _ => AstNode::Null,
    };
}

pub fn get_reduced_rule(rule: RuleType, elems: Vec<Rc<AstNode>>) -> Rc<AstNode> {
    return match rule {
        RuleType::opPipeExpr__opPipeExpr_PIPE_opOrExpr => {
            Rc::new(AstNode::Pipe(elems[0].clone(), elems[2].clone()))
        }
        RuleType::baseExpr__LEFT_PAREN_expr_RIGHT_PAREN => elems[1].clone(),
        RuleType::baseDotExpr__DOT => Rc::new(AstNode::Echo),

        // Access
        RuleType::baseDotAccess__DOT_IDENTIFIER => Rc::new(AstNode::Access(elems[1].clone())),
        RuleType::opAccessExpr__opAccessExpr_DOT_IDENTIFIER => Rc::new(AstNode::Pipe(elems[0].clone(), Rc::new(AstNode::Access(elems[2].clone())))),
        RuleType::opAccessExpr__opAccessExpr_LEFT_BRACKET_listAccessExpr_RIGHT_BRACKET => Rc::new(AstNode::Pipe(elems[0].clone(), Rc::new(AstNode::Access(elems[2].clone())))),

        // Assign
        RuleType::assignExpr__LET_IDENTIFIER_EQUAL_opOrExpr_PIPE_assignExpr => Rc::new(AstNode::Assign(match &*elems[1] {
            AstNode::StringLiteral(var_name) => var_name.clone(),
            _ => {panic!("unreachable");}
        }, elems[3].clone(), elems[5].clone())),
        RuleType::assignExpr__opPipeExpr_PIPE_LET_IDENTIFIER_EQUAL_opOrExpr_PIPE_assignExpr => Rc::new(AstNode::Pipe(elems[0].clone(), Rc::new(AstNode::Assign(match &*elems[3] {
            AstNode::StringLiteral(var_name) => var_name.clone(),
            _ => {panic!("unreachable");}
        }, elems[5].clone(), elems[7].clone())))),
        RuleType::baseExpr__IDENTIFIER => Rc::new(AstNode::VarAccess(match &*elems[0] {AstNode::StringLiteral(var_name) => var_name.clone(), _ => {panic!("unreachable");}})),

        // List Access
        RuleType::listAccessIdx__FORWARD_SLASH_expr => Rc::new(AstNode::ReverseIdx(elems[1].clone())),
        RuleType::listAccessExpr__COLON => Rc::new(AstNode::SliceAccess(None, None)),
        RuleType::listAccessExpr__COLON_listAccessIdx => Rc::new(AstNode::SliceAccess(None, Some(elems[1].clone()))),
        RuleType::listAccessExpr__listAccessIdx_COLON => Rc::new(AstNode::SliceAccess(Some(elems[0].clone()), None)),
        RuleType::listAccessExpr__listAccessIdx_COLON_listAccessIdx => Rc::new(AstNode::SliceAccess(Some(elems[0].clone()), Some(elems[2].clone()))),


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

        // Format String
        RuleType::stringLiteral__F_STRING_SINGLE_QUOTE_LEFT_innerFormatStringSingleQuote_F_STRING_SINGLE_QUOTE_RIGHT=>{
            Rc::new(AstNode::FormatStringNode(vec![elems[0].clone(), elems[1].clone(), elems[2].clone()]))
        }
        RuleType::innerFormatStringSingleQuote__innerFormatStringSingleQuote_F_STRING_SINGLE_QUOTE_MIDDLE_expr=>{
            Rc::new(AstNode::FormatStringNode(vec![elems[0].clone(), elems[1].clone(), elems[2].clone()]))
        },

        // function callls
        RuleType::fcnCallExpr__IDENTIFIER_LEFT_PAREN_RIGHT_PAREN => {
            Rc::new(AstNode::FcnCall(elems[0].clone(), None))
        }
        RuleType::fcnCallExpr__IDENTIFIER_LEFT_PAREN_fcnCallArgs_RIGHT_PAREN => {Rc::new(AstNode::FcnCall(elems[0].clone(), Some(elems[2].clone())))}
        RuleType::fcnCallArgs__fcnCallArgs_COMMA_expr => {
            Rc::new(AstNode::FcnCallArgNode(elems[0].clone(), elems[2].clone()))
        }

        // Add and Subtract
        RuleType::opAddExpr__opAddExpr_opAddOperator_opMulExpr => {
            Rc::new(match *elems[1] {
                AstNode::PLUS => AstNode::Add(elems[0].clone(), elems[2].clone()),
                AstNode::MINUS => AstNode::Subtract(elems[0].clone(), elems[2].clone()),
                _ => panic!("invalid add operator")
            })
        }

        // equality operators
        RuleType::opEqualityExpr__opAddExpr_equalityOperator_opAddExpr => {
            Rc::new(match *elems[1] {
                AstNode::DOUBLE_EQUALS => AstNode::Equals(elems[0].clone(), elems[2].clone()),
                AstNode::NOT_EQUAL => AstNode::NotEqual(elems[0].clone(), elems[2].clone()),
                AstNode::LESS_THAN => AstNode::LessThan(elems[0].clone(), elems[2].clone()),
                AstNode::LESS_THAN_OR_EQUAL => AstNode::LessThanOrEqual(elems[0].clone(), elems[2].clone()),
                AstNode::GREATER_THAN => AstNode::GreaterThan(elems[0].clone(), elems[2].clone()),
                AstNode::GREATER_THAN_OR_EQUAL => AstNode::GreaterThanOrEqual(elems[0].clone(), elems[2].clone()),
                _ => panic!("invalid equality operator")
            })
        }

        // or
        RuleType::opOrExpr__opOrExpr_OR_opAndExpr => {
            Rc::new(AstNode::Or(elems[0].clone(), elems[2].clone()))
        }

        // and
        RuleType::opAndExpr__opAndExpr_AND_opEqualityExpr => {
            Rc::new(AstNode::And(elems[0].clone(), elems[2].clone()))
        }

        // Multiply and Divide
        RuleType::opMulExpr__opMulExpr_opMulOperator_opAccessExpr => {
            Rc::new(match *elems[1] {
                AstNode::ASTERISK => AstNode::Multiply(elems[0].clone(), elems[2].clone()),
                AstNode::FORWARD_SLASH => AstNode::Divide(elems[0].clone(), elems[2].clone()),
                _ => panic!("invalid add operator")
            })
        }

        // Default
        _ => elems[0].clone(),
    };
}

fn escape_string_literal(input: &str) -> String {
    let bytes = input.as_bytes();
    assert!(bytes.len() >= 2);
    let trimmed = &bytes[1..bytes.len() - 1];

    let mut escaped_bytes = Vec::<u8>::new();
    let mut idx = 0;
    while idx < trimmed.len() {
        if trimmed[idx] == '\\' as u8 {
            if idx + 1 >= trimmed.len() {
                escaped_bytes.push(trimmed[idx]);
            } else {
                let next_char = trimmed[idx + 1] as char;
                match next_char {
                    'n' => escaped_bytes.push('\n' as u8),
                    'r' => escaped_bytes.push('\r' as u8),
                    't' => escaped_bytes.push('\t' as u8),
                    '{' => escaped_bytes.push('{' as u8),
                    '}' => escaped_bytes.push('}' as u8),
                    '"' => escaped_bytes.push('"' as u8),
                    '\'' => escaped_bytes.push('\'' as u8),
                    _ => escaped_bytes.push(next_char as u8),
                };
            }
            idx += 2;
        } else {
            escaped_bytes.push(trimmed[idx]);
            idx += 1;
        }
    }

    let input = std::str::from_utf8(escaped_bytes.as_slice()).unwrap();
    return input.to_string();
}
