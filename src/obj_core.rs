struct ObjectType {}
impl ObjectType {
    const INT_U64: u8 = 1;
    const INT_I64: u8 = 2;
    const STRING: u8 = 3;
    const LIST: u8 = 4;
    const OBJECT: u8 = 5;
}

pub fn write_u64(vec: &mut Vec<u8>, val: u64) {
    vec.push(ObjectType::INT_U64);
    internal_append_u64(vec, val);
}

pub fn read_u64(vec: &Vec<u8>, idx: usize) -> u64 {
    assert_eq!(vec[idx], ObjectType::INT_U64);
    let idx = idx + 1;
    internal_read_u64(vec, idx)
}

pub fn write_i64(vec: &mut Vec<u8>, val: i64) {
    vec.push(ObjectType::INT_I64);
    vec.extend_from_slice(&val.to_le_bytes());
}

pub fn read_i64(vec: &Vec<u8>, idx: usize) -> i64 {
    assert_eq!(vec[idx], ObjectType::INT_I64);
    let idx = idx + 1;
    let bytes: [u8; 8] = vec[idx..idx + 8].try_into().unwrap();
    i64::from_le_bytes(bytes)
}

pub fn write_string(vec: &mut Vec<u8>, val: &str) {
    vec.push(ObjectType::STRING);
    internal_append_u64(vec, val.len() as u64);
    vec.extend_from_slice(val.as_bytes());
}

pub fn read_string(vec: &Vec<u8>, idx: usize) -> &str {
    assert_eq!(vec[idx], ObjectType::STRING);
    let idx = idx + 1;
    let len = internal_read_u64(vec, idx);
    let idx = idx + 8;
    str::from_utf8(&vec[idx..(idx + (len as usize))]).unwrap()
}

fn internal_read_u64(vec: &Vec<u8>, idx: usize) -> u64 {
    let bytes: [u8; 8] = vec[idx..idx + 8].try_into().unwrap();
    u64::from_le_bytes(bytes)
}

fn internal_write_u64(vec: &mut Vec<u8>, idx: usize, val: u64) {
    vec[idx..idx + 8].copy_from_slice(&val.to_le_bytes());
}

fn internal_append_u64(vec: &mut Vec<u8>, val: u64) {
    vec.extend_from_slice(&val.to_le_bytes());
}

pub struct ListStart {
    idx: usize,
}

pub fn start_list(vec: &mut Vec<u8>) -> ListStart {
    /*
    Lists have the following layout:
    - u8: ObjectType::List
    - u64: LEN
    - u64: N_BYTES
    - ... serialized elements
    // - ... u64 for each serialized element
    */
    let idx = vec.len();
    vec.push(ObjectType::LIST);
    vec.extend_from_slice(&[0u8; 8]); // LEN
    vec.extend_from_slice(&[0u8; 8]); // N_BYTES
    ListStart { idx }
}

pub fn end_list(vec: &mut Vec<u8>, list_start: &ListStart) {
    let start_idx = list_start.idx;
    let end_idx = vec.len();

    let mut len: usize = 0;
    let mut n_bytes: usize = 1 + 8 + 8;
    // let first_elem_idx = start_idx + n_bytes;
    loop {
        let idx = start_idx + n_bytes;
        if idx >= end_idx {
            assert_eq!(idx, end_idx);
            break;
        }
        internal_append_u64(vec, n_bytes as u64); // add index
        let elem_n_bytes = get_obj_len_bytes(vec, idx);
        len += 1;
        n_bytes += elem_n_bytes;
    }
    n_bytes += len * 8; // add bytes for indexes
    internal_write_u64(vec, start_idx + 1, len as u64);
    internal_write_u64(vec, start_idx + 1 + 8, n_bytes as u64);
}

pub fn get_list_len(vec: &Vec<u8>, idx: usize) -> usize {
    assert_eq!(vec[idx], ObjectType::LIST);
    internal_read_u64(vec, idx + 1) as usize
}

