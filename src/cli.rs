use crate::output::Format;
use crate::{exit, print_language_list, Sort};
use clap::{crate_name, crate_version, value_t_or_exit, App, AppSettings, Arg, SubCommand};
use glob::Pattern;
use std::path::{Path, PathBuf};

pub fn parse() -> Options {
    let app = App::new(crate_name!())
        .version(crate_version!())
        .global_setting(AppSettings::ColoredHelp)
        .setting(AppSettings::VersionlessSubcommands)
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
        print_language_list();
        std::process::exit(0)
    }

    let dir = app.value_of("directory").unwrap_or(".");
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
        .map(|values| values.map(|s| s.to_string()).collect::<Vec<String>>());

    Options {
        work_dir,
        print_error,
        exclude,
        include,
        format,
        sort,
        extension,
    }
}

pub struct Options {
    pub work_dir: PathBuf,
    pub print_error: bool,
    pub exclude: Option<Vec<Pattern>>,
    pub include: Option<Vec<Pattern>>,
    pub format: Format,
    pub sort: Sort,
    pub extension: Option<Vec<String>>,
}

// Translate to the same path
// ./src src => ./src ./src
// /src  src => /src   /src
// src   src => src    src
fn force_to_glob(path: &Path, values: Vec<&str>) -> Vec<Pattern> {
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
