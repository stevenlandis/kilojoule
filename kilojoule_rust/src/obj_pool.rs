use std::cell::OnceCell;
use std::cmp::Ordering;
use std::hash::Hasher;
use std::hash::{DefaultHasher, Hash};

#[derive(Clone, Copy)]
pub struct ObjPoolRef {
    idx: usize,
}

pub enum ObjPoolObjValue {
    Null,
    Err(String),
    Float64(f64),
    Bool(bool),
    String(String),
    List(Vec<ObjPoolRef>),
    Map(OrderedMap),
}

#[derive(Hash)]
enum HashTypes {
    Null,
    Err,
    Float64,
    Bool,
    String,
    List,
    Map,
}

struct ObjPoolObj {
    ref_count: usize,
    hash: OnceCell<u64>,
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
            hash: OnceCell::new(),
            value: ObjPoolObjValue::Null,
        });
        ObjPoolRef { idx }
    }

    pub fn new_err(&mut self, msg: &str) -> ObjPoolRef {
        let idx = self.vals.len();
        self.vals.push(ObjPoolObj {
            ref_count: 0,
            hash: OnceCell::new(),
            value: ObjPoolObjValue::Err(msg.to_string()),
        });
        ObjPoolRef { idx }
    }

    pub fn new_f64(&mut self, val: f64) -> ObjPoolRef {
        let idx = self.vals.len();
        self.vals.push(ObjPoolObj {
            ref_count: 0,
            hash: OnceCell::new(),
            value: ObjPoolObjValue::Float64(val),
        });
        ObjPoolRef { idx }
    }

    pub fn new_bool(&mut self, val: bool) -> ObjPoolRef {
        let idx = self.vals.len();
        self.vals.push(ObjPoolObj {
            ref_count: 0,
            hash: OnceCell::new(),
            value: ObjPoolObjValue::Bool(val),
        });
        ObjPoolRef { idx }
    }

    pub fn new_str(&mut self, val: &str) -> ObjPoolRef {
        let idx = self.vals.len();
        self.vals.push(ObjPoolObj {
            ref_count: 0,
            hash: OnceCell::new(),
            value: ObjPoolObjValue::String(val.to_string()),
        });
        ObjPoolRef { idx }
    }

    pub fn new_list(&mut self, list: Vec<ObjPoolRef>) -> ObjPoolRef {
        let idx = self.vals.len();
        self.vals.push(ObjPoolObj {
            ref_count: 0,
            hash: OnceCell::new(),
            value: ObjPoolObjValue::List(list),
        });
        ObjPoolRef { idx }
    }

    pub fn new_map(&mut self, map: OrderedMap) -> ObjPoolRef {
        let idx = self.vals.len();
        self.vals.push(ObjPoolObj {
            ref_count: 0,
            hash: OnceCell::new(),
            value: ObjPoolObjValue::Map(map),
        });
        ObjPoolRef { idx }
    }

    pub fn new_map_from_iter<'a>(
        &mut self,
        pairs: impl Iterator<Item = &'a (ObjPoolRef, ObjPoolRef)>,
    ) -> ObjPoolRef {
        let idx = self.vals.len();
        let mut map = OrderedMap::new();
        for (key, val) in pairs {
            map.insert(self, *key, *val);
        }

        self.vals.push(ObjPoolObj {
            ref_count: 0,
            hash: OnceCell::new(),
            value: ObjPoolObjValue::Map(map),
        });
        ObjPoolRef { idx }
    }

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
                ObjPoolObjValue::Bool(_) => {}
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
        for ch in val.as_bytes() {
            match *ch as char {
                '\n' => {
                    writer.write("\\n".as_bytes())?;
                }
                '\t' => {
                    writer.write("\\t".as_bytes())?;
                }
                '\r' => {
                    writer.write("\\r".as_bytes())?;
                }
                '"' => {
                    writer.write("\\\"".as_bytes())?;
                }
                '\\' => {
                    writer.write("\\\\".as_bytes())?;
                }
                _ => {
                    writer.write(&[*ch])?;
                }
            }
        }
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
            ObjPoolObjValue::Bool(val) => {
                if *val {
                    writer.write("true".as_bytes())?;
                } else {
                    writer.write("false".as_bytes())?;
                }
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

    pub fn val_equals(&self, left: ObjPoolRef, right: ObjPoolRef) -> bool {
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
            ObjPoolObjValue::Bool(left) => match &self.vals[right.idx].value {
                ObjPoolObjValue::Bool(right) => left == right,
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

    pub fn cmp_values(&self, left: ObjPoolRef, right: ObjPoolRef) -> Ordering {
        let lval = &self.vals[left.idx].value;
        let rval = &self.vals[right.idx].value;
        match lval {
            ObjPoolObjValue::Err(lval) => match rval {
                ObjPoolObjValue::Err(rval) => lval.cmp(rval),
                ObjPoolObjValue::Null => Ordering::Less,
                ObjPoolObjValue::Bool(_) => Ordering::Less,
                ObjPoolObjValue::Float64(_) => Ordering::Less,
                ObjPoolObjValue::String(_) => Ordering::Less,
                ObjPoolObjValue::List(_) => Ordering::Less,
                ObjPoolObjValue::Map(_) => Ordering::Less,
            },
            ObjPoolObjValue::Null => match rval {
                ObjPoolObjValue::Err(_) => Ordering::Greater,
                ObjPoolObjValue::Null => Ordering::Equal,
                ObjPoolObjValue::Bool(_) => Ordering::Less,
                ObjPoolObjValue::Float64(_) => Ordering::Less,
                ObjPoolObjValue::String(_) => Ordering::Less,
                ObjPoolObjValue::List(_) => Ordering::Less,
                ObjPoolObjValue::Map(_) => Ordering::Less,
            },
            ObjPoolObjValue::Bool(lval) => match rval {
                ObjPoolObjValue::Err(_) => Ordering::Greater,
                ObjPoolObjValue::Null => Ordering::Greater,
                ObjPoolObjValue::Bool(rval) => lval.cmp(rval),
                ObjPoolObjValue::Float64(_) => Ordering::Less,
                ObjPoolObjValue::String(_) => Ordering::Less,
                ObjPoolObjValue::List(_) => Ordering::Less,
                ObjPoolObjValue::Map(_) => Ordering::Less,
            },
            ObjPoolObjValue::Float64(lval) => match rval {
                ObjPoolObjValue::Err(_) => Ordering::Greater,
                ObjPoolObjValue::Null => Ordering::Greater,
                ObjPoolObjValue::Bool(_) => Ordering::Greater,
                ObjPoolObjValue::Float64(rval) => lval.total_cmp(rval),
                ObjPoolObjValue::String(_) => Ordering::Less,
                ObjPoolObjValue::List(_) => Ordering::Less,
                ObjPoolObjValue::Map(_) => Ordering::Less,
            },
            ObjPoolObjValue::String(lval) => match rval {
                ObjPoolObjValue::Err(_) => Ordering::Greater,
                ObjPoolObjValue::Null => Ordering::Greater,
                ObjPoolObjValue::Bool(_) => Ordering::Greater,
                ObjPoolObjValue::Float64(_) => Ordering::Greater,
                ObjPoolObjValue::String(rval) => lval.cmp(rval),
                ObjPoolObjValue::List(_) => Ordering::Less,
                ObjPoolObjValue::Map(_) => Ordering::Less,
            },
            ObjPoolObjValue::List(lval) => match rval {
                ObjPoolObjValue::Err(_) => Ordering::Greater,
                ObjPoolObjValue::Null => Ordering::Greater,
                ObjPoolObjValue::Bool(_) => Ordering::Greater,
                ObjPoolObjValue::Float64(_) => Ordering::Greater,
                ObjPoolObjValue::String(_) => Ordering::Greater,
                ObjPoolObjValue::List(rval) => self.list_cmp(lval, rval),
                ObjPoolObjValue::Map(_) => Ordering::Less,
            },
            ObjPoolObjValue::Map(lval) => match rval {
                ObjPoolObjValue::Err(_) => Ordering::Greater,
                ObjPoolObjValue::Null => Ordering::Greater,
                ObjPoolObjValue::Bool(_) => Ordering::Greater,
                ObjPoolObjValue::Float64(_) => Ordering::Greater,
                ObjPoolObjValue::String(_) => Ordering::Greater,
                ObjPoolObjValue::List(_) => Ordering::Greater,
                ObjPoolObjValue::Map(rval) => self.map_cmp(lval, rval),
            },
        }
    }

    fn list_cmp(&self, left: &Vec<ObjPoolRef>, right: &Vec<ObjPoolRef>) -> Ordering {
        for idx in 0..std::cmp::min(left.len(), right.len()) {
            let result = self.cmp_values(left[idx], right[idx]);
            if result != std::cmp::Ordering::Equal {
                return result;
            }
        }

        return left.len().cmp(&right.len());
    }

    fn map_cmp(&self, left: &OrderedMap, right: &OrderedMap) -> Ordering {
        let len_cmp = left.len().cmp(&right.len());
        if len_cmp == Ordering::Equal {
            return Ordering::Equal;
        }

        let lpairs = self.map_get_sorted_kv_pairs(left);
        let rpairs = self.map_get_sorted_kv_pairs(right);
        assert_eq!(lpairs.len(), rpairs.len());

        for idx in 0..lpairs.len() {
            let result = self.cmp_values(lpairs[idx].0, rpairs[idx].0);
            if result != std::cmp::Ordering::Equal {
                return result;
            }
        }

        Ordering::Equal
    }

    fn map_get_sorted_kv_pairs(&self, map: &OrderedMap) -> Vec<(ObjPoolRef, ObjPoolRef)> {
        let mut pairs = map.pairs.clone();
        pairs.sort_by(|(left, _), (right, _)| self.cmp_values(*left, *right));
        return pairs;
    }

    fn get_hash(&self, obj: ObjPoolRef) -> u64 {
        *self.vals[obj.idx].hash.get_or_init(|| {
            let mut hasher = DefaultHasher::new();
            match &self.vals[obj.idx].value {
                ObjPoolObjValue::Null => {
                    HashTypes::Null.hash(&mut hasher);
                }
                ObjPoolObjValue::Err(val) => {
                    HashTypes::Err.hash(&mut hasher);
                    val.hash(&mut hasher);
                }
                ObjPoolObjValue::Float64(val) => {
                    HashTypes::Float64.hash(&mut hasher);
                    val.to_ne_bytes().hash(&mut hasher);
                }
                ObjPoolObjValue::Bool(val) => {
                    HashTypes::Bool.hash(&mut hasher);
                    val.hash(&mut hasher);
                }
                ObjPoolObjValue::String(val) => {
                    HashTypes::String.hash(&mut hasher);
                    val.hash(&mut hasher);
                }
                ObjPoolObjValue::List(val) => {
                    HashTypes::List.hash(&mut hasher);

                    // TODO: Don't clone here
                    let val = val.clone();
                    for elem in val {
                        self.get_hash(elem).hash(&mut hasher);
                    }
                }
                ObjPoolObjValue::Map(val) => {
                    HashTypes::Map.hash(&mut hasher);

                    // get kv pairs sorted by key
                    let pairs = self.map_get_sorted_kv_pairs(val);
                    for (key, val) in pairs {
                        self.get_hash(key).hash(&mut hasher);
                        self.get_hash(val).hash(&mut hasher);
                    }
                }
            };
            hasher.finish()
        })
    }
}

pub struct OrderedMap {
    pairs: Vec<(ObjPoolRef, ObjPoolRef)>,
    key_to_idx: Vec<Option<(ObjPoolRef, usize)>>,
    // key_to_idx: Vec<Option<usize>>,
}

impl OrderedMap {
    pub fn new() -> Self {
        OrderedMap {
            pairs: Vec::new(),
            key_to_idx: Vec::new(),
        }
    }

    fn resize_key_to_idx(&mut self, pool: &ObjPool) {
        if self.key_to_idx.len() < (self.pairs.len() + 1) << 1 {
            let mut size: usize = 1;
            while size < (self.pairs.len() + 1) << 1 {
                size <<= 1;
            }

            self.key_to_idx = vec![None; size];

            for (idx, (key, _)) in self.pairs.iter().enumerate() {
                let hash = pool.get_hash(*key);
                let mut hash_idx = (hash as usize) % size;
                loop {
                    match &self.key_to_idx[hash_idx] {
                        None => {
                            self.key_to_idx[hash_idx] = Some((*key, idx));
                            break;
                        }
                        Some(_) => {
                            hash_idx += 1;
                            if hash_idx == size {
                                hash_idx = 0;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn insert(&mut self, pool: &ObjPool, key: ObjPoolRef, val: ObjPoolRef) {
        self.resize_key_to_idx(pool);

        let hash = pool.get_hash(key);
        let mut hash_idx = (hash as usize) % self.key_to_idx.len();
        loop {
            match &self.key_to_idx[hash_idx] {
                None => {
                    self.key_to_idx[hash_idx] = Some((key, self.pairs.len()));
                    self.pairs.push((key, val));
                    break;
                }
                Some((existing_key, existing_idx)) => {
                    if pool.val_equals(key, *existing_key) {
                        self.pairs[*existing_idx].1 = val;
                        break;
                    } else {
                        hash_idx += 1;
                        if hash_idx == self.key_to_idx.len() {
                            hash_idx = 0;
                        }
                    }
                }
            }
        }
    }

    pub fn get(&self, pool: &ObjPool, key: ObjPoolRef) -> Option<ObjPoolRef> {
        let hash = pool.get_hash(key);
        let mut hash_idx = (hash as usize) % self.key_to_idx.len();
        loop {
            match &self.key_to_idx[hash_idx] {
                None => {
                    return None;
                }
                Some((existing_key, existing_idx)) => {
                    if pool.val_equals(key, *existing_key) {
                        return Some(self.pairs[*existing_idx].1);
                    } else {
                        hash_idx += 1;
                        if hash_idx == self.key_to_idx.len() {
                            hash_idx = 0;
                        }
                    }
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.pairs.len()
    }
}