pub fn get_list_elem_idx(vec: &Vec<u8>, list_idx: usize, elem_idx: usize) -> usize {
    assert_eq!(vec[list_idx], ObjectType::LIST);
    let len = internal_read_u64(vec, list_idx + 1) as usize;
    let n_bytes = internal_read_u64(vec, list_idx + 1 + 8) as usize;
    list_idx + internal_read_u64(vec, list_idx + n_bytes - 8 * (len - elem_idx)) as usize
}

fn get_obj_len_bytes(vec: &Vec<u8>, idx: usize) -> usize {
    match vec[idx] {
        ObjectType::INT_U64 => 1 + 8,
        ObjectType::INT_I64 => 1 + 8,
        ObjectType::STRING => 1 + 8 + (internal_read_u64(vec, idx + 1) as usize),
        ObjectType::LIST => internal_read_u64(vec, idx + 1 + 8) as usize,
        ObjectType::OBJECT => internal_read_u64(vec, idx + 1 + 8) as usize,
        _ => unimplemented!(),
    }
}

pub fn obj_cmp(vec: &Vec<u8>, left: usize, right: usize) -> std::cmp::Ordering {
    let resp = vec[left].cmp(&vec[right]);
    if resp != std::cmp::Ordering::Equal {
        return resp;
    }

    match vec[left] {
        ObjectType::INT_I64 => read_i64(vec, left).cmp(&read_i64(vec, right)),
        _ => unimplemented!(),
    }
}

pub struct ObjectStart {
    idx: usize,
}

pub fn start_object(vec: &mut Vec<u8>) -> ObjectStart {
    /*
    Objects have the following layout:
    - u8: ObjectType::OBJECT
    - u64: LEN
    - u64: N_BYTES
    - ... serialized key,value pairs
    - ... sorted key indexes for binary search
    */
    let idx = vec.len();
    vec.push(ObjectType::OBJECT);
    vec.extend_from_slice(&[0u8; 8]); // LEN
    vec.extend_from_slice(&[0u8; 8]); // N_BYTES
    ObjectStart { idx }
}

