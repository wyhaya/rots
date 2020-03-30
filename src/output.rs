use crate::Detail;
use std::convert::TryFrom;

#[derive(Debug)]
pub enum Format {
    ASCII,
    HTML,
    Markdown,
}

impl TryFrom<&str> for Format {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "ascii" => Ok(Format::ASCII),
            "html" => Ok(Format::HTML),
            "markdown" => Ok(Format::Markdown),
            _ => Err(()),
        }
    }
}

impl Default for Format {
    fn default() -> Self {
        Format::ASCII
    }
}

macro_rules! total {
    ($name: ident, $type: path) => {
        fn $name(&self) -> $type {
            let mut n = 0;
            for item in self.0 {
                n += item.$name
            }
            n
        }
    };
}

struct Total<'a>(&'a Vec<Detail>);

impl<'a> Total<'a> {
    total!(code, i32);
    total!(comment, i32);
    total!(blank, i32);
    total!(file, i32);
    total!(size, u64);
}

pub struct Output(pub Vec<Detail>);

impl Output {
    pub fn new(data: Vec<Detail>) -> Self {
        Output(data)
    }

    pub fn print(self, format: Format) {
        match format {
            Format::ASCII => self.ascii(),
            Format::HTML => self.html(),
            Format::Markdown => self.markdown(),
        };
    }

    fn ascii(&self) {
        println!("┌{:─<78}┐", "");
        println!(
            "| {:<14}{:>12}{:>12}{:>12}{:>12}{:>14} |",
            "Language", "Code", "Comment", "Blank", "File", "Size"
        );
        println!("├{:─<78}┤", "");

        for item in &self.0 {
            println!(
                "| {:<14}{:>12}{:>12}{:>12}{:>12}{:>14} |",
                item.language,
                item.code,
                item.comment,
                item.blank,
                item.file,
                bytes_to_size(item.size as f64)
            );
        }
        println!("├{:─<78}┤", "");
        let total = Total(&self.0);
        println!(
            "| {:<14}{:>12}{:>12}{:>12}{:>12}{:>14} |",
            "Total",
            format_number(total.code()),
            format_number(total.comment()),
            format_number(total.blank()),
            format_number(total.file()),
            bytes_to_size(total.size() as f64)
        );
        println!("└{:─<78}┘", "");
    }

    fn html(&self) {
        println!("<table>");
        println!(
            "   <thead>
        <tr>
            <th>Language</th>
            <th>Code</th>
            <th>Comment</th>
            <th>Blank</th>
            <th>File</th>
            <th>Size</th>
        </tr>
    </thead>"
        );

        println!("    <tbody>");
        for item in &self.0 {
            println!(
                "        <tr>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
        </tr>",
                item.language,
                item.code,
                item.comment,
                item.blank,
                item.file,
                bytes_to_size(item.size as f64)
            );
        }
        println!("    </tbody>");
        let total = Total(&self.0);
        println!(
            "    <tfoot>
        <tr>
            <td>Total</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
        </tr>
    </tfoot>",
            format_number(total.code()),
            format_number(total.comment()),
            format_number(total.blank()),
            format_number(total.file()),
            bytes_to_size(total.size() as f64)
        );

        println!("</table>");
    }

    fn markdown(&self) {
        println!(
            "| {:<14} | {:<12} | {:<12} | {:<12} | {:<12} | {:<14} |",
            "Language", "Code", "Comment", "Blank", "File", "Size"
        );
        println!(
            "| {:-<14} | {:-<12} | {:-<12} | {:-<12} | {:-<12} | {:-<14} |",
            "", "", "", "", "", ""
        );
        for item in &self.0 {
            println!(
                "| {:<14} | {:<12} | {:<12} | {:<12} | {:<12} | {:<14} |",
                item.language,
                item.code,
                item.comment,
                item.blank,
                item.file,
                bytes_to_size(item.size as f64)
            );
        }
        let total = Total(&self.0);
        println!(
            "| {:<14} | {:<12} | {:<12} | {:<12} | {:<12} | {:<14} |",
            "Total",
            format_number(total.code()),
            format_number(total.comment()),
            format_number(total.blank()),
            format_number(total.file()),
            bytes_to_size(total.size() as f64)
        );
    }
}

fn bytes_to_size(bytes: f64) -> String {
    const SIZE: f64 = 1024_f64;
    const UNITS: [&str; 9] = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    if bytes <= 1_f64 {
        return format!("{:.2} B", bytes);
    }
    let i = (bytes.ln() / SIZE.ln()) as i32;
    format!("{:.2} {}", bytes / SIZE.powi(i), UNITS[i as usize])
}

fn format_number<T: ToString>(num: T) -> String {
    let text = num.to_string();
    let mut vec = Vec::new();
    for (i, ch) in text.chars().rev().into_iter().enumerate() {
        if i != 0 && i % 3 == 0 {
            vec.push(',');
        }
        vec.push(ch);
    }
    vec.reverse();
    let mut s = String::with_capacity(vec.len());
    s.extend(&vec);
    s
}
