use std::rc::Rc;

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum AstNode {
    Null,
    StringLiteral(String),
    F64Literal(f64),
    Bool(bool),
    Echo,
    Access(Rc<AstNode>),
    Pipe(Rc<AstNode>, Rc<AstNode>),
    MapLiteral(Option<Rc<AstNode>>),
    MapElemListNode(Rc<AstNode>, Rc<AstNode>),
    MapKeyValPair(Rc<AstNode>, Rc<AstNode>),
    ListLiteral(Option<Rc<AstNode>>),
    ListElemListNode(Rc<AstNode>, Rc<AstNode>),
    FormatStringNode(Vec<Rc<AstNode>>),
    FcnCall(Rc<AstNode>, Option<Rc<AstNode>>),
    FcnCallArgNode(Rc<AstNode>, Rc<AstNode>),
    Add(Rc<AstNode>, Rc<AstNode>),
    Subtract(Rc<AstNode>, Rc<AstNode>),
    PLUS,
    MINUS,
    DOUBLE_EQUALS,
    NOT_EQUAL,
    LESS_THAN,
    LESS_THAN_OR_EQUAL,
    GREATER_THAN,
    GREATER_THAN_OR_EQUAL,
    Equals(Rc<AstNode>, Rc<AstNode>),
    NotEqual(Rc<AstNode>, Rc<AstNode>),
    LessThan(Rc<AstNode>, Rc<AstNode>),
    LessThanOrEqual(Rc<AstNode>, Rc<AstNode>),
    GreaterThan(Rc<AstNode>, Rc<AstNode>),
    GreaterThanOrEqual(Rc<AstNode>, Rc<AstNode>),
    Or(Rc<AstNode>, Rc<AstNode>),
    And(Rc<AstNode>, Rc<AstNode>),
    ASTERISK,
    FORWARD_SLASH,
    Multiply(Rc<AstNode>, Rc<AstNode>),
    Divide(Rc<AstNode>, Rc<AstNode>),
}
