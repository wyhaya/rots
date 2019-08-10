use std::fs;
use std::io::Write;

fn main() {
    let all = vec![
        ("js", "//"),
        ("ts", "//"),
        ("rs", "//"),
        ("go", "//"),
        ("py", "#"),
        ("php", "#"),
        ("sh", "#"),
    ];
    for name in all {
        create(name);
    }
}

fn create((name, comment): (&str, &str)) {
    fs::create_dir_all(format!("./temp/{}", &name)).unwrap();
    let cur = format!("./temp/{}/0.{}", &name, &name);
    let mut f = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&cur)
        .unwrap();
    for i in 0..9999 {
        writeln!(f, "{}", "let a = 0;").unwrap();
        if i % 5 == 0 {
            writeln!(f, "{}", "    ").unwrap();
        }
        if i % 8 == 0 {
            writeln!(f, "{}", format!("{} this is a comment", &comment)).unwrap();
        }
    }
    for i in 1..99 {
        fs::copy(&cur, format!("./temp/{}/{}.{}", &name, i, &name)).unwrap();
    }
}
