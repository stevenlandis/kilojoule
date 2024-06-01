#[derive(Clone, Copy)]
pub struct ObjPoolRef {
    idx: usize,
}

pub enum ObjPoolObjValue {
    Null,
    Err(String),
    Float64(f64),
    String(String),
    List(Vec<ObjPoolRef>),
    Map(OrderedMap),
}

struct ObjPoolObj {
    ref_count: usize,
    value: ObjPoolObjValue,
}

pub struct ObjPool {
    vals: Vec<ObjPoolObj>,
}

impl ObjPool {
    pub fn new() -> Self {
        ObjPool { vals: Vec::new() }
    }

    pub fn get(&self, ptr: ObjPoolRef) -> &ObjPoolObjValue {
        &self.vals[ptr.idx].value
    }

    pub fn new_null(&mut self) -> ObjPoolRef {
        let idx = self.vals.len();
        self.vals.push(ObjPoolObj {
            ref_count: 0,
            value: ObjPoolObjValue::Null,
        });
        ObjPoolRef { idx }
    }

    pub fn new_err(&mut self, msg: &str) -> ObjPoolRef {
        let idx = self.vals.len();
        self.vals.push(ObjPoolObj {
            ref_count: 0,
            value: ObjPoolObjValue::Err(msg.to_string()),
        });
        ObjPoolRef { idx }
    }

    pub fn new_f64(&mut self, val: f64) -> ObjPoolRef {
        let idx = self.vals.len();
        self.vals.push(ObjPoolObj {
            ref_count: 0,
            value: ObjPoolObjValue::Float64(val),
        });
        ObjPoolRef { idx }
    }

    pub fn new_str(&mut self, val: &str) -> ObjPoolRef {
        let idx = self.vals.len();
        self.vals.push(ObjPoolObj {
            ref_count: 0,
            value: ObjPoolObjValue::String(val.to_string()),
        });
        ObjPoolRef { idx }
    }

    // fn new_list(&mut self) -> ObjPoolRef {
    //     let idx = self.vals.len();
    //     self.vals.push(ObjPoolObj {
    //         ref_count: 0,
    //         value: ObjPoolObjValue::List(Vec::new()),
    //     });
    //     ObjPoolRef { idx }
    // }

    pub fn new_map(&mut self, map: OrderedMap) -> ObjPoolRef {
        let idx = self.vals.len();
        self.vals.push(ObjPoolObj {
            ref_count: 0,
            value: ObjPoolObjValue::Map(map),
        });
        ObjPoolRef { idx }
    }

    // fn get_f64(&self, obj: ObjPoolRef) -> f64 {
    //     match self.vals[obj.idx].value {
    //         ObjPoolObjValue::Float64(val) => val,
    //         _ => panic!(),
    //     }
    // }

    fn get_list(&self, obj: ObjPoolRef) -> &[ObjPoolRef] {
        match &self.vals[obj.idx].value {
            ObjPoolObjValue::List(list) => list.as_slice(),
            _ => panic!(),
        }
    }

    fn incr_ref(&mut self, obj: ObjPoolRef) {
        self.vals[obj.idx].ref_count += 1;
    }

    fn decr_ref(&mut self, obj: ObjPoolRef) {
        assert!(self.vals[obj.idx].ref_count > 0);
        self.vals[obj.idx].ref_count -= 1;
    }

    fn collect_garbage(&mut self) {
        while self.vals.len() > 0 && self.vals[self.vals.len() - 1].ref_count == 0 {
            let top = self.vals.pop().unwrap();
            match top.value {
                ObjPoolObjValue::Null => {}
                ObjPoolObjValue::Err(_) => {}
                ObjPoolObjValue::Float64(_) => {}
                ObjPoolObjValue::String(_) => {}
                ObjPoolObjValue::List(val) => {
                    for elem in val {
                        self.decr_ref(elem);
                    }
                }
                ObjPoolObjValue::Map(val) => {
                    for (key, val) in val.pairs {
                        self.decr_ref(key);
                        self.decr_ref(val);
                    }
                }
            }
        }
    }

    fn list_push(&mut self, obj: ObjPoolRef) {
        match &mut self.vals[obj.idx].value {
            ObjPoolObjValue::List(val) => {
                val.push(obj);
                self.incr_ref(obj);
            }
            _ => panic!(),
        }
    }

    fn write_json_escaped_str(
        &self,
        writer: &mut impl std::io::Write,
        val: &str,
    ) -> std::io::Result<usize> {
        writer.write("\"".as_bytes())?;
        writer.write(val.as_bytes())?;
        writer.write("\"".as_bytes())?;

        Ok(0)
    }

    pub fn write_to_str(
        &self,
        writer: &mut impl std::io::Write,
        val: ObjPoolRef,
        indent: u64,
        use_indent: bool,
    ) -> std::io::Result<usize> {
        self.inner_write_str(writer, val, indent, use_indent)?;
        if use_indent {
            writer.write("\n".as_bytes())?;
        }
        Ok(0)
    }

