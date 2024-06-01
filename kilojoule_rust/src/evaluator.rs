use super::ast_node_pool::{AstNode, AstNodePtr};
use super::obj_pool::{ObjPool, ObjPoolObjValue, ObjPoolRef, OrderedMap};
use super::parser::Parser;

pub struct Evaluator {
    obj_pool: ObjPool,
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {
            obj_pool: ObjPool::new(),
        }
    }

    pub fn parse_and_eval(&mut self, text: &str) -> ObjPoolRef {
        let mut parser = Parser::new(text);
        match parser.parse_expr() {
            None => self.obj_pool.new_null(),
            Some(ast) => match ast {
                Err(_) => self.obj_pool.new_err("Parse Error"),
                Ok(ast) => {
                    // for (idx, val) in parser.pool.vals.iter().enumerate() {
                    //     println!("{}: {:?}", idx, val);
                    // }
                    let val = self.obj_pool.new_null();
                    self.eval(ast, val, &parser)
                }
            },
        }
    }

    fn eval(&mut self, node: AstNodePtr, obj: ObjPoolRef, parser: &Parser) -> ObjPoolRef {
        match parser.get_node(node) {
            AstNode::Null => self.obj_pool.new_null(),
            AstNode::Pipe(left, right) => {
                let left_val = self.eval(*left, obj, parser);
                self.eval(*right, left_val, parser)
            }
            AstNode::Dot => obj,
            AstNode::Add(left, right) => {
                let left_val = self.eval(*left, obj, parser);
                let left_val = match self.obj_pool.get(left_val) {
                    ObjPoolObjValue::Float64(val) => *val,
                    _ => {
                        return self
                            .obj_pool
                            .new_err("Left side of addition has to be a float");
                    }
                };
                let right_val = self.eval(*right, obj, parser);
                let right_val = match self.obj_pool.get(right_val) {
                    ObjPoolObjValue::Float64(val) => val,
                    _ => {
                        return self
                            .obj_pool
                            .new_err("Right side of addition has to be a float");
                    }
                };
                self.obj_pool.new_f64(left_val + right_val)
            }
            AstNode::Integer(val) => self.obj_pool.new_f64(*val as f64),
            AstNode::MapLiteral(contents) => {
                let mut map = OrderedMap::new();
                fn helper(
                    this: &mut Evaluator,
                    obj: ObjPoolRef,
                    parser: &Parser,
                    map: &mut OrderedMap,
                    node: AstNodePtr,
                ) {
                    match parser.get_node(node) {
                        AstNode::ListNode(left, right) => {
                            helper(this, obj, parser, map, *left);
                            helper(this, obj, parser, map, *right);
                        }
                        AstNode::MapKeyValPair { key, val } => {
                            let key_obj = match parser.get_node(*key) {
                                AstNode::Identifier(key_name) => this.obj_pool.new_str(key_name),
                                _ => panic!(),
                            };
                            let val_obj = this.eval(*val, obj, parser);
                            map.insert(&this.obj_pool, key_obj, val_obj);
                        }
                        _ => panic!(),
                    }
                }

                match contents {
                    None => {}
                    Some(contents) => {
                        helper(self, obj, parser, &mut map, *contents);
                    }
                };

                self.obj_pool.new_map(map)
            }
            AstNode::Access(expr) => match &self.obj_pool.get(obj) {
                ObjPoolObjValue::Map(_) => {
                    let key_val = match parser.get_node(*expr) {
                        AstNode::Identifier(key) => self.obj_pool.new_str(key),
                        _ => panic!(),
                    };
                    let map = match &self.obj_pool.get(obj) {
                        ObjPoolObjValue::Map(map) => map,
                        _ => panic!(),
                    };
                    match map.get(&self.obj_pool, key_val) {
                        None => self.obj_pool.new_null(),
                        Some(val) => val,
                    }
                }
                _ => panic!(),
            },
            _ => panic!("Unimplemented {:?}", parser.get_node(node)),
        }
    }

    pub fn write_val(
        &self,
        val: ObjPoolRef,
        writer: &mut impl std::io::Write,
        use_indent: bool,
    ) -> std::io::Result<()> {
        match self.obj_pool.write_to_str(writer, val, 0, use_indent) {
            Err(err) => Err(err),
            Ok(_) => Ok(()),
        }
    }
}
