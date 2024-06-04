use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::hash::Hasher;
use std::{cell::OnceCell, collections::HashMap, hash::DefaultHasher, rc::Rc};

#[derive(Clone)]
pub struct Val {
    inner_val: Rc<InnerVal>,
}

struct InnerVal {
    hash: OnceCell<u64>,
    val_type: ValType,
}

pub enum ValType {
    Null,
    Err(String),
    Float64(f64),
    Bool(bool),
    String(String),
    List(Vec<Val>),
    Map(OrderedMap),
}

impl Val {
    pub fn get_val(&self) -> &ValType {
        &self.inner_val.val_type
    }

    fn new_val(val_type: ValType) -> Val {
        Val {
            inner_val: Rc::new(InnerVal {
                hash: OnceCell::new(),
                val_type,
            }),
        }
    }

    pub fn new_null() -> Val {
        Val::new_val(ValType::Null)
    }

    pub fn new_err(msg: &str) -> Val {
        Val::new_val(ValType::Err(msg.to_string()))
    }

    pub fn new_f64(val: f64) -> Val {
        Val::new_val(ValType::Float64(val))
    }

    pub fn new_bool(val: bool) -> Val {
        Val::new_val(ValType::Bool(val))
    }

    pub fn new_str(val: &str) -> Val {
        Val::new_val(ValType::String(val.to_string()))
    }

    pub fn new_list(val: Vec<Val>) -> Val {
        Val::new_val(ValType::List(val))
    }

    pub fn new_map(val: OrderedMap) -> Val {
        Val::new_val(ValType::Map(val))
    }

    fn get_hash(&self) -> u64 {
        *self.inner_val.hash.get_or_init(|| {
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

            let mut hasher = DefaultHasher::new();
            match self.get_val() {
                ValType::Null => {
                    HashTypes::Null.hash(&mut hasher);
                }
                ValType::Err(val) => {
                    HashTypes::Err.hash(&mut hasher);
                    val.hash(&mut hasher);
                }
                ValType::Float64(val) => {
                    HashTypes::Float64.hash(&mut hasher);
                    val.to_ne_bytes().hash(&mut hasher);
                }
                ValType::Bool(val) => {
                    HashTypes::Bool.hash(&mut hasher);
                    val.hash(&mut hasher);
                }
                ValType::String(val) => {
                    HashTypes::String.hash(&mut hasher);
                    val.hash(&mut hasher);
                }
                ValType::List(val) => {
                    HashTypes::List.hash(&mut hasher);
                    for elem in val {
                        elem.get_hash().hash(&mut hasher);
                    }
                }
                ValType::Map(val) => {
                    HashTypes::Map.hash(&mut hasher);

                    // get kv pairs sorted by key
                    let pairs = val.get_sorted_kv_pairs();
                    for (key, val) in pairs {
                        key.get_hash().hash(&mut hasher);
                        val.get_hash().hash(&mut hasher);
                    }
                }
            };
            hasher.finish()
        })
    }

    pub fn write_to_str(
        &self,
        writer: &mut impl std::io::Write,
        indent: u64,
        use_indent: bool,
    ) -> std::io::Result<usize> {
        self.inner_write_str(writer, indent, use_indent)?;
        if use_indent {
            writer.write("\n".as_bytes())?;
        }
        Ok(0)
    }

