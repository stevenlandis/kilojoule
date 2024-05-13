use std::rc::Rc;

#[derive(Debug)]
pub enum AstNode<'a> {
    None,
    StringLiteral(&'a str),
    Echo,
    Access(Rc<AstNode<'a>>),
    Pipe(Rc<AstNode<'a>>, Rc<AstNode<'a>>),

    Int(u64),
    Plus,
    Asterisk,
    Add(Rc<AstNode<'a>>, Rc<AstNode<'a>>),
    Mul(Rc<AstNode<'a>>, Rc<AstNode<'a>>),
    End,
}
