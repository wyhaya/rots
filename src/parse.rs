use crate::config::Language;
use std::{fs, io::ErrorKind, path::PathBuf};

#[derive(Debug)]
pub enum Value {
    Ok(Data),
    Err(ErrorKind, PathBuf),
    Invalid,
}

#[derive(Debug)]
pub struct Data {
    pub language: &'static str,
    pub blank: i32,
    pub comment: i32,
    pub code: i32,
    pub size: u64,
}

pub fn parser(path: PathBuf, config: &Language) -> Value {
    let size = match path.metadata() {
        Ok(meta) => {
            if !meta.is_file() {
                return Value::Invalid;
            }
            meta.len()
        }
        Err(err) => return Value::Err(err.kind(), path),
    };

    let content = match fs::read_to_string(&path) {
        Ok(data) => data,
        Err(err) => return Value::Err(err.kind(), path),
    };

    let mut blank = 0;
    let mut comment = 0;
    let mut code = 0;
    let mut in_comment = None;

    'line: for line in content.lines() {
        let line = line.trim();

        // Matching blank line
        if line.is_empty() {
            blank += 1;
            continue 'line;
        }

        // Match multiple lines of comments
        for (start, end) in config.multi {
            if let Some(d) = in_comment {
                if d != (start, end) {
                    continue;
                }
            }

            // Multi-line comments may also end in a single line
            let mut same_line = false;

            if line.starts_with(start) {
                in_comment = match in_comment {
                    Some(_) => {
                        comment += 1;
                        in_comment = None;
                        continue 'line;
                    }
                    None => {
                        same_line = true;
                        Some((start, end))
                    }
                };
            }

            // This line is in the comment
            if in_comment.is_some() {
                comment += 1;
                if line.ends_with(end) {
                    if same_line {
                        if line.len() >= (start.len() + end.len()) {
                            in_comment = None;
                        }
                    } else {
                        in_comment = None;
                    }
                }
                continue 'line;
            }
        }

        //  Match single line comments
        for single in config.single {
            if line.starts_with(single) {
                comment += 1;
                continue 'line;
            }
        }

        code += 1;
    }

    Value::Ok(Data {
        language: config.name,
        blank,
        comment,
        code,
        size,
    })
}
