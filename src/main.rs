#[macro_use]
extern crate lazy_static;

mod config;
mod output;
mod parse;

use ace::App;
use bright::Colorful;
use config::{Config, Language};
use crossbeam_deque::{Stealer, Worker};
use glob::Pattern;
use output::{Format, Output};
use parse::{parser, Data, Value};
use std::convert::TryFrom;
use std::{path::PathBuf, thread};
use walkdir::WalkDir;

macro_rules! exit {
    ($($arg:tt)*) => {
       {
            eprint!("{} ", "error:".red().bold());
            eprintln!($($arg)*);
            std::process::exit(1)
       }
    };
}

macro_rules! err {
    ($kind: expr, $path: expr) => {
        eprintln!("{} {:?} {:?}", "error:".yellow(), $kind, $path);
    };
}

lazy_static! {
    static ref CONFIG: Config = Config::new();
}

fn main() {
    let app = App::new()
        .name(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .cmd("ls", "Print a list of supported languages")
        .cmd("version", "Print version information")
        .cmd("help", "Print help information")
        .opt("-ext", "Parse file with specified extension")
        .opt("-e", "Exclude files using 'glob' matching")
        .opt("-i", "Include files using 'glob' matching")
        .opt("-o", "Set output format")
        .opt("-s", "Set sort by");

    // Default working directory
    let mut work_dir = PathBuf::from(".");

    if let Some(cmd) = app.command() {
        match cmd.as_str() {
            "help" => return app.print_help(),
            "ls" => return print_language_list(&CONFIG.data),
            "version" => return app.print_version(),
            _ => {
                // Specifying new working directory
                work_dir = PathBuf::from(cmd);
            }
        }
    }

    let extension = app.value("-ext").map(|values| {
        if values.is_empty() {
            exit!("-ext value: [extension] [extension] ..")
        } else {
            values.iter().map(|val| val.as_str()).collect::<Vec<&str>>()
        }
    });

    let exclude = app.value("-e").map(|values| {
        if values.is_empty() {
            exit!("-e value: [glob] [glob] ..")
        } else {
            force_to_glob(&work_dir, values)
        }
    });

    let include = app.value("-i").map(|values| {
        if values.is_empty() {
            exit!("-i value: [glob] [glob] ..")
        } else {
            force_to_glob(&work_dir, values)
        }
    });

    let format = app
        .value("-o")
        .map(|values| {
            if values.is_empty() {
                exit!("-o value: ascii | html | markdown")
            } else {
                Format::try_from(values[0].as_str())
                    .unwrap_or_else(|_| exit!("-o value: ascii | html | markdown"))
            }
        })
        .unwrap_or_default();

    let sort = app
        .value("-s")
        .map(|values| {
            if values.is_empty() {
                exit!("-s value: language | code | comment | blank | file | size")
            } else {
                Sort::try_from(values[0].as_str()).unwrap_or_else(|_| {
                    exit!("-s value: language | code | comment | blank | file | size")
                })
            }
        })
        .unwrap_or_default();

    // Init
    let worker = Worker::new_fifo();
    let cpus = num_cpus::get();
    let mut threads = Vec::with_capacity(cpus);

    // Created thread
    for _ in 0..cpus {
        let fifo = Queue(worker.stealer().clone());
        threads.push(thread::spawn(|| fifo.start()));
    }

    // Get all files
    let tree = WalkDir::new(work_dir).into_iter().filter_map(|item| {
        let entry = match item {
            Ok(entry) => entry,
            Err(error) => {
                if let (Some(err), Some(path)) = (error.io_error(), error.path()) {
                    err!(err.kind(), path);
                }
                return None;
            }
        };

        let path = entry.path();

        // Exclude files
        if let Some(exclude) = &exclude {
            for matcher in exclude {
                if matcher.matches_path(path) {
                    return None;
                }
            }
        }

        // Include files
        if let Some(include) = &include {
            let any = include.iter().any(|m| m.matches_path(path));
            if !any {
                return None;
            }
        }

        // File with the specified extension
        let ext = match path.extension() {
            Some(s) => match s.to_str() {
                Some(ext) => ext,
                None => return None,
            },
            None => return None,
        };

        // This extension is not included in config
        if let Some(extension) = &extension {
            if !extension.contains(&ext) {
                return None;
            }
        }

        // Get file path and configuration
        CONFIG
            .get(ext)
            .map(|config| (entry.path().to_path_buf(), config))
    });

    for (path, config) in tree {
        worker.push(Work::Parse(path, config));
    }

    for _ in 0..cpus {
        worker.push(Work::Quit);
    }

    // Merge data
    let mut result = Vec::new();

    for thread in threads {
        let data = thread.join().unwrap_or_else(|err| {
            exit!("Thread exits abnormally\n{:#?}", err);
        });

        for d in data {
            let position = result
                .iter()
                .position(|item: &Detail| item.language == d.language);

            match position {
                Some(i) => {
                    result[i].comment += d.comment;
                    result[i].blank += d.blank;
                    result[i].code += d.code;
                    result[i].size += d.size;
                    result[i].file += 1;
                }
                None => {
                    result.push(Detail {
                        language: d.language,
                        comment: d.comment,
                        blank: d.blank,
                        code: d.code,
                        size: d.size,
                        file: 1,
                    });
                }
            }
        }
    }

    let data = match sort {
        Sort::Language => bubble_sort(result, |a, b| position(a.language) > position(b.language)),
        Sort::Code => bubble_sort(result, |a, b| a.code > b.code),
        Sort::Comment => bubble_sort(result, |a, b| a.comment > b.comment),
        Sort::Blank => bubble_sort(result, |a, b| a.blank > b.blank),
        Sort::File => bubble_sort(result, |a, b| a.file > b.file),
        Sort::Size => bubble_sort(result, |a, b| a.size > b.size),
    };

    Output::new(data).print(format);
}

fn print_language_list(data: &[Language]) {
    let n = data
        .iter()
        .map(|language| language.name.len())
        .fold(0, |a, b| a.max(b));

    for language in data {
        let ext = language
            .extension
            .iter()
            .map(|e| format!(".{}", e))
            .collect::<Vec<String>>()
            .join(" ");
        println!("{:name$}    {}", language.name, ext, name = n);
    }
}

// Translate to the same path
// ./src src => ./src ./src
// /src  src => /src   /src
// src   src => src    src
fn force_to_glob(path: &PathBuf, values: Vec<&String>) -> Vec<Pattern> {
    values
        .iter()
        .map(|s| {
            if path.starts_with(".") && !s.starts_with("./") {
                format!("./{}", s)
            } else if path.starts_with("/") && !s.starts_with('/') {
                format!("/{}", s)
            } else {
                (*s).to_string()
            }
        })
        .map(|s| {
            Pattern::new(s.as_str())
                .unwrap_or_else(|err| exit!("Cannot parse '{}' to glob matcher\n{:#?}", s, err))
        })
        .collect::<Vec<Pattern>>()
}

fn bubble_sort<T>(mut vec: Vec<T>, call: fn(&T, &T) -> bool) -> Vec<T> {
    for x in 0..vec.len() {
        for y in x..vec.len() {
            if call(&vec[x], &vec[y]) {
                vec.swap(x, y);
            }
        }
    }
    vec
}

fn position(s: &str) -> usize {
    const LETTER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let first = s.chars().next().unwrap_or_default();
    LETTER.chars().position(|d| d == first).unwrap_or(0)
}

#[derive(Debug, Clone)]
pub struct Detail {
    language: &'static str,
    blank: i32,
    comment: i32,
    code: i32,
    size: u64,
    file: i32,
}

#[derive(Debug)]
enum Sort {
    Language,
    Code,
    Comment,
    Blank,
    File,
    Size,
}

impl TryFrom<&str> for Sort {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "language" => Ok(Sort::Language),
            "code" => Ok(Sort::Code),
            "comment" => Ok(Sort::Comment),
            "blank" => Ok(Sort::Blank),
            "file" => Ok(Sort::File),
            "size" => Ok(Sort::Size),
            _ => Err(()),
        }
    }
}

impl Default for Sort {
    fn default() -> Self {
        Sort::Language
    }
}

enum Work<'a> {
    Parse(PathBuf, &'a Language),
    Quit,
}

struct Queue<'a>(Stealer<Work<'a>>);

impl<'a> Queue<'a> {
    fn start(self) -> Vec<Data> {
        let mut result = Vec::new();

        loop {
            // Receive message
            let work = match self.0.steal().success() {
                Some(work) => work,
                None => continue,
            };

            match work {
                Work::Parse(path, config) => {
                    match parser(path, &config) {
                        Value::Ok(data) => result.push(data),
                        Value::Err(kind, p) => err!(kind, p),
                        Value::Invalid => continue,
                    };
                }
                Work::Quit => break,
            }
        }

        result
    }
}
