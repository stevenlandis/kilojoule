use std::rc::Rc;

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
    Plus,
    Minus,
}
