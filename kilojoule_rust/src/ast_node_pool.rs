#[derive(Debug)]
pub enum AstNode<'a> {
    Null,
    SubString(&'a str),
    Integer(u64),
    Bool(bool),
    Pipe(AstNodePtr, AstNodePtr),
    Dot,
    Access(AstNodePtr),
    Add(AstNodePtr, AstNodePtr),
    FcnCall(Option<AstNodePtr>),
    ListNode(AstNodePtr, AstNodePtr),
    MapKeyValPair { key: AstNodePtr, val: AstNodePtr },
    MapLiteral(Option<AstNodePtr>),
    ListLiteral(Option<AstNodePtr>),
    FormatString(Option<AstNodePtr>),
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

    pub fn new_null(&mut self) -> AstNodePtr {
        let ptr = self.vals.len() as AstNodePtr;
        self.vals.push(AstNode::Null);
        ptr
    }

    pub fn new_identifier(&mut self, text: &'a str) -> AstNodePtr {
        let ptr = self.vals.len() as AstNodePtr;
        self.vals.push(AstNode::SubString(text));
        ptr
    }

    pub fn new_integer(&mut self, val: u64) -> AstNodePtr {
        let ptr = self.vals.len() as AstNodePtr;
        self.vals.push(AstNode::Integer(val));
        ptr
    }

    pub fn new_bool(&mut self, val: bool) -> AstNodePtr {
        let ptr = self.vals.len() as AstNodePtr;
        self.vals.push(AstNode::Bool(val));
        ptr
    }

    pub fn new_pipe(&mut self, left: AstNodePtr, right: AstNodePtr) -> AstNodePtr {
        let ptr = self.vals.len();
        self.vals.push(AstNode::Pipe(left, right));
        ptr
    }

    pub fn new_dot(&mut self) -> AstNodePtr {
        let ptr = self.vals.len();
        self.vals.push(AstNode::Dot);
        ptr
    }

    pub fn new_access(&mut self, accessor: AstNodePtr) -> AstNodePtr {
        let ptr = self.vals.len();
        self.vals.push(AstNode::Access(accessor));
        ptr
    }

    pub fn new_add(&mut self, left: AstNodePtr, right: AstNodePtr) -> AstNodePtr {
        let ptr = self.vals.len();
        self.vals.push(AstNode::Add(left, right));
        ptr
    }

    pub fn new_list_node(&mut self, left: AstNodePtr, right: AstNodePtr) -> AstNodePtr {
        let ptr = self.vals.len();
        self.vals.push(AstNode::ListNode(left, right));
        ptr
    }

    pub fn new_fcn_call(&mut self, args: Option<AstNodePtr>) -> AstNodePtr {
        let ptr = self.vals.len();
        self.vals.push(AstNode::FcnCall(args));
        ptr
    }

    pub fn new_map_kv_pair(&mut self, key: AstNodePtr, val: AstNodePtr) -> AstNodePtr {
        let ptr = self.vals.len();
        self.vals.push(AstNode::MapKeyValPair { key, val });
        ptr
    }

    pub fn new_map_literal(&mut self, contents: Option<AstNodePtr>) -> AstNodePtr {
        let ptr = self.vals.len();
        self.vals.push(AstNode::MapLiteral(contents));
        ptr
    }

    pub fn new_list_literal(&mut self, contents: Option<AstNodePtr>) -> AstNodePtr {
        let ptr = self.vals.len();
        self.vals.push(AstNode::ListLiteral(contents));
        ptr
    }

    pub fn new_format_string(&mut self, contents: Option<AstNodePtr>) -> AstNodePtr {
        let ptr = self.vals.len();
        self.vals.push(AstNode::FormatString(contents));
        ptr
    }
}
