use crate::Detail;
use std::str::FromStr;

#[derive(Debug)]
pub enum Format {
    Table,
    HTML,
    Markdown,
}

impl FromStr for Format {
    type Err = ();
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "table" => Ok(Format::Table),
            "html" => Ok(Format::HTML),
            "markdown" => Ok(Format::Markdown),
            _ => Err(()),
        }
    }
}

#[derive(Default)]
pub struct Output {
    pub data: Vec<Detail>,

    pub total_code: i32,
    pub total_comment: i32,
    pub total_blank: i32,
    pub total_file: i32,
    pub total_size: u64,
}

impl Output {
    pub fn new(data: Vec<Detail>) -> Self {
        let (total_code, total_comment, total_blank, total_file, total_size) = data
            .iter()
            .map(|detail| {
                (
                    detail.code,
                    detail.comment,
                    detail.blank,
                    detail.file,
                    detail.size,
                )
            })
            .fold((0, 0, 0, 0, 0), |p, n| {
                (p.0 + n.0, p.1 + n.1, p.2 + n.2, p.3 + n.3, p.4 + n.4)
            });

        Self {
            data,
            total_code,
            total_comment,
            total_blank,
            total_file,
            total_size,
        }
    }

    pub fn print(self, format: Format) {
        let mut data = vec![];
        match format {
            Format::Table => self.table(&mut data),
            Format::HTML => self.html(&mut data),
            Format::Markdown => self.markdown(&mut data),
        };

        println!("{}", data.join("\n"));
    }

    fn table(&self, data: &mut Vec<String>) {
        data.push(format!("┌{:─<78}┐", ""));
        data.push(format!(
            "| {:<14}{:>12}{:>12}{:>12}{:>12}{:>14} |",
            "Language", "Code", "Comment", "Blank", "File", "Size"
        ));
        data.push(format!("├{:─<78}┤", ""));

        for item in &self.data {
            data.push(format!(
                "| {:<14}{:>12}{:>12}{:>12}{:>12}{:>14} |",
                item.language,
                item.code,
                item.comment,
                item.blank,
                item.file,
                bytes_to_size(item.size)
            ));
        }

        data.push(format!("├{:─<78}┤", ""));

        data.push(format!(
            "| {:<14}{:>12}{:>12}{:>12}{:>12}{:>14} |",
            "Total",
            format_number(self.total_code),
            format_number(self.total_comment),
            format_number(self.total_blank),
            format_number(self.total_file),
            bytes_to_size(self.total_size)
        ));
        data.push(format!("└{:─<78}┘", ""));
    }

    fn html(&self, data: &mut Vec<String>) {
        data.push("<table>".to_string());
        data.push(
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
                .to_string(),
        );
        data.push("    <tbody>".to_string());

        for item in &self.data {
            data.push(format!(
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
            ));
        }
        data.push("    </tbody>".to_string());

        data.push(format!(
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
            format_number(self.total_code),
            format_number(self.total_comment),
            format_number(self.total_blank),
            format_number(self.total_file),
            bytes_to_size(self.total_size)
        ));
        data.push("</table>".to_string());
    }

    fn markdown(&self, data: &mut Vec<String>) {
        data.push(format!(
            "| {:<14} | {:<12} | {:<12} | {:<12} | {:<12} | {:<14} |",
            "Language", "Code", "Comment", "Blank", "File", "Size"
        ));
        data.push(format!(
            "| :{:-<13} | {:-<11}: | {:-<11}: | {:-<11}: | {:-<11}: | {:-<13}: |",
            "", "", "", "", "", ""
        ));
        for item in &self.data {
            data.push(format!(
                "| {:<14} | {:<12} | {:<12} | {:<12} | {:<12} | {:<14} |",
                item.language,
                item.code,
                item.comment,
                item.blank,
                item.file,
                bytes_to_size(item.size)
            ));
        }

        data.push(format!(
            "| {:<14} | {:<12} | {:<12} | {:<12} | {:<12} | {:<14} |",
            "Total",
            format_number(self.total_code),
            format_number(self.total_comment),
            format_number(self.total_blank),
            format_number(self.total_file),
            bytes_to_size(self.total_size)
        ));
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
