#[derive(Debug)]
pub enum AstNode<'a> {
    Null,
    SubString(&'a str),
    Integer(u64),
    Bool(bool),
    Pipe(AstNodePtr, AstNodePtr),
    Dot,
    Access(AstNodePtr),
    Equals(AstNodePtr, AstNodePtr),
    NotEquals(AstNodePtr, AstNodePtr),
    LessThan(AstNodePtr, AstNodePtr),
    LessThanOrEqual(AstNodePtr, AstNodePtr),
    GreaterThan(AstNodePtr, AstNodePtr),
    GreaterThanOrEqual(AstNodePtr, AstNodePtr),
    Or(AstNodePtr, AstNodePtr),
    And(AstNodePtr, AstNodePtr),
    Add(AstNodePtr, AstNodePtr),
    Subtract(AstNodePtr, AstNodePtr),
    Multiply(AstNodePtr, AstNodePtr),
    Divide(AstNodePtr, AstNodePtr),
    FcnCall {
        name: AstNodePtr,
        args: Option<AstNodePtr>,
    },
    ListNode(AstNodePtr, AstNodePtr),
    MapKeyValPair {
        key: AstNodePtr,
        val: AstNodePtr,
    },
    MapLiteral(Option<AstNodePtr>),
    ListLiteral(Option<AstNodePtr>),
    FormatString(Option<AstNodePtr>),
    ReverseIdx(AstNodePtr),
    SliceAccess(Option<AstNodePtr>, Option<AstNodePtr>),
}

pub type AstNodePtr = usize;

pub struct AstNodePool<'a> {
    pub vals: Vec<AstNode<'a>>,
}

impl<'a> AstNodePool<'a> {
    pub fn new() -> Self {
        AstNodePool { vals: Vec::new() }
    }

    pub fn get(&self, ptr: AstNodePtr) -> &AstNode {
        &self.vals[ptr]
    }

    pub fn new_node(&mut self, val: AstNode<'a>) -> AstNodePtr {
        let ptr = self.vals.len() as AstNodePtr;
        self.vals.push(val);
        ptr
    }

    pub fn new_null(&mut self) -> AstNodePtr {
        self.new_node(AstNode::Null)
    }

    pub fn new_identifier(&mut self, text: &'a str) -> AstNodePtr {
        self.new_node(AstNode::SubString(text))
    }

    pub fn new_integer(&mut self, val: u64) -> AstNodePtr {
        self.new_node(AstNode::Integer(val))
    }

    pub fn new_bool(&mut self, val: bool) -> AstNodePtr {
        self.new_node(AstNode::Bool(val))
    }

    pub fn new_pipe(&mut self, left: AstNodePtr, right: AstNodePtr) -> AstNodePtr {
        self.new_node(AstNode::Pipe(left, right))
    }

    pub fn new_dot(&mut self) -> AstNodePtr {
        self.new_node(AstNode::Dot)
    }

    pub fn new_access(&mut self, accessor: AstNodePtr) -> AstNodePtr {
        self.new_node(AstNode::Access(accessor))
    }

    pub fn new_or(&mut self, left: AstNodePtr, right: AstNodePtr) -> AstNodePtr {
        self.new_node(AstNode::Or(left, right))
    }

    pub fn new_and(&mut self, left: AstNodePtr, right: AstNodePtr) -> AstNodePtr {
        self.new_node(AstNode::And(left, right))
    }

    pub fn new_add(&mut self, left: AstNodePtr, right: AstNodePtr) -> AstNodePtr {
        self.new_node(AstNode::Add(left, right))
    }

    pub fn new_subtract(&mut self, left: AstNodePtr, right: AstNodePtr) -> AstNodePtr {
        self.new_node(AstNode::Subtract(left, right))
    }

    pub fn new_list_node(&mut self, left: AstNodePtr, right: AstNodePtr) -> AstNodePtr {
        self.new_node(AstNode::ListNode(left, right))
    }

    pub fn new_fcn_call(&mut self, name: AstNodePtr, args: Option<AstNodePtr>) -> AstNodePtr {
        self.new_node(AstNode::FcnCall { name, args })
    }

    pub fn new_map_kv_pair(&mut self, key: AstNodePtr, val: AstNodePtr) -> AstNodePtr {
        self.new_node(AstNode::MapKeyValPair { key, val })
    }

    pub fn new_map_literal(&mut self, contents: Option<AstNodePtr>) -> AstNodePtr {
        self.new_node(AstNode::MapLiteral(contents))
    }

    pub fn new_list_literal(&mut self, contents: Option<AstNodePtr>) -> AstNodePtr {
        self.new_node(AstNode::ListLiteral(contents))
    }

    pub fn new_format_string(&mut self, contents: Option<AstNodePtr>) -> AstNodePtr {
        self.new_node(AstNode::FormatString(contents))
    }
}
