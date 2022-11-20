use std::ops::Range;
use std::fs;

const LIB_PATH: &str =  "/usr/share/gtk-ui/";

pub fn check_error(result: Result<(), (String, Range<usize>)>, file: &String, file_content: &String) {
    if let Err(err) = result {
        if err.1.start > err.1.end {
            println!("\x1b[1;31mError:\x1b[0m {} (in {})", err.0, file);
        } else {
            match get_position_from_char_index(err.1.start, file_content) {
                Ok((line, char)) => {
                    println!("\x1b[1;31mError:\x1b[0m {} (line {}, char {}, in {})", err.0, line, char, file);
                },
                Err(message) => println!("\x1b[1;31mError:\x1b[0m {}", message)
            }
        }
        std::process::exit(1);
    }
}

pub fn get_position_from_char_index(char_index: usize, file_content: &String) -> Result<(usize, usize), &str> {
    // Quick sanity check
    if char_index >= file_content.len() {
        Err("character index bigger than file content (something horrible must have gone wrong)")
    } else {
        let mut line_count = 1;
        let mut char_count = 1;
        for i in 0..char_index {
            char_count += 1;
            if file_content.chars().nth(i).unwrap() == '\n' {
                line_count += 1;
                char_count = 1;
            }
        }
        Ok((line_count, char_count))
    }
}

pub fn get_include_path(path: &String) -> Option<String> {
    let lib_file_path = format!("{LIB_PATH}/{path}.gui");
    if fs::metadata(&lib_file_path).is_ok() {
        Some(lib_file_path)
    } else if fs::metadata(&path).is_ok() {
        Some(path.clone())
    } else {
        None
    }
}
