use super::lexer::Position;

pub fn check_error(result: Result<(), (String, Position)>) {
    if let Err(e) = result {
        println!("\x1b[1;31mError:\x1b[0m {} (line {}, char {})", e.0, e.1.line, e.1.character);
        std::process::exit(1);
    }
}

