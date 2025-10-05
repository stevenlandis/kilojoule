use base64::{engine::general_purpose::STANDARD, Engine as _};
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
    Bytes(Vec<u8>),

    // Types
    IntType,
    FloatType,
    AnyType,
}

impl Val {
    pub fn get_val(&self) -> &ValType {
        &self.inner_val.val_type
    }

    pub fn new(val_type: ValType) -> Val {
        Val {
            inner_val: Rc::new(InnerVal {
                hash: OnceCell::new(),
                val_type,
            }),
        }
    }

    pub fn new_null() -> Val {
        Val::new(ValType::Null)
    }

    pub fn new_err(msg: &str) -> Val {
        Val::new(ValType::Err(msg.to_string()))
    }

    pub fn new_f64(val: f64) -> Val {
        Val::new(ValType::Float64(val))
    }

    pub fn new_bool(val: bool) -> Val {
        Val::new(ValType::Bool(val))
    }

    pub fn new_str(val: &str) -> Val {
        Val::new(ValType::String(val.to_string()))
    }

    pub fn new_list(val: Vec<Val>) -> Val {
        Val::new(ValType::List(val))
    }

    pub fn new_map(val: OrderedMap) -> Val {
        Val::new(ValType::Map(val))
    }

    pub fn new_bytes(val: Vec<u8>) -> Val {
        Val::new(ValType::Bytes(val))
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
                Bytes,
                // Types
                IntType,
                FloatType,
                AnyType,
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
                ValType::Bytes(val) => {
                    HashTypes::Bytes.hash(&mut hasher);

                    val.hash(&mut hasher);
                }
                ValType::IntType => {
                    HashTypes::IntType.hash(&mut hasher);
                }
                ValType::FloatType => {
                    HashTypes::FloatType.hash(&mut hasher);
                }
                ValType::AnyType => {
                    HashTypes::AnyType.hash(&mut hasher);
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
        if let ValType::Bytes(bytes) = self.get_val() {
            writer.write(bytes.as_slice())?;
            return Ok(0);
        }
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
                    match key.get_val() {
                        ValType::String(_) => {
                            key.inner_write_str(writer, indent + 1, use_indent)?;
                        }
                        _ => {
                            let mut temp_writer = Vec::<u8>::new();
                            key.inner_write_str(&mut temp_writer, 0, false)?;
                            let serialized_key =
                                std::str::from_utf8(temp_writer.as_slice()).unwrap();
                            write_json_escaped_str(writer, serialized_key)?;
                        }
                    }
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
            ValType::Bytes(val) => {
                writer.write("\"".as_bytes())?;
                writer.write(STANDARD.encode(val).as_bytes())?;
                writer.write("\"".as_bytes())?;
            }
            // Types
            ValType::IntType => {
                writer.write("%int".as_bytes())?;
            }
            ValType::FloatType => {
                writer.write("%float".as_bytes())?;
            }
            ValType::AnyType => {
                writer.write("%any".as_bytes())?;
            }
        }
        Ok(0)
    }

    pub fn from_json_str(json_str: &str) -> Self {
        match serde_json::from_str::<Val>(json_str) {
            Ok(val) => val,
            Err(_) => Val::new_err("unable to parse JSON"),
        }
    }

    pub fn from_toml_str(toml_str: &str) -> Self {
        match toml::from_str::<Val>(toml_str) {
            Ok(val) => val,
            Err(_) => Val::new_err("unable to parse toml"),
        }
    }

