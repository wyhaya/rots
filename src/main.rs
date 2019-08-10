mod config;
mod output;

use ace::App;
use config::{Config, Language};
use crossbeam_deque::{Stealer, Worker};
use output::{Output, Print};
use regex::Regex;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::thread;

macro_rules! exit {
    ($($arg:tt)*) => {
       {
            eprint!("\x1b[91m{}: \x1b[0m", "error");
            eprintln!($($arg)*);
            std::process::exit(1)
       }
    };
}

macro_rules! warn {
    ($kind: expr, $path: expr) => {
        eprint!("\x1b[93m{}: \x1b[0m", "error");
        eprintln!("{:?} {:?}", $kind, $path);
    };
}

static mut CONFIG: Option<Config> = None;

fn main() {
    unsafe {
        CONFIG = Some(config::new());
    }

    let app = App::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .cmd("help", "Print help information")
        .cmd("list", "Print a list of supported languages")
        .cmd("version", "Print version information")
        .opt("-e", "Parse the specified extension")
        .opt("-i", "Ignore files using 'Rust regex'")
        .opt("-o", "Set output format")
        .opt("-s", "Set sort by");

    let mut path = PathBuf::from(".");

    if let Some(cmd) = app.command() {
        match cmd.as_str() {
            "help" => {
                app.help();
                return;
            }
            "list" => {
                print_support_list();
                return;
            }
            "version" => {
                app.version();
                return;
            }
            _ => {
                path = PathBuf::from(cmd);
            }
        }
    }

    let extension = match app.value("-e") {
        Some(values) => {
            if values.len() == 0 {
                exit!("-e value: [extension] [extension] ..");
            }
            values
        }
        None => vec![],
    };

    let ignore = match app.value("-i") {
        Some(values) => {
            if values.len() == 0 {
                exit!("-i value: [regex] [regex] ..");
            }

            let val = values
                .iter()
                .map(|val| format!("({})", &val))
                .collect::<Vec<String>>()
                .join("|");

            match Regex::new(&val) {
                Ok(reg) => Some(reg),
                Err(err) => exit!("{:?}", err),
            }
        }
        None => None,
    };

    let output_err = || -> ! {
        exit!("-o value: ascii | html | markdown");
    };
    let output = match app.value("-o") {
        Some(values) => {
            if values.len() == 0 {
                output_err();
            }

            match values[0].to_lowercase().as_str() {
                "ascii" => Output::ASCII,
                "html" => Output::HTML,
                "markdown" => Output::Markdown,
                _ => output_err(),
            }
        }
        None => Output::ASCII,
    };

    let sort_err = || -> ! {
        exit!("-s value: language | code | comment | blank | file | size");
    };
    let sort = match app.value("-s") {
        Some(values) => {
            if values.len() == 0 {
                sort_err();
            }

            match values[0].to_lowercase().as_str() {
                "language" => Sort::Language,
                "code" => Sort::Code,
                "comment" => Sort::Comment,
                "blank" => Sort::Blank,
                "file" => Sort::File,
                "size" => Sort::Size,
                _ => sort_err(),
            }
        }
        None => Sort::Language,
    };

    let work = Worker::new_fifo();
    let stealer = work.stealer();
    let mut threads = vec![];

    for _ in 0..num_cpus::get() {
        let fifo = Queue(stealer.clone());
        threads.push(thread::spawn(|| fifo.run()));
    }

    // todo
    tree(path, &extension, &ignore, &work);

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
        Sort::Code => bubble_sort(result, |a, b| a.code > b.code),
        Sort::Comment => bubble_sort(result, |a, b| a.comment > b.comment),
        Sort::Blank => bubble_sort(result, |a, b| a.blank > b.blank),
        Sort::File => bubble_sort(result, |a, b| a.file > b.file),
        Sort::Size => bubble_sort(result, |a, b| a.size > b.size),
        _ => bubble_sort(result, |a, b| position(a.language) > position(b.language)),
    };

    match output {
        Output::ASCII => Print(data).ascii(),
        Output::HTML => Print(data).html(),
        Output::Markdown => Print(data).markdown(),
    };
}

fn print_support_list() {
    let config = unsafe { CONFIG.as_ref() }.unwrap();

    let mut max = 0;
    for item in &config.data {
        if item.name.len() > max {
            max = item.name.len();
        }
    }

    for item in &config.data {
        let ext = item
            .extension
            .iter()
            .map(|e| format!(".{}", e))
            .collect::<Vec<String>>()
            .join(" ");
        println!("{:name$}    {}", item.name, ext, name = max);
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

const LETTER: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
fn position(s: &str) -> usize {
    if let Some(c) = s.chars().next() {
        let index = LETTER.chars().position(|d| d == c);
        return match index {
            Some(i) => i,
            None => 0,
        };
    }
    0
}

fn tree(dir: PathBuf, ext: &Vec<&String>, ignore: &Option<Regex>, work: &Worker<Work>) {
    let read_dir = match fs::read_dir(&dir) {
        Ok(dir) => dir,
        Err(err) => {
            warn!(err.kind(), &dir);
            return;
        }
    };

    for file in read_dir {
        let file = match file {
            Ok(file) => file,
            Err(err) => {
                warn!(err.kind(), &dir);
                continue;
            }
        };

        let meta = match file.metadata() {
            Ok(meta) => meta,
            Err(err) => {
                warn!(err.kind(), &dir);
                continue;
            }
        };

        if let Some(ignore) = ignore {
            match file.file_name().to_str() {
                Some(name) => {
                    if ignore.is_match(name) {
                        continue;
                    }
                }
                None => continue,
            };
        }
        let path = file.path();

        if meta.is_dir() {
            tree(path, &ext, &ignore, &work);
            continue;
        }

        let extension = match path.extension() {
            Some(d) => match d.to_str() {
                Some(d) => d,
                None => continue,
            },
            None => continue,
        };

        if ext.len() != 0 {
            if !ext.iter().any(|item| item == &extension) {
                continue;
            }
        }

        if let Some(config) = unsafe { CONFIG.as_ref() }.unwrap().get(extension) {
            work.push(Work::File(path, meta.len(), config));
        }
    }
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
                None => {
                    continue;
                }
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
                Work::Quit => {
                    break;
                }
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
                if line.starts_with(start) {
                    in_comment = match in_comment {
                        Some(_) => None,
                        None => Some((start, end)),
                    };
                }
                if line.ends_with(end) {
                    in_comment = None;
                    comment += 1;
                    continue 'line;
                }
                if let Some(_) = in_comment {
                    comment += 1;
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
