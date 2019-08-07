use crate::Regex;

macro_rules! regex {
    ($reg:expr) => {{
        Some(Regex::new($reg).unwrap())
    }};
    ($start:expr, $end:expr) => {{
        Some((Regex::new($start).unwrap(), Regex::new($end).unwrap()))
    }};
}

// todo
pub fn new() -> Config {
    let mut config = Config::default();

    macro_rules! language {
        ($name: expr, $ext: expr, $single: expr, $multi: expr) => {{
            config.data.push(Language {
                name: $name,
                extension: $ext,
                single: $single,
                multi: $multi,
            });
        }};
    }

    language!("AspNet", vec!["aspx"], None, None);
    language!("C", vec!["c"], None, None);
    language!("CSS", vec!["css", "scss", "sass", "less"], None, None);
    language!("Cpp", vec!["cpp"], None, None);
    language!("CoffeeScript", vec!["coffee"], None, None);
    language!("D", vec!["d"], None, None);
    language!("Dart", vec!["dart"], None, None);
    language!("Go", vec!["go"], None, None);
    language!(
        "HTML",
        vec!["htm", "html"],
        None,
        regex!(r#"<!--"#, r#"-->"#)
    );
    language!("Haskell", vec!["hs"], None, None);
    language!(
        "JavaScript",
        vec!["js", "mjs"],
        regex!(r#"^\s*//"#),
        regex!(r#"/\*"#, r#"\*/"#)
    );
    language!("JavaScript JSX", vec!["jsx"], None, None);
    language!("JSON", vec!["json"], None, None);
    language!("Julia", vec!["jl"], None, None);
    language!("Java", vec!["java"], None, None);
    language!("LLVM", vec!["ll"], None, None);
    language!("Lua", vec!["lua"], None, None);
    language!("Markdown", vec!["md", "markdown"], None, None);
    language!("Nim", vec!["nim"], None, None);
    language!("ObjectiveC", vec!["m"], None, None);
    language!("ObjectiveCpp", vec!["mm"], None, None);
    language!(
        "PHP",
        vec!["php"],
        regex!(r#"^\s*//"#),
        regex!(r#"/\*"#, r#"\*/"#)
    );
    language!("Python", vec!["py"], regex!(r#"#"#), None);
    language!("Perl", vec!["pl", "pm"], None, None);
    language!("R", vec!["r"], regex!(r#"#"#), None);
    language!(
        "Rust",
        vec!["rs"],
        regex!(r#"^\s*//"#),
        regex!(r#"/\*"#, r#"\*/"#)
    );
    language!("Ruby", vec!["rb"], None, None);
    language!("Swift", vec!["swift"], None, None);
    language!("Scala", vec!["sc"], None, None);
    language!(
        "Shell",
        vec!["sh", "bash", "zsh", "fish"],
        regex!(r#"#"#),
        None
    );
    language!("SQL", vec!["sql"], None, None);
    language!("SVG", vec!["svg"], None, None);
    language!(
        "TypeScript",
        vec!["ts"],
        regex!(r#"^\s*//"#),
        regex!(r#"/\*"#, r#"\*/"#)
    );
    language!("TypeScript JSX", vec!["tsx"], None, None);
    language!("TOML", vec!["toml"], None, None);
    language!("Vue", vec!["vue"], None, None);
    language!("VimScript", vec!["vim"], None, None);
    language!("XML", vec!["xml"], None, None);
    language!("YAML", vec!["yml", "yaml"], regex!(r#"#"#), None);

    config
}

#[derive(Debug, Default)]
pub struct Config {
    pub data: Vec<Language>,
}

#[derive(Debug)]
pub struct Language {
    pub extension: Vec<&'static str>,
    pub name: &'static str,
    pub single: Option<Regex>,
    pub multi: Option<(Regex, Regex)>,
}

impl Config {
    pub fn get(&self, extension: &str) -> Option<&Language> {
        for item in &self.data {
            for ext in &item.extension {
                if *ext == extension {
                    return Some(&item);
                }
            }
        }
        None
    }
}
