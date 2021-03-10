

# lok

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/wyhaya/lok/Build?style=flat-square)](https://github.com/wyhaya/lok/actions)
[![Crates.io](https://img.shields.io/crates/v/lok.svg?style=flat-square)](https://crates.io/crates/lok)
[![LICENSE](https://img.shields.io/crates/l/lok.svg?style=flat-square)](https://github.com/wyhaya/lok/blob/master/LICENSE)

`lok` is a command line tool, that is used to quickly calculate the number of lines of various language codes in a project

```
╭──────────────────────────────────────────────────────────────────────────────╮
│ Language              Code     Comment       Blank        File          Size │
├──────────────────────────────────────────────────────────────────────────────┤
│ CSS                   6384           9          40         108     145.89 KB │
│ HTML                    19           0           0           1       1011 B  │
│ JSON                   205           0           0         110      13.25 KB │
│ JavaScript             206           8           1         113     590.14 KB │
│ Markdown                 8           0           5           1        229 B  │
│ TypeScript JSX       14733         262         800         109     530.76 KB │
│ TypeScript             680          57          57           8      22.07 KB │
│ YAML                     5           0           0           1         83 B  │
├──────────────────────────────────────────────────────────────────────────────┤
│ Total               22,240         336         903         451       1.27 MB │
╰──────────────────────────────────────────────────────────────────────────────╯
```

## Features

* Quickly calculate data
* Support multiple languages
* Support multiple output formats, ASCII, HTML, Markdown

## Install

[Download](https://github.com/wyhaya/lok/releases) the binary from the release page

Or use `cargo` to install

```bash
cargo install lok
```

## Use

Go to your project in the terminal and type `lok` on the command line

```bash
cd your-project
lok

# Change working directory
lok /root/code
```

```bash
# Exclude all files matched by glob
lok -e './node_modules/**'

# Exclude all files with the specified extension
lok -e '**/*.ts' '**/*.js'
```

```bash
# Include only files matching glob
lok -i './src/*.rs'
```

```bash
# Only count files containing extensions
lok --extension js ts jsx tsx
```

```bash
# Output other formats: table, html, markdown
lok -o markdown

# Save to file
lok -o html > code.html
lok -o markdown > code.md
```

```bash
# Sort by: language, code, comment, blank, file, size
lok -s code
```    
 
## Contributing

If you want to add statistics for other languages, please update [config.rs](./src/config.rs)

Example:

```rust
language!(
    "Rust", 
    vec!["rs"], 
    vec!["//", "///"], 
    vec![("/*", "*/")]
);
// ...
```

## Benchmark

First need to install

```bash
cargo install hyperfine loc tokei
```

Run

```bash
./benchmark
```

## License

[MIT](./LICENSE) LICENSE

