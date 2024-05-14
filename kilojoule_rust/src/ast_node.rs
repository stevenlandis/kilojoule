use std::rc::Rc;

#[derive(Debug)]
pub enum AstNode<'a> {
    None,
    StringLiteral(&'a str),
    F64Literal(f64),
    Echo,
    Access(Rc<AstNode<'a>>),
    Pipe(Rc<AstNode<'a>>, Rc<AstNode<'a>>),
    MapLiteral(Option<Rc<AstNode<'a>>>),
    MapElemListNode(Rc<AstNode<'a>>, Rc<AstNode<'a>>),
    MapKeyValPair(Rc<AstNode<'a>>, Rc<AstNode<'a>>),
    ListLiteral(Option<Rc<AstNode<'a>>>),
    ListElemListNode(Rc<AstNode<'a>>, Rc<AstNode<'a>>),

    Int(u64),
    Plus,
    Asterisk,
    Add(Rc<AstNode<'a>>, Rc<AstNode<'a>>),
    Mul(Rc<AstNode<'a>>, Rc<AstNode<'a>>),
    End,
}
