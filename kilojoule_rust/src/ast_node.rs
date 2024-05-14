use std::rc::Rc;

#[derive(Debug)]
pub enum AstNode {
    None,
    StringLiteral(String),
    F64Literal(f64),
    Echo,
    Access(Rc<AstNode>),
    Pipe(Rc<AstNode>, Rc<AstNode>),
    MapLiteral(Option<Rc<AstNode>>),
    MapElemListNode(Rc<AstNode>, Rc<AstNode>),
    MapKeyValPair(Rc<AstNode>, Rc<AstNode>),
    ListLiteral(Option<Rc<AstNode>>),
    ListElemListNode(Rc<AstNode>, Rc<AstNode>),

    Int(u64),
    Plus,
    Asterisk,
    Add(Rc<AstNode>, Rc<AstNode>),
    Mul(Rc<AstNode>, Rc<AstNode>),
    End,
}
