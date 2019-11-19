mod config;
mod output;

use ace::App;
use bright::Colorful;
use config::{Config, Language};
use crossbeam_deque::{Stealer, Worker};
use output::{Output, Print};
use regex::Regex;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::thread;
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

macro_rules! warn {
    ($kind: expr, $path: expr) => {
        eprintln!("{} {:?} {:?}", "error:".yellow(), $kind, $path);
    };
}

macro_rules! empty {
    ($arr: expr, $exec: expr) => {
        if $arr.is_empty() {
            $exec
        }
    };
}

static mut CONFIG: Option<Config> = None;

fn main() {
    let app = App::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .cmd("help", "Print help information")
        .cmd("ls", "Print a list of supported languages")
        .cmd("version", "Print version information")
        .opt("-ext", "Parse the specified extension")
        .opt("-e", "Exclude `Rust regex` specified files")
        .opt("-i", "Include `Rust regex` specified files")
        .opt("-o", "Set output format")
        .opt("-s", "Set sort by");

    let mut path = PathBuf::from(".");

    if let Some(cmd) = app.command() {
        match cmd.as_str() {
            "help" => return app.help(),
            "ls" => return print_language_list(),
            "version" => return app.version(),
            _ => {
                path = PathBuf::from(cmd);
            }
        }
    }

    let extension = app
        .value("-ext")
        .map(|values| {
            empty!(values, exit!("-ext value: [extension] [extension] .."));
            values
        })
        .unwrap_or_default();

    let exclude = app
        .value("-e")
        .map(|values| {
            empty!(values, exit!("-e value: [regex] [regex] .."));
            Some(vec_to_regex(values))
        })
        .unwrap_or_default();

    let include = app
        .value("-i")
        .map(|values| {
            empty!(values, exit!("-i value: [regex] [regex] .."));
            Some(vec_to_regex(values))
        })
        .unwrap_or_default();

    let output = app
        .value("-o")
        .map(|values| {
            empty!(values, exit!("-o value: ascii | html | markdown"));
            match values[0].to_lowercase().as_str() {
                "ascii" => Output::ASCII,
                "html" => Output::HTML,
                "markdown" => Output::Markdown,
                _ => exit!("-o value: ascii | html | markdown"),
            }
        })
        .unwrap_or_default();

    let sort = app
        .value("-s")
        .map(|values| {
            empty!(
                values,
                exit!("-s value: language | code | comment | blank | file | size")
            );

            match values[0].to_lowercase().as_str() {
                "language" => Sort::Language,
                "code" => Sort::Code,
                "comment" => Sort::Comment,
                "blank" => Sort::Blank,
                "file" => Sort::File,
                "size" => Sort::Size,
                _ => exit!("-s value: language | code | comment | blank | file | size"),
            }
        })
        .unwrap_or_default();

    unsafe {
        CONFIG = Some(config::new());
    }
    let work = Worker::new_fifo();
    let stealer = work.stealer();
    let cpus = num_cpus::get();
    let mut threads = Vec::with_capacity(cpus);

    for _ in 0..cpus {
        let fifo = Queue(stealer.clone());
        threads.push(thread::spawn(|| fifo.run()));
    }

    let tree = WalkDir::new(path).into_iter().filter_map(|item| {
        let entry = match item {
            Ok(entry) => entry,
            Err(error) => {
                if let (Some(err), Some(path)) = (error.io_error(), error.path()) {
                    warn!(err.kind(), path);
                }
                return None;
            }
        };

        let path = entry.path();

        if exclude.is_some() || include.is_some() {
            let filename = match path.file_name() {
                Some(s) => match s.to_str() {
                    Some(name) => name,
                    None => return None,
                },
                None => return None,
            };

            // exclude files
            if let Some(exclude) = &exclude {
                if exclude.is_match(filename) {
                    return None;
                }
            }

            // include files
            if let Some(include) = &include {
                if !include.is_match(filename) {
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

        if !extension.is_empty() && !extension.iter().any(|item| item == &ext) {
            return None;
        }

        if let Some(config) = unsafe { CONFIG.as_ref() }.unwrap().get(ext) {
            if let Ok(meta) = path.metadata() {
                return Some((entry.path().to_path_buf(), meta.len(), config));
            }
        }
        None
    });

    for (path, len, config) in tree {
        work.push(Work::File(path, len, config));
    }

    for _ in 0..threads.len() {
        work.push(Work::Quit);
    }

    let mut result = vec![];

    for t in threads {
        for d in t.join().unwrap() {
            let find = result
                .iter()
                .position(|item: &Detail| item.language == d.language);

            if let Some(i) = find {
                result[i].comment += d.comment;
                result[i].blank += d.blank;
                result[i].code += d.code;
                result[i].size += d.size;
                result[i].file += 1;
            } else {
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

    let data = match sort {
        Sort::Language => bubble_sort(result, |a, b| position(a.language) > position(b.language)),
        Sort::Code => bubble_sort(result, |a, b| a.code > b.code),
        Sort::Comment => bubble_sort(result, |a, b| a.comment > b.comment),
        Sort::Blank => bubble_sort(result, |a, b| a.blank > b.blank),
        Sort::File => bubble_sort(result, |a, b| a.file > b.file),
        Sort::Size => bubble_sort(result, |a, b| a.size > b.size),
    };

    match output {
        Output::ASCII => Print(data).ascii(),
        Output::HTML => Print(data).html(),
        Output::Markdown => Print(data).markdown(),
    };
}

fn print_language_list() {
    let data = config::new().data;

    let n = data
        .iter()
        .map(|item| item.name.len())
        .fold(0, |a, b| a.max(b));

    for item in &data {
        let ext = item
            .extension
            .iter()
            .map(|e| format!(".{}", e))
            .collect::<Vec<String>>()
            .join(" ");
        println!("{:name$}    {}", item.name, ext, name = n);
    }
}

fn vec_to_regex(values: Vec<&String>) -> Regex {
    let val = values
        .iter()
        .map(|val| format!("({})", &val))
        .collect::<Vec<String>>()
        .join("|");

    match Regex::new(&val) {
        Ok(reg) => reg,
        Err(err) => exit!("{:?}", err),
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

const LETTER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
fn position(s: &str) -> usize {
    if let Some(c) = s.chars().next() {
        let index = LETTER.chars().position(|d| d == c);
        if let Some(i) = index {
            return i;
        }
    }
    0
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

impl Default for Sort {
    fn default() -> Self {
        Sort::Language
    }
}

enum Work<'a> {
    File(PathBuf, u64, &'a Language),
    Quit,
}

struct Queue<'a>(Stealer<Work<'a>>);

impl<'a> Queue<'a> {
    fn run(self) -> Vec<Parse> {
        let mut vec = vec![];
        loop {
            let work = match self.0.steal().success() {
                Some(work) => work,
                None => continue,
            };
            match work {
                Work::File(path, size, config) => {
                    match Parse::new(path, size, &config) {
                        Ok(d) => vec.push(d),
                        Err((kind, p)) => {
                            warn!(kind, p);
                        }
                    };
                }
                Work::Quit => break,
            }
        }
        vec
    }
}

#[derive(Debug, Clone)]
struct Parse {
    language: &'static str,
    blank: i32,
    comment: i32,
    code: i32,
    size: u64,
}

impl Parse {
    fn new(path: PathBuf, size: u64, config: &Language) -> Result<Parse, (ErrorKind, PathBuf)> {
        let content = match fs::read_to_string(&path) {
            Ok(data) => data,
            Err(err) => return Err((err.kind(), path)),
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
            for (start, end) in &config.multi {
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
            for single in &config.single {
                if line.starts_with(single) {
                    comment += 1;
                    continue 'line;
                }
            }

            code += 1;
        }

        Ok(Parse {
            language: config.name,
            blank,
            comment,
            code,
            size,
        })
    }
}
