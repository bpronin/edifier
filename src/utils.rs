use std::fmt::Write;

pub(crate) fn split_into_bytes(value: u16) -> [u8; 2] {
    [(value >> 8) as u8, (value & 0xFF) as u8]
}

pub(crate) fn join_hex<T: AsRef<[u8]>>(data: T, delimiter: &str) -> String {
    let bytes = data.as_ref();
    let mut result = String::with_capacity(bytes.len() * (2 + delimiter.len()));
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            result.push_str(delimiter);
        }
        write!(&mut result, "{:02X}", b).unwrap();
    }
    result
}

pub(crate) fn join_str<E: ToString, T: AsRef<[E]>>(data: T, delimiter: &str) -> String {
    let bytes = data.as_ref();
    let mut result = String::with_capacity(bytes.len() * (2 + delimiter.len()));
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            result.push_str(delimiter);
        }
        result.push_str(b.to_string().as_str());
    }
    result
}