    fn inner_write_str(
        &self,
        writer: &mut impl std::io::Write,
        val: ObjPoolRef,
        indent: u64,
        use_indent: bool,
    ) -> std::io::Result<usize> {
        fn write_indent(writer: &mut impl std::io::Write, indent: u64) -> std::io::Result<usize> {
            for _ in 0..indent {
                writer.write("  ".as_bytes())?;
            }
            Ok(0)
        }

        match &self.vals[val.idx].value {
            ObjPoolObjValue::Null => {
                writer.write("null".as_bytes())?;
            }
            ObjPoolObjValue::Float64(val) => {
                // TODO: Don't allocate on every float write
                writer.write(val.to_string().as_str().as_bytes())?;
            }
            ObjPoolObjValue::String(val) => {
                self.write_json_escaped_str(writer, val.as_str())?;
            }
            ObjPoolObjValue::Err(val) => {
                writer.write("{\"ERROR\":".as_bytes())?;
                self.write_json_escaped_str(writer, val.as_str())?;
                writer.write("}".as_bytes())?;
            }
            ObjPoolObjValue::List(val) => {
                writer.write("[".as_bytes())?;
                for (idx, elem) in val.iter().enumerate() {
                    if idx > 0 {
                        if use_indent {
                            writer.write(", ".as_bytes())?;
                        } else {
                            writer.write(",".as_bytes())?;
                        }
                    }
                    if use_indent {
                        writer.write("\n".as_bytes())?;
                        write_indent(writer, indent + 1)?;
                    }
                    self.inner_write_str(writer, *elem, indent + 1, use_indent)?;
                }
                if val.len() > 0 && use_indent {
                    writer.write("\n".as_bytes())?;
                    write_indent(writer, indent)?;
                }
                writer.write("]".as_bytes())?;
            }
            ObjPoolObjValue::Map(val) => {
                writer.write("{".as_bytes())?;
                for (idx, (key, val)) in val.pairs.iter().enumerate() {
                    if idx > 0 {
                        if use_indent {
                            writer.write(", ".as_bytes())?;
                        } else {
                            writer.write(",".as_bytes())?;
                        }
                    }
                    if use_indent {
                        writer.write("\n".as_bytes())?;
                        write_indent(writer, indent + 1)?;
                    }
                    self.inner_write_str(writer, *key, indent + 1, use_indent)?;
                    if use_indent {
                        writer.write(": ".as_bytes())?;
                    } else {
                        writer.write(":".as_bytes())?;
                    }
                    self.inner_write_str(writer, *val, indent + 1, use_indent)?;
                }
                if val.pairs.len() > 0 && use_indent {
                    writer.write("\n".as_bytes())?;
                    write_indent(writer, indent)?;
                }
                writer.write("}".as_bytes())?;
            }
        }
        Ok(0)
    }

    fn val_equals(&self, left: ObjPoolRef, right: ObjPoolRef) -> bool {
        match &self.vals[left.idx].value {
            ObjPoolObjValue::Null => match self.vals[right.idx].value {
                ObjPoolObjValue::Null => true,
                _ => false,
            },
            ObjPoolObjValue::Err(left) => match &self.vals[right.idx].value {
                ObjPoolObjValue::Err(right) => left == right,
                _ => false,
            },
            ObjPoolObjValue::Float64(left) => match self.vals[right.idx].value {
                ObjPoolObjValue::Float64(right) => {
                    left.total_cmp(&right) == std::cmp::Ordering::Equal
                }
                _ => false,
            },
            ObjPoolObjValue::String(left) => match &self.vals[right.idx].value {
                ObjPoolObjValue::String(right) => left == right,
                _ => false,
            },
            ObjPoolObjValue::List(left) => match &self.vals[right.idx].value {
                ObjPoolObjValue::List(right) => {
                    if left.len() != right.len() {
                        return false;
                    }
                    for (left_val, right_val) in std::iter::zip(left, right) {
                        if !self.val_equals(*left_val, *right_val) {
                            return false;
                        }
                    }
                    return true;
                }
                _ => false,
            },
            ObjPoolObjValue::Map(left) => match &self.vals[right.idx].value {
                ObjPoolObjValue::Map(right) => {
                    if left.pairs.len() != right.pairs.len() {
                        return false;
                    }
                    for (left_key, left_val) in &left.pairs {
                        let mut found_match = false;
                        for (right_key, right_val) in &right.pairs {
                            if self.val_equals(*left_key, *right_key) {
                                found_match = true;
                                if !self.val_equals(*left_val, *right_val) {
                                    return false;
                                }
                                break;
                            }
                        }
                        if !found_match {
                            return false;
                        }
                    }
                    return true;
                }
                _ => false,
            },
        }
    }
}

pub struct OrderedMap {
    pairs: Vec<(ObjPoolRef, ObjPoolRef)>,
}

impl OrderedMap {
    pub fn new() -> Self {
        OrderedMap { pairs: Vec::new() }
    }

    pub fn insert(&mut self, pool: &ObjPool, key: ObjPoolRef, val: ObjPoolRef) {
        for (loop_key, loop_val) in &mut self.pairs {
            if pool.val_equals(*loop_key, key) {
                *loop_val = val;
                return;
            }
        }

        self.pairs.push((key, val));
    }

    pub fn get(&self, pool: &ObjPool, key: ObjPoolRef) -> Option<ObjPoolRef> {
        for (loop_key, loop_val) in &self.pairs {
            if pool.val_equals(*loop_key, key) {
                return Some(*loop_val);
            }
        }
        None
    }
}
