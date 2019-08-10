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

    language!(
        "ASP.NET",
        vec!["asax", "ascx", "asmx", "aspx", "master", "sitemap", "webinfo"],
        vec![],
        vec![("<!--", "-->"), ("<%--", "-->")]
    );
    language!("C", vec!["c"], vec!["//"], vec![("/*", "*/")]);
    language!(
        "CSS",
        vec!["css", "scss", "sass", "less"],
        vec!["//"],
        vec![("/*", "*/")]
    );
    language!("C++", vec!["cpp"], vec!["//"], vec![("/*", "*/")]);
    language!(
        "CoffeeScript",
        vec!["coffee"],
        vec!["#"],
        vec![("###", "###")]
    );
    language!("C#", vec!["cs"], vec!["//", "///"], vec![("/*", "*/")]);
    language!("D", vec!["d"], vec!["//", "///"], vec![("/*", "*/")]);
    language!("Dart", vec!["dart"], vec!["//", "///"], vec![("/*", "*/")]);
    language!("Go", vec!["go"], vec!["//"], vec![("/*", "*/")]);
    language!("HTML", vec!["htm", "html"], vec![], vec![("<!--", "-->")]);
    language!("Haskell", vec!["hs"], vec!["--"], vec![("{-", "-}")]);
    language!(
        "JavaScript",
        vec!["js", "mjs"],
        vec!["//"],
        vec![("/*", "*/")]
    );
    language!(
        "JavaScript JSX",
        vec!["jsx"],
        vec!["//"],
        vec![("/*", "*/")]
    );
    language!("JSON", vec!["json"], vec![], vec![]);
    language!("Julia", vec!["jl"], vec!["#"], vec![("#=", "=#")]);
    language!("Java", vec!["java"], vec!["//"], vec![("/*", "*/")]);
    language!("LLVM", vec!["ll"], vec![";"], vec![]);
    language!("Lua", vec!["lua"], vec!["--"], vec![("--[[", "]]")]);
    language!("Markdown", vec!["md", "markdown"], vec![], vec![]);
    language!("Nim", vec!["nim"], vec!["#"], vec![("＃[", "]#")]);
    language!(
        "ObjectiveC",
        vec!["m"],
        vec!["//", "///"],
        vec![("/*", "*/")]
    );
    language!("Objective-C++", vec!["mm"], vec!["//"], vec![("/*", "*/")]);
    language!("PHP", vec!["php"], vec!["//", "#"], vec![("/*", "*/")]);
    language!(
        "Python",
        vec!["py"],
        vec!["#"],
        vec![("'''", "'''"), (r#"""""#, r#"""""#)]
    );
    language!("Perl", vec!["pl", "pm"], vec!["#"], vec![("=", "=")]);
    language!("R", vec!["r"], vec!["#"], vec![]);
    language!("Rust", vec!["rs"], vec!["//", "///"], vec![("/*", "*/")]);
    language!("Ruby", vec!["rb"], vec!["#"], vec![("=", "=")]);
    language!("Swift", vec!["swift"], vec!["//"], vec![("/*", "*/")]);
    language!("Scala", vec!["sc"], vec!["//"], vec![("/*", "*/")]);
    language!(
        "Shell",
        vec!["sh", "bash", "zsh", "fish"],
        vec!["#"],
        vec![]
    );
    language!("SQL", vec!["sql"], vec!["--"], vec![("/*", "*/")]);
    language!("TypeScript", vec!["ts"], vec!["//"], vec![("/*", "*/")]);
    // todo
    language!(
        "TypeScript JSX",
        vec!["tsx"],
        vec!["//"],
        vec![("/*", "*/")]
    );
    language!("TOML", vec!["toml"], vec!["#"], vec![]);
    // This file may contain multiple languages. html.. js ts .. css scss sass..
    // Not processed here
    language!(
        "Vue",
        vec!["vue"],
        vec!["//"],
        vec![("<!--", "-->"), ("/*", "*/")]
    );
    language!("VimScript", vec!["vim"], vec![], vec![]);
    language!("XML", vec!["xml"], vec![], vec![("<!--", "-->")]);
    language!("YAML", vec!["yml", "yaml"], vec!["#"], vec![]);

    config
}

#[derive(Debug, Default)]
pub struct Config {
    pub data: Vec<Language>,
}

#[derive(Debug)]
pub struct Language {
    pub name: &'static str,
    pub extension: Vec<&'static str>,
    pub single: Vec<&'static str>,
    pub multi: Vec<(&'static str, &'static str)>,
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
