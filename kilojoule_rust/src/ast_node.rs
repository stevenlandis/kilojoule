use std::rc::Rc;

#[derive(Debug)]
pub enum AstNode {
    Int(u64),
    Plus,
    Asterisk,
    Add(Rc<AstNode>, Rc<AstNode>),
    Mul(Rc<AstNode>, Rc<AstNode>),
    End,
}
