mod cli;
mod config;
mod output;
mod parse;

use cli::Options;
use config::{Language, CONFIG};
use crossbeam_deque::{Stealer, Worker};
use output::Output;
use parse::{parser, Data, Value};
use std::path::PathBuf;
use walkdir::WalkDir;

#[macro_export]
macro_rules! exit {
    ($($arg:tt)*) => {
       {
            use bright::Colorful;
            eprint!("{} ", "error:".red().bold());
            eprintln!($($arg)*);
            std::process::exit(1)
       }
    };
}

macro_rules! err {
    ($kind: expr, $path: expr) => {{
        use bright::Colorful;
        eprintln!("{} {:?} {:?}", "error:".yellow(), $kind, $path);
    }};
}

fn main() {
    let Options {
        work_dir,
        print_error,
        exclude,
        include,
        format,
        sort,
        extension,
    } = cli::parse();

    let worker = Worker::new_fifo();
    let cpus = num_cpus::get();
    let mut threads = Vec::with_capacity(cpus);

    // Created thread
    for _ in 0..cpus {
        let stealer = worker.stealer().clone();
        threads.push(std::thread::spawn(move || {
            let task = Task {
                stealer,
                print_error,
            };
            task.start()
        }));
    }

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
            if !extension.iter().any(|s| s == ext) {
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

    // Summary of all data
    let mut total = Vec::new();

    for thread in threads {
        let task_data = thread.join().unwrap_or_else(|err| {
            exit!("Thread exits abnormally\n{:#?}", err);
        });

        for data in task_data {
            let find = total
                .iter_mut()
                .find(|item: &&mut Detail| item.language == data.language);

            match find {
                Some(detail) => detail.add(data),
                None => total.push(data.into_detail()),
            }
        }
    }

    let data = match sort {
        Sort::Language => bubble_sort(total, |a, b| position(a.language) > position(b.language)),
        Sort::Code => bubble_sort(total, |a, b| a.code > b.code),
        Sort::Comment => bubble_sort(total, |a, b| a.comment > b.comment),
        Sort::Blank => bubble_sort(total, |a, b| a.blank > b.blank),
        Sort::File => bubble_sort(total, |a, b| a.file > b.file),
        Sort::Size => bubble_sort(total, |a, b| a.size > b.size),
    };

    Output::new(data).print(format);
}

pub fn print_language_list() {
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

impl Detail {
    fn add(&mut self, data: Data) {
        self.comment += data.comment;
        self.blank += data.blank;
        self.code += data.code;
        self.size += data.size;
        self.file += 1;
    }
}

#[derive(Debug)]
pub enum Sort {
    Language,
    Code,
    Comment,
    Blank,
    File,
    Size,
}

impl std::str::FromStr for Sort {
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

struct Task<'a> {
    stealer: Stealer<Work<'a>>,
    print_error: bool,
}

impl<'a> Task<'a> {
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