    pub fn from_yaml_str(toml_str: &str) -> Self {
        match serde_yaml::from_str::<Val>(toml_str) {
            Ok(val) => val,
            Err(_) => Val::new_err("unable to parse yaml"),
        }
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
                ValType::Bytes(_) => Ordering::Less,
                ValType::IntType => todo!(),
                ValType::FloatType => todo!(),
                ValType::AnyType => todo!(),
            },
            ValType::Null => match rval {
                ValType::Err(_) => Ordering::Greater,
                ValType::Null => Ordering::Equal,
                ValType::Bool(_) => Ordering::Less,
                ValType::Float64(_) => Ordering::Less,
                ValType::String(_) => Ordering::Less,
                ValType::List(_) => Ordering::Less,
                ValType::Map(_) => Ordering::Less,
                ValType::Bytes(_) => Ordering::Less,
                ValType::IntType => todo!(),
                ValType::FloatType => todo!(),
                ValType::AnyType => todo!(),
            },
            ValType::Bool(lval) => match rval {
                ValType::Err(_) => Ordering::Greater,
                ValType::Null => Ordering::Greater,
                ValType::Bool(rval) => lval.cmp(rval),
                ValType::Float64(_) => Ordering::Less,
                ValType::String(_) => Ordering::Less,
                ValType::List(_) => Ordering::Less,
                ValType::Map(_) => Ordering::Less,
                ValType::Bytes(_) => Ordering::Less,
                ValType::IntType => todo!(),
                ValType::FloatType => todo!(),
                ValType::AnyType => todo!(),
            },
            ValType::Float64(lval) => match rval {
                ValType::Err(_) => Ordering::Greater,
                ValType::Null => Ordering::Greater,
                ValType::Bool(_) => Ordering::Greater,
                ValType::Float64(rval) => lval.total_cmp(rval),
                ValType::String(_) => Ordering::Less,
                ValType::List(_) => Ordering::Less,
                ValType::Map(_) => Ordering::Less,
                ValType::Bytes(_) => Ordering::Less,
                ValType::IntType => todo!(),
                ValType::FloatType => todo!(),
                ValType::AnyType => todo!(),
            },
            ValType::String(lval) => match rval {
                ValType::Err(_) => Ordering::Greater,
                ValType::Null => Ordering::Greater,
                ValType::Bool(_) => Ordering::Greater,
                ValType::Float64(_) => Ordering::Greater,
                ValType::String(rval) => lval.cmp(rval),
                ValType::List(_) => Ordering::Less,
                ValType::Map(_) => Ordering::Less,
                ValType::Bytes(_) => Ordering::Less,
                ValType::IntType => todo!(),
                ValType::FloatType => todo!(),
                ValType::AnyType => todo!(),
            },
            ValType::List(lval) => match rval {
                ValType::Err(_) => Ordering::Greater,
                ValType::Null => Ordering::Greater,
                ValType::Bool(_) => Ordering::Greater,
                ValType::Float64(_) => Ordering::Greater,
                ValType::String(_) => Ordering::Greater,
                ValType::List(rval) => list_cmp(lval, rval),
                ValType::Map(_) => Ordering::Less,
                ValType::Bytes(_) => Ordering::Less,
                ValType::IntType => todo!(),
                ValType::FloatType => todo!(),
                ValType::AnyType => todo!(),
            },
            ValType::Map(lval) => match rval {
                ValType::Err(_) => Ordering::Greater,
                ValType::Null => Ordering::Greater,
                ValType::Bool(_) => Ordering::Greater,
                ValType::Float64(_) => Ordering::Greater,
                ValType::String(_) => Ordering::Greater,
                ValType::List(_) => Ordering::Greater,
                ValType::Map(rval) => map_cmp(lval, rval),
                ValType::Bytes(_) => Ordering::Less,
                ValType::IntType => todo!(),
                ValType::FloatType => todo!(),
                ValType::AnyType => todo!(),
            },
            ValType::Bytes(lval) => match rval {
                ValType::Err(_) => Ordering::Greater,
                ValType::Null => Ordering::Greater,
                ValType::Bool(_) => Ordering::Greater,
                ValType::Float64(_) => Ordering::Greater,
                ValType::String(_) => Ordering::Greater,
                ValType::List(_) => Ordering::Greater,
                ValType::Map(_) => Ordering::Greater,
                ValType::Bytes(rval) => bytes_cmp(lval, rval),
                ValType::IntType => todo!(),
                ValType::FloatType => todo!(),
                ValType::AnyType => todo!(),
            },
            ValType::IntType => todo!(),
            ValType::FloatType => todo!(),
            ValType::AnyType => todo!(),
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

fn bytes_cmp(left: &Vec<u8>, right: &Vec<u8>) -> Ordering {
    for idx in 0..std::cmp::min(left.len(), right.len()) {
        let result = left[idx].cmp(&right[idx]);
        if result != std::cmp::Ordering::Equal {
            return result;
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

    pub fn delete(&mut self, key: &Val) {
        match self.key_to_idx.remove(key) {
            None => {}
            Some(idx) => {
                self.pairs.remove(idx);
                for (_, other_idx) in self.key_to_idx.iter_mut() {
                    if *other_idx > idx {
                        *other_idx -= 1;
                    }
                }
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

    pub fn get_kv_pair_slice(&self) -> &[(Val, Val)] {
        &self.pairs
    }

    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    fn get_sorted_kv_pairs(&self) -> Vec<(Val, Val)> {
        let mut pairs = self.pairs.clone();
        pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
        return pairs;
    }

    pub fn keys(&self) -> Vec<Val> {
        self.pairs.iter().map(|(key, _)| key.clone()).collect()
    }

    pub fn values(&self) -> Vec<Val> {
        self.pairs.iter().map(|(_, val)| val.clone()).collect()
    }

    pub fn items(&self) -> Vec<Val> {
        self.pairs
            .iter()
            .map(|(key, val)| {
                Val::new_map(OrderedMap::from_kv_pair_slice(&[
                    (Val::new_str("key"), key.clone()),
                    (Val::new_str("val"), val.clone()),
                ]))
            })
            .collect()
    }

    pub fn has(&self, key: &Val) -> bool {
        self.key_to_idx.contains_key(key)
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

struct ValVisitor {}

impl ValVisitor {
    fn new() -> Self {
        ValVisitor {}
    }
}

impl<'de> serde::de::Visitor<'de> for ValVisitor {
    type Value = Val;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a Val")
    }

    fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut map = OrderedMap::new();

        while let Some((key, value)) = access.next_entry::<Val, Val>()? {
            map.insert(&key, &value);
        }

        Ok(Val::new_map(map))
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_bool(v))
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_f64(v as f64))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_f64(v as f64))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_f64(v as f64))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_f64(v as f64))
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_f64(v as f64))
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_f64(v as f64))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_f64(v as f64))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_f64(v as f64))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_f64(v as f64))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_f64(v as f64))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_f64(v))
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_f64(v as f64))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_str(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_str(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_str(v.as_str()))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut vals = Vec::<Val>::new();

        while let Some(val) = seq.next_element::<Val>()? {
            vals.push(val)
        }

        Ok(Val::new_list(vals))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_bytes(v.to_vec()))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_bytes(v.to_vec()))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_bytes(v.to_vec()))
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::from_json_str(v.to_string().as_str()))
    }

    fn visit_enum<A>(self, _data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        Ok(Val::new_err("\"enum\" parsing is unimplemented"))
    }

    fn visit_newtype_struct<D>(self, _deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Val::new_err("\"newtype_struct\" parsing is unimplemented"))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_null())
    }

    fn visit_some<D>(self, _deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Val::new_err("\"some\" parsing is unimplemented"))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Val::new_null())
    }
}

