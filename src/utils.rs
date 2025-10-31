#[macro_export]
macro_rules! print_flush {
    ($($arg:tt)*) => {{
        print!($($arg)*);
        $crate::io::stdout().flush().unwrap();
    }};
}

#[macro_export]
macro_rules! print_discard {
    () => {{
        print!("\x1B[2K\r");
    }};
}