pub fn end_object(vec: &mut Vec<u8>, obj_start: &ObjectStart, sort_buf: &mut Vec<usize>) {
    sort_buf.clear();

    let start_idx = obj_start.idx;
    let mut end_idx = vec.len();

    let mut len: usize = 0;
    let mut n_bytes: usize = 1 + 8 + 8;
    loop {
        let idx = start_idx + n_bytes;
        if idx >= end_idx {
            assert_eq!(idx, end_idx);
            break;
        }
        sort_buf.push(n_bytes);
        let key_n_bytes = get_obj_len_bytes(vec, idx);
        let val_n_bytes = get_obj_len_bytes(vec, idx + key_n_bytes);
        len += 1;
        n_bytes += key_n_bytes + val_n_bytes;
    }

    // Now that we have all the key indexes, sort keys
    sort_buf.sort_unstable_by(|a, b| {
        let resp = obj_cmp(vec, start_idx + *a, start_idx + *b);
        if resp != std::cmp::Ordering::Equal {
            return resp;
        }

        // For overlaps, sort last key earlier in list to keep most recent value
        if a > b {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    // and move duplicates to end of list
    let mut cursor0 = 0;
    let mut cursor1 = 1;
    while cursor1 < sort_buf.len() {
        if obj_cmp(
            vec,
            start_idx + sort_buf[cursor0],
            start_idx + sort_buf[cursor1],
        ) != std::cmp::Ordering::Equal
        {
            cursor0 += 1;
            sort_buf.swap(cursor0, cursor1);
        }
        cursor1 += 1;
    }
    cursor0 += 1;

    // sort keys to delete
    sort_buf[cursor0..cursor1].sort();
    let delete_slice = &sort_buf[cursor0..cursor1];

    if delete_slice.len() > 0 {
        // delete dupe key,value pairs
        let mut delete_cursor0: usize = start_idx + 1 + 8 + 8;
        let mut delete_cursor1 = delete_cursor0;
        let mut delete_idx: usize = 0;
        let mut n_deleted_bytes: usize = 0;
        while delete_cursor1 < end_idx {
            let kv_size = get_obj_len_bytes(vec, delete_cursor1);
            let kv_size = kv_size + get_obj_len_bytes(vec, delete_cursor1 + kv_size);

            if delete_idx < delete_slice.len()
                && delete_cursor1 == start_idx + delete_slice[delete_idx]
            {
                delete_idx += 1;
                n_deleted_bytes += kv_size;
            } else {
                vec.copy_within(delete_cursor1..(delete_cursor1 + kv_size), delete_cursor0);
                delete_cursor0 += kv_size;
            }

            delete_cursor1 += kv_size;
        }

        // delete old bytes from vec
        vec.truncate(vec.len() - n_deleted_bytes);

        // re-create sort_buf
        sort_buf.clear();
        end_idx = vec.len();

        len = 0;
        n_bytes = 1 + 8 + 8;
        loop {
            let idx = start_idx + n_bytes;
            if idx >= end_idx {
                assert_eq!(idx, end_idx);
                break;
            }
            sort_buf.push(n_bytes);
            let key_n_bytes = get_obj_len_bytes(vec, idx);
            let val_n_bytes = get_obj_len_bytes(vec, idx + key_n_bytes);
            len += 1;
            n_bytes += key_n_bytes + val_n_bytes;
        }

        // Now that we have all the key indexes, sort keys
        sort_buf.sort_unstable_by(|a, b| {
            let resp = obj_cmp(vec, start_idx + *a, start_idx + *b);

            // There shouldn't be any equal keys anymore
            assert_ne!(resp, std::cmp::Ordering::Equal);
            resp
        });
    }

    for key_idx in sort_buf.iter() {
        internal_append_u64(vec, *key_idx as u64);
    }
    sort_buf.clear();
    n_bytes += 8 * len;

    internal_write_u64(vec, start_idx + 1, len as u64);
    internal_write_u64(vec, start_idx + 1 + 8, n_bytes as u64);
}

pub fn get_object_len(vec: &Vec<u8>, idx: usize) -> usize {
    assert_eq!(vec[idx], ObjectType::OBJECT);
    internal_read_u64(vec, idx + 1) as usize
}

pub fn get_object_key_idx(vec: &Vec<u8>, obj_idx: usize, key_idx: usize) -> Option<usize> {
    assert_eq!(vec[obj_idx], ObjectType::OBJECT);
    let len = internal_read_u64(vec, obj_idx + 1) as usize;
    let n_bytes = internal_read_u64(vec, obj_idx + 1 + 8) as usize;

    // binary search for
    let mut lo = 0;
    let mut hi = len;
    let index_start = obj_idx + n_bytes - 8 * len;
    while lo < hi {
        let mid = lo + ((hi - lo) / 2);
        let mid_obj_idx = obj_idx + internal_read_u64(vec, index_start + 8 * mid) as usize;
        let cmp = obj_cmp(vec, mid_obj_idx, key_idx);
        match cmp {
            std::cmp::Ordering::Equal => return Some(mid_obj_idx),
            std::cmp::Ordering::Less => {
                lo = mid + 1;
            }
            std::cmp::Ordering::Greater => {
                hi = mid;
            }
        }
    }

    None
}

pub fn get_object_value_idx(vec: &Vec<u8>, obj_idx: usize, key_idx: usize) -> Option<usize> {
    match get_object_key_idx(vec, obj_idx, key_idx) {
        None => None,
        Some(key_idx) => Some(key_idx + get_obj_len_bytes(vec, key_idx)),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_i64() {
        let mut vec = Vec::<u8>::new();
        write_i64(&mut vec, 123456);
        assert_eq!(read_i64(&vec, 0), 123456);
        assert_eq!(get_obj_len_bytes(&vec, 0), 9);
    }

    #[test]
    fn test_basic_u64() {
        let mut vec = Vec::<u8>::new();
        write_u64(&mut vec, 123456);
        assert_eq!(read_u64(&vec, 0), 123456);
    }

    #[test]
    fn test_string() {
        let mut vec = Vec::<u8>::new();
        write_string(&mut vec, "stuff and things");
        assert_eq!(read_string(&vec, 0), "stuff and things");
    }

    #[test]
    fn test_list() {
        let mut vec = Vec::<u8>::new();
        write_i64(&mut vec, -1);
        let list_idx = vec.len();
        let lst = start_list(&mut vec);
        write_i64(&mut vec, 123);
        write_string(&mut vec, "stuff");
        write_i64(&mut vec, 321);
        end_list(&mut vec, &lst);

        assert_eq!(get_list_len(&vec, list_idx), 3);
        assert_eq!(read_i64(&vec, get_list_elem_idx(&vec, list_idx, 0)), 123);
        assert_eq!(
            read_string(&vec, get_list_elem_idx(&vec, list_idx, 1)),
            "stuff"
        );
        assert_eq!(read_i64(&vec, get_list_elem_idx(&vec, list_idx, 2)), 321);
    }

    #[test]
    fn test_obj() {
        let mut vec = Vec::<u8>::new();
        let mut sort_buf = Vec::<usize>::new();
        write_i64(&mut vec, -1);
        let obj_idx = vec.len();
        let obj = start_object(&mut vec);
        write_i64(&mut vec, 20);
        write_i64(&mut vec, 22);

        write_i64(&mut vec, 10);
        write_i64(&mut vec, 11);

        end_object(&mut vec, &obj, &mut sort_buf);

        assert_eq!(get_object_len(&vec, obj_idx), 2);

        let k0 = vec.len();
        write_i64(&mut vec, 20);
        assert_eq!(
            read_i64(&vec, get_object_value_idx(&vec, obj_idx, k0).unwrap()),
            22
        );

        let k1 = vec.len();
        write_i64(&mut vec, 10);
        assert_eq!(
            read_i64(&vec, get_object_value_idx(&vec, obj_idx, k1).unwrap()),
            11
        );

        let k2 = vec.len();
        write_i64(&mut vec, 999);
        assert_eq!(get_object_value_idx(&vec, obj_idx, k2), None);
    }

    #[test]
    fn test_obj_dupe_vals() {
        let mut vec = Vec::<u8>::new();
        let mut sort_buf = Vec::<usize>::new();
        write_i64(&mut vec, -1);
        let obj_idx = vec.len();
        let obj = start_object(&mut vec);
        write_i64(&mut vec, 20);
        write_i64(&mut vec, 22);

        write_i64(&mut vec, 10);
        write_i64(&mut vec, 11);

        write_i64(&mut vec, 20);
        write_i64(&mut vec, 222);

        write_i64(&mut vec, 40);
        write_i64(&mut vec, 44);

        end_object(&mut vec, &obj, &mut sort_buf);

        assert_eq!(get_object_len(&vec, obj_idx), 3);

        let k0 = vec.len();
        write_i64(&mut vec, 20);
        assert_eq!(
            read_i64(&vec, get_object_value_idx(&vec, obj_idx, k0).unwrap()),
            222
        );

        let k1 = vec.len();
        write_i64(&mut vec, 10);
        assert_eq!(
            read_i64(&vec, get_object_value_idx(&vec, obj_idx, k1).unwrap()),
            11
        );

        let k2 = vec.len();
        write_i64(&mut vec, 40);
        assert_eq!(
            read_i64(&vec, get_object_value_idx(&vec, obj_idx, k2).unwrap()),
            44
        );
    }
}