    fn inner_write_str(
        &self,
        writer: &mut impl std::io::Write,
        indent: u64,
        use_indent: bool,
    ) -> std::io::Result<usize> {
        fn write_indent(writer: &mut impl std::io::Write, indent: u64) -> std::io::Result<usize> {
            for _ in 0..indent {
                writer.write("  ".as_bytes())?;
            }
            Ok(0)
        }

        match self.get_val() {
            ValType::Null => {
                writer.write("null".as_bytes())?;
            }
            ValType::Float64(val) => {
                // TODO: Don't allocate on every float write
                writer.write(val.to_string().as_str().as_bytes())?;
            }
            ValType::Bool(val) => {
                if *val {
                    writer.write("true".as_bytes())?;
                } else {
                    writer.write("false".as_bytes())?;
                }
            }
            ValType::String(val) => {
                write_json_escaped_str(writer, val.as_str())?;
            }
            ValType::Err(val) => {
                writer.write("{\"ERROR\":".as_bytes())?;
                write_json_escaped_str(writer, val.as_str())?;
                writer.write("}".as_bytes())?;
            }
            ValType::List(val) => {
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
                    elem.inner_write_str(writer, indent + 1, use_indent)?;
                }
                if val.len() > 0 && use_indent {
                    writer.write("\n".as_bytes())?;
                    write_indent(writer, indent)?;
                }
                writer.write("]".as_bytes())?;
            }
            ValType::Map(val) => {
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
                    key.inner_write_str(writer, indent + 1, use_indent)?;
                    if use_indent {
                        writer.write(": ".as_bytes())?;
                    } else {
                        writer.write(":".as_bytes())?;
                    }
                    val.inner_write_str(writer, indent + 1, use_indent)?;
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
}

fn write_json_escaped_str(writer: &mut impl std::io::Write, val: &str) -> std::io::Result<usize> {
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

impl PartialEq for Val {
    fn eq(&self, other: &Self) -> bool {
        if self.get_hash() != other.get_hash() {
            return false;
        }
        return self.cmp(other) == Ordering::Equal;
    }
}

impl Eq for Val {}

impl PartialOrd for Val {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Val {
    fn cmp(&self, other: &Self) -> Ordering {
        let lval = self.get_val();
        let rval = other.get_val();
        match lval {
            ValType::Err(lval) => match rval {
                ValType::Err(rval) => lval.cmp(rval),
                ValType::Null => Ordering::Less,
                ValType::Bool(_) => Ordering::Less,
                ValType::Float64(_) => Ordering::Less,
                ValType::String(_) => Ordering::Less,
                ValType::List(_) => Ordering::Less,
                ValType::Map(_) => Ordering::Less,
            },
            ValType::Null => match rval {
                ValType::Err(_) => Ordering::Greater,
                ValType::Null => Ordering::Equal,
                ValType::Bool(_) => Ordering::Less,
                ValType::Float64(_) => Ordering::Less,
                ValType::String(_) => Ordering::Less,
                ValType::List(_) => Ordering::Less,
                ValType::Map(_) => Ordering::Less,
            },
            ValType::Bool(lval) => match rval {
                ValType::Err(_) => Ordering::Greater,
                ValType::Null => Ordering::Greater,
                ValType::Bool(rval) => lval.cmp(rval),
                ValType::Float64(_) => Ordering::Less,
                ValType::String(_) => Ordering::Less,
                ValType::List(_) => Ordering::Less,
                ValType::Map(_) => Ordering::Less,
            },
            ValType::Float64(lval) => match rval {
                ValType::Err(_) => Ordering::Greater,
                ValType::Null => Ordering::Greater,
                ValType::Bool(_) => Ordering::Greater,
                ValType::Float64(rval) => lval.total_cmp(rval),
                ValType::String(_) => Ordering::Less,
                ValType::List(_) => Ordering::Less,
                ValType::Map(_) => Ordering::Less,
            },
            ValType::String(lval) => match rval {
                ValType::Err(_) => Ordering::Greater,
                ValType::Null => Ordering::Greater,
                ValType::Bool(_) => Ordering::Greater,
                ValType::Float64(_) => Ordering::Greater,
                ValType::String(rval) => lval.cmp(rval),
                ValType::List(_) => Ordering::Less,
                ValType::Map(_) => Ordering::Less,
            },
            ValType::List(lval) => match rval {
                ValType::Err(_) => Ordering::Greater,
                ValType::Null => Ordering::Greater,
                ValType::Bool(_) => Ordering::Greater,
                ValType::Float64(_) => Ordering::Greater,
                ValType::String(_) => Ordering::Greater,
                ValType::List(rval) => list_cmp(lval, rval),
                ValType::Map(_) => Ordering::Less,
            },
            ValType::Map(lval) => match rval {
                ValType::Err(_) => Ordering::Greater,
                ValType::Null => Ordering::Greater,
                ValType::Bool(_) => Ordering::Greater,
                ValType::Float64(_) => Ordering::Greater,
                ValType::String(_) => Ordering::Greater,
                ValType::List(_) => Ordering::Greater,
                ValType::Map(rval) => map_cmp(lval, rval),
            },
        }
    }
}

fn list_cmp(left: &Vec<Val>, right: &Vec<Val>) -> Ordering {
    for idx in 0..std::cmp::min(left.len(), right.len()) {
        let result = left[idx].cmp(&right[idx]);
        if result != std::cmp::Ordering::Equal {
            return result;
        }
    }

    return left.len().cmp(&right.len());
}

fn map_cmp(left: &OrderedMap, right: &OrderedMap) -> Ordering {
    let lpairs = left.get_sorted_kv_pairs();
    let rpairs = right.get_sorted_kv_pairs();

    for idx in 0..std::cmp::min(lpairs.len(), rpairs.len()) {
        let (lkey, lval) = &lpairs[idx];
        let (rkey, rval) = &rpairs[idx];

        let key_ord = lkey.cmp(rkey);
        if key_ord != Ordering::Equal {
            return key_ord;
        }

        let val_ord = lval.cmp(rval);
        if val_ord != Ordering::Equal {
            return val_ord;
        }
    }

    return left.len().cmp(&right.len());
}

impl Hash for Val {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_hash().hash(state)
    }
}

pub struct OrderedMap {
    pairs: Vec<(Val, Val)>,
    key_to_idx: HashMap<Val, usize>,
}

impl OrderedMap {
    pub fn new() -> Self {
        OrderedMap {
            pairs: Vec::new(),
            key_to_idx: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &Val, val: &Val) {
        match self.key_to_idx.entry(key.clone()) {
            Entry::Occupied(entry) => {
                self.pairs[*entry.get()].1 = val.clone();
            }
            Entry::Vacant(entry) => {
                entry.insert(self.pairs.len());
                self.pairs.push((key.clone(), val.clone()));
            }
        }
    }

    pub fn get(&self, key: &Val) -> Option<Val> {
        match self.key_to_idx.get(key) {
            None => None,
            Some(idx) => Some(self.pairs[*idx].1.clone()),
        }
    }

    pub fn from_kv_pair_slice(slice: &[(Val, Val)]) -> Self {
        let mut map = OrderedMap::new();
        for (key, val) in slice {
            map.insert(key, val);
        }
        map
    }

    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    fn get_sorted_kv_pairs(&self) -> Vec<(Val, Val)> {
        let mut pairs = self.pairs.clone();
        pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
        return pairs;
    }
}

impl PartialEq for OrderedMap {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for OrderedMap {}

impl PartialOrd for OrderedMap {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedMap {
    fn cmp(&self, right: &Self) -> Ordering {
        let left = self;
        let lpairs = left.get_sorted_kv_pairs();
        let rpairs = right.get_sorted_kv_pairs();

        for idx in 0..std::cmp::min(lpairs.len(), rpairs.len()) {
            let (lkey, lval) = &lpairs[idx];
            let (rkey, rval) = &rpairs[idx];

            let key_ord = lkey.cmp(rkey);
            if key_ord != Ordering::Equal {
                return key_ord;
            }

            let val_ord = lval.cmp(rval);
            if val_ord != Ordering::Equal {
                return val_ord;
            }
        }

        return left.len().cmp(&right.len());
    }
}
