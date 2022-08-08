#![macro_use]

macro_rules! name_range{() => {'a'..='z' | 'A'..='Z' | '-' | '_'}}
macro_rules! format_error {
    ($line:expr,$char:expr,$message:expr,$( $format:expr ),*) => {
        println!("{} {} (line {}, char {})", format!("\x1b[1;31mError:"), format!($message $(,$format)*), $line, $char);
        std::process::exit(1);
    }
}

macro_rules! error {
    ($line:expr,$char:expr,$message:expr) => {
        println!("{} {} (line {}, char {})", format!("\x1b[1;31mError:\x1b[0m"), $message, $line, $char);
        std::process::exit(1);
    }
}
