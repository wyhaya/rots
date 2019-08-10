

# lok

[![Build Status](https://img.shields.io/travis/wyhaya/lok.svg?style=flat-square)](https://travis-ci.org/wyhaya/lok)
[![Crates.io](https://img.shields.io/crates/l/lok.svg?style=flat-square)](https://github.com/wyhaya/lok/blob/master/LICENSE)

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
| Total                28032        2335        4276         353       1.71 MB |
└──────────────────────────────────────────────────────────────────────────────┘
```

## Features

* Quickly calculate data
* Support multiple languages
* Support multiple outputs, ASCII, HTML, Markdown

## Install

[Download](https://github.com/wyhaya/lok/releases) the binary from the release page

Or use `cargo` to install

```bash
cargo install lok
```

## Use

Go to your project in the terminal and type lok on the command line

```bash
cd your-project
lok
```

```bash
# If you want to calculate some languages
lok -e ts js
```

```bash
# If you want to ignore files
lok -i node_modules dist

# Use regular expressions
lok -i 'node_\w+|dist'

# Ignore all files with the extension js jsx ts tsx
lok -i 'jsx?$' 'tsx?$'
```

```bash
# If you want to output other formats: ascii, html, markdown
lok -o markdown

# Save to file
lok -o html > code.html
lok -o markdown > code.md
```

```bash
# Change working directory
lok -p /root/code
```
      
```bash
# Sort by: code, comment, blank, file, size
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

## License

[MIT](./LICENSE) LICENSE

