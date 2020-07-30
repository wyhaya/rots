

# lok

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/wyhaya/lok/Build?style=flat-square)](https://github.com/wyhaya/lok/actions)
[![Crates.io](https://img.shields.io/crates/v/lok.svg?style=flat-square)](https://crates.io/crates/lok)
[![LICENSE](https://img.shields.io/crates/l/lok.svg?style=flat-square)](https://github.com/wyhaya/lok/blob/master/LICENSE)

`lok` is a command line tool, that is used to quickly calculate the number of lines of various language codes in a project

```
┌──────────────────────────────────────────────────────────────────────────────┐
| Language              Code     Comment       Blank        File          Size |
├──────────────────────────────────────────────────────────────────────────────┤
| HTML                   360           0          27          13      24.97 KB |
| JavaScript             238         240          79          22     935.95 KB |
| JavaScript JSX       26570        2011        4096         299     766.10 KB |
| JSON                    81           0           3           4       1.97 KB |
| Markdown                31           0          13           1      882.00 B |
| TypeScript              57           6          12           3       3.78 KB |
| TypeScript JSX         691          78          46          10      19.12 KB |
| YML                      4           0           0           1       58.00 B |
├──────────────────────────────────────────────────────────────────────────────┤
| Total               28,032       2,335       4,276         353       1.71 MB |
└──────────────────────────────────────────────────────────────────────────────┘
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

