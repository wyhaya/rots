

# lok

[![Build Status](https://img.shields.io/travis/wyhaya/lok.svg?style=flat-square)](https://travis-ci.org/wyhaya/lok)
[![Crates.io](https://img.shields.io/crates/l/lok.svg?style=flat-square)](https://github.com/wyhaya/lok/blob/master/LICENSE)

`lok` is a command line tool, that is used to quickly calculate the number of lines of various language codes in a project

![preview](https://user-images.githubusercontent.com/23690145/51882818-3c5b8c80-23bb-11e9-8da6-5e7b19a7f536.png)


## Features

* Quickly calculate data
* Support multiple languages
* Support multiple outputs, ASCII, HTML, MarkDown

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

If you want to add statistics for other languages, please update [./src/main.rs](./src/main.rs)

Example:

```rust
language!(
    "js",
    "JavaScript",
    regex!(r#"^\s*//"#),
    regex!(r#"/\*"#, r#"\*/"#)
);
// ...
```

## License

[MIT](./LICENSE) LICENSE

