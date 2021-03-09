use crate::Detail;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug)]
pub enum Format {
    Table,
    Html,
    Markdown,
}

impl FromStr for Format {
    type Err = ();
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "table" => Ok(Format::Table),
            "html" => Ok(Format::Html),
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
            Format::Html => self.html(&mut data),
            Format::Markdown => self.markdown(&mut data),
        };

        println!("{}", data.join("\n"));
    }

    fn table(&self, data: &mut Vec<String>) {
        data.push(format!("╭{:─<78}╮", ""));
        data.push(format!(
            "│ {:<14}{:>12}{:>12}{:>12}{:>12}{:>14} │",
            "Language", "Code", "Comment", "Blank", "File", "Size"
        ));
        data.push(format!("├{:─<78}┤", ""));

        for item in &self.data {
            data.push(format!(
                "│ {:<14}{:>12}{:>12}{:>12}{:>12}{:>14} │",
                item.language,
                item.code,
                item.comment,
                item.blank,
                item.file,
                format_size(item.size)
            ));
        }

        data.push(format!("├{:─<78}┤", ""));

        data.push(format!(
            "│ {:<14}{:>12}{:>12}{:>12}{:>12}{:>14} │",
            "Total",
            format_number(self.total_code),
            format_number(self.total_comment),
            format_number(self.total_blank),
            format_number(self.total_file),
            format_size(self.total_size)
        ));
        data.push(format!("╰{:─<78}╯", ""));
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
                format_size(item.size)
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
            format_size(self.total_size)
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
                format_size(item.size)
            ));
        }

        data.push(format!(
            "| {:<14} | {:<12} | {:<12} | {:<12} | {:<12} | {:<14} |",
            "Total",
            format_number(self.total_code),
            format_number(self.total_comment),
            format_number(self.total_blank),
            format_number(self.total_file),
            format_size(self.total_size)
        ));
    }
}

fn format_size(n: u64) -> String {
    const UNITS: [char; 6] = ['K', 'M', 'G', 'T', 'P', 'E'];
    if n < 1024 {
        return format!("{} B ", n);
    }
    let bytes = n as f64;
    let i = (bytes.ln() / 1024_f64.ln()) as i32;
    format!(
        "{:.2} {}B",
        bytes / 1024_f64.powi(i),
        UNITS[(i - 1) as usize]
    )
}

fn format_number<T: Display>(num: T) -> String {
    let num = format!("{}", num);
    let mut rst = String::new();
    for (i, ch) in num.chars().enumerate() {
        rst.push(ch);
        if i != num.len() - 1 && (num.len() - 1 - i) % 3 == 0 {
            rst.push(',');
        }
    }
    rst
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B ");
        assert_eq!(format_size(1), "1 B ");
        assert_eq!(format_size(1023), "1023 B ");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1 * 1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1 * 1024 * 1024 * 1024 * 1024), "1.00 TB");
        assert_eq!(format_size(u64::max_value()), "16.00 EB");
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