impl<'de> serde::de::Deserialize<'de> for Val {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ValVisitor::new())
    }
}

impl serde::ser::Serialize for Val {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.get_val() {
            ValType::Null => serializer.serialize_unit(),
            ValType::Err(err) => serializer.collect_map(
                [(Val::new_str("ERROR"), Val::new_str(err))]
                    .iter()
                    .map(|(key, val)| (key, val)),
            ),
            ValType::Float64(val) => {
                if *val == val.trunc() {
                    serializer.serialize_i64(*val as i64)
                } else {
                    serializer.serialize_f64(*val)
                }
            }
            ValType::Bool(val) => serializer.serialize_bool(*val),
            ValType::String(val) => serializer.serialize_str(val.as_str()),
            ValType::List(val) => serializer.collect_seq(val.iter()),
            ValType::Map(val) => {
                serializer.collect_map(val.get_kv_pair_slice().iter().map(|(key, val)| (key, val)))
            }
            ValType::Bytes(val) => serializer.serialize_str(STANDARD.encode(val).as_str()),
            ValType::IntType => serialize_as_str(self, serializer),
            ValType::FloatType => serialize_as_str(self, serializer),
            ValType::AnyType => serialize_as_str(self, serializer),
        }
    }
}

fn serialize_as_str<S>(val: &Val, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut writer = Vec::<u8>::new();
    val.write_to_str(&mut writer, 0, false).unwrap();
    serializer.serialize_str(String::from_utf8(writer).unwrap().as_str())
}
