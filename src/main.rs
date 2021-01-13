mod config;
mod output;
mod parse;

use bright::Colorful;
use clap::{value_t_or_exit, App, AppSettings, Arg, SubCommand};
use config::{Language, CONFIG};
use crossbeam_deque::{Stealer, Worker};
use glob::Pattern;
use output::{Format, Output};
use parse::{parser, Data, Value};
use std::str::FromStr;
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

fn main() {
    let app = App::new(env!("CARGO_PKG_NAME"))
        .global_setting(AppSettings::ColoredHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(SubCommand::with_name("ls").about("Print a list of supported languages"))
        .arg(Arg::with_name("directory").help("Calculate the specified directory"))
        .arg(
            Arg::with_name("error")
                .long("error")
                .help("Show error message"),
        )
        .arg(
            Arg::with_name("exclude")
                .short("e")
                .long("exclude")
                .value_name("GLOB")
                .multiple(true)
                .help("Exclude files using 'glob' matching"),
        )
        .arg(
            Arg::with_name("include")
                .short("i")
                .long("include")
                .value_name("GLOB")
                .multiple(true)
                .help("Include files using 'glob' matching"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("OUTPUT")
                .possible_values(&["table", "html", "markdown"])
                .default_value("table")
                .max_values(1)
                .hide_default_value(true)
                .help("Specify output format"),
        )
        .arg(
            Arg::with_name("sort")
                .short("s")
                .long("sort")
                .value_name("SORT")
                .possible_values(&["language", "code", "comment", "blank", "file", "size"])
                .default_value("language")
                .max_values(1)
                .hide_default_value(true)
                .help("Specify the column sort by"),
        )
        .arg(
            Arg::with_name("extension")
                .long("extension")
                .multiple(true)
                .value_name("EXTENSION")
                .display_order(1000)
                .help("Parse file with specified extension"),
        )
        .get_matches();

    if app.is_present("ls") {
        return print_language_list();
    }

    let dir = match app.values_of("directory") {
        Some(targets) => targets.collect::<Vec<&str>>()[0],
        None => ".",
    };

    let work_dir = PathBuf::from(dir);

    // Whether the output is wrong
    let print_error = app.is_present("error");

    let exclude = app
        .values_of("exclude")
        .map(|values| force_to_glob(&work_dir, values.collect()));

    let include = app
        .values_of("include")
        .map(|values| force_to_glob(&work_dir, values.collect()));

    let format = value_t_or_exit!(app, "output", Format);

    let sort = value_t_or_exit!(app, "sort", Sort);

    let extension = app
        .values_of("extension")
        .map(|values| values.collect::<Vec<&str>>());

    // Init
    let worker = Worker::new_fifo();
    let cpus = num_cpus::get();
    let mut threads = Vec::with_capacity(cpus);

    // Created thread
    for _ in 0..cpus {
        let fifo = Queue {
            stealer: worker.stealer().clone(),
            print_error,
        };
        threads.push(thread::spawn(|| fifo.start()));
    }

    // Get all files
    let files = WalkDir::new(work_dir).into_iter().filter_map(|item| {
        let entry = match item {
            Ok(entry) => entry,
            Err(error) => {
                if print_error {
                    if let (Some(err), Some(path)) = (error.io_error(), error.path()) {
                        err!(err.kind(), path);
                    }
                }
                return None;
            }
        };

        let path = entry.path();

        // Include files
        if let Some(include) = &include {
            let any = include.iter().any(|m| m.matches_path(path));
            if !any {
                return None;
            }
        }

        // Exclude files
        if let Some(exclude) = &exclude {
            for matcher in exclude {
                if matcher.matches_path(path) {
                    return None;
                }
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

    for (path, config) in files {
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

fn print_language_list() {
    let n = CONFIG
        .all_language()
        .iter()
        .map(|language| language.name.len())
        .fold(0, |a, b| a.max(b));

    for language in CONFIG.all_language() {
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
fn force_to_glob(path: &PathBuf, values: Vec<&str>) -> Vec<Pattern> {
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

#[derive(Debug)]
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

impl FromStr for Sort {
    type Err = ();
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
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

struct Queue<'a> {
    stealer: Stealer<Work<'a>>,
    print_error: bool,
}

impl<'a> Queue<'a> {
    fn start(self) -> Vec<Data> {
        let mut result = Vec::new();

        loop {
            // Receive message
            let work = match self.stealer.steal().success() {
                Some(work) => work,
                None => continue,
            };

            match work {
                Work::Parse(path, config) => {
                    match parser(path, &config) {
                        Value::Ok(data) => result.push(data),
                        Value::Err(kind, p) => {
                            if self.print_error {
                                err!(kind, p)
                            }
                        }
                        Value::Invalid => continue,
                    };
                }
                Work::Quit => break,
            }
        }

        result
    }
}
