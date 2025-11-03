use std::fmt::Write;

#[macro_export]
macro_rules! print_discardable {
    ($($arg:tt)*) => {{
        print!($($arg)*);
        std::io::stdout().flush().unwrap();
    }};
}

#[macro_export]
macro_rules! print_discard {
    () => {{
        print!("\x1B[2K\r");
    }};
}

pub fn join_hex<T: AsRef<[u8]>>(data: T, delimiter: &str) -> String {
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

pub fn join_str<E: ToString, T: AsRef<[E]>>(data: T, delimiter: &str) -> String {
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
