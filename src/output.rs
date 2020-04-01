use crate::Detail;
use std::convert::TryFrom;

#[derive(Debug)]
pub enum Format {
    Table,
    HTML,
    Markdown,
}

impl TryFrom<&str> for Format {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "table" => Ok(Format::Table),
            "html" => Ok(Format::HTML),
            "markdown" => Ok(Format::Markdown),
            _ => Err(()),
        }
    }
}

impl Default for Format {
    fn default() -> Self {
        Format::Table
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
            Format::Table => self.table(),
            Format::HTML => self.html(),
            Format::Markdown => self.markdown(),
        };
    }

    fn table(&self) {
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
                bytes_to_size(item.size)
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
            bytes_to_size(total.size())
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
                bytes_to_size(item.size)
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
            bytes_to_size(total.size())
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
                bytes_to_size(item.size)
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
            bytes_to_size(total.size())
        );
    }
}

fn bytes_to_size(bytes: u64) -> String {
    const UNITS: [&str; 7] = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];
    if bytes < 1024 {
        return format!("{}.00 B", bytes);
    }
    let bytes = bytes as f64;
    let i = (bytes.ln() / 1024_f64.ln()) as i32;
    format!("{:.2} {}", bytes / 1024_f64.powi(i), UNITS[i as usize])
}

fn format_number<T: ToString>(num: T) -> String {
    let text = num.to_string();
    let mut vec = Vec::new();
    for (i, ch) in text.chars().rev().enumerate() {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bytes_to_size() {
        assert_eq!(bytes_to_size(0), "0.00 B");
        assert_eq!(bytes_to_size(1), "1.00 B");
        assert_eq!(bytes_to_size(1023), "1023.00 B");
        assert_eq!(bytes_to_size(1024), "1.00 KB");
        assert_eq!(bytes_to_size(1 * 1024 * 1024), "1.00 MB");
        assert_eq!(bytes_to_size(1 * 1024 * 1024 * 1024 * 1024), "1.00 TB");
        assert_eq!(bytes_to_size(u64::max_value()), "16.00 EB");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(1), "1");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(999999), "999,999");
        assert_eq!(format_number(1234567), "1,234,567");
    }
}
