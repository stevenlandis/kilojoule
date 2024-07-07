use std::{fmt::Pointer, rc::Rc};

#[derive(Clone)]
pub struct AstNode {
    inner_val: Rc<InnerVal>,
}

impl core::fmt::Debug for AstNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner_val.fmt(f)
    }
}

struct InnerVal {
    node_type: AstNodeType,
}

#[derive(Debug)]
pub enum AstNodeType {
    Null,
    SubString(String),
    Identifier(String),
    Integer(u64),
    Bool(bool),
    Pipe(AstNode, AstNode),
    Dot,
    AccessChain(AstNode, AstNode),
    Equals(AstNode, AstNode),
    NotEquals(AstNode, AstNode),
    LessThan(AstNode, AstNode),
    LessThanOrEqual(AstNode, AstNode),
    GreaterThan(AstNode, AstNode),
    GreaterThanOrEqual(AstNode, AstNode),
    Or(AstNode, AstNode),
    And(AstNode, AstNode),
    Not(AstNode),
    Add(AstNode, AstNode),
    Subtract(AstNode, AstNode),
    Multiply(AstNode, AstNode),
    Divide(AstNode, AstNode),
    Negative(AstNode),
    FcnCall {
        name: AstNode,
        args: Option<AstNode>,
    },
    ListNode(AstNode, AstNode),
    MapKeyValPair {
        key: AstNode,
        val: AstNode,
    },
    MapLiteral(Option<AstNode>),
    ListLiteral(Option<AstNode>),
    FormatString(Option<AstNode>),
    ReverseIdx(AstNode),
    SliceAccess(Option<AstNode>, Option<AstNode>),
    Coalesce(AstNode, AstNode),
    Spread(AstNode),
    KeywordArgument(AstNode, AstNode),
    LetStmt {
        identifier: AstNode,
        expr: AstNode,
    },
}

impl AstNode {
    pub fn new(node_type: AstNodeType) -> AstNode {
        AstNode {
            inner_val: Rc::new(InnerVal { node_type }),
        }
    }

    pub fn get_type(&self) -> &AstNodeType {
        &self.inner_val.node_type
    }
}
