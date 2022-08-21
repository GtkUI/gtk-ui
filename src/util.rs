use super::lexer::Position;

pub fn check_error(result: Result<(), (String, Position)>) {
    if let Err(e) = result {
        if e.1.line == -1 {
            println!("\x1b[1;31mError:\x1b[0m {}", e.0);
        } else {
            println!("\x1b[1;31mError:\x1b[0m {} (line {}, char {})", e.0, e.1.line, e.1.character);
        }
        std::process::exit(1);
    }
}

