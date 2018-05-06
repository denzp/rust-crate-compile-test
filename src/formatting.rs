use std::fmt;
use std::iter::repeat;

pub fn display_list<T: fmt::Display>(list: &Vec<T>) -> String {
    match list.len() {
        0 => "".into(),
        1 => {
            String::from(" └─ ")
                + &prefix_each_next_line(list.iter().last().unwrap().to_string(), "    ")
                + "\n"
        }
        _ => trim_lines(
            String::from(" ├─ ")
                + &list.iter()
                    .take(list.len() - 1)
                    .map(|item| prefix_each_next_line(item.to_string(), " │  "))
                    .collect::<Vec<_>>()
                    .join("\n │\n ├─ ") + "\n │\n └─ "
                + &prefix_each_next_line(list.iter().last().unwrap().to_string(), "    ")
                + "\n",
        ),
    }
}

pub fn display_block<S1: AsRef<str>, S2: AsRef<str>>(header: S1, body: S2) -> String {
    format!(
        " {}\n╭{}\n{}\n╰",
        header.as_ref(),
        repeat("─")
            .take(header.as_ref().len())
            .collect::<String>(),
        trim_lines(prefix_each_line(body, "│ ")).trim_right()
    )
}

pub fn prefix_each_line<S: AsRef<str>>(input: S, prefix: &str) -> String {
    String::from(prefix) + &prefix_each_next_line(input, prefix)
}

pub fn prefix_each_next_line<S: AsRef<str>>(input: S, prefix: &str) -> String {
    trim_lines(input.as_ref().replace("\n", &format!("\n{}", prefix)))
}

fn trim_lines(input: String) -> String {
    input
        .lines()
        .map(|line| line.trim_right())
        .collect::<Vec<_>>()
        .join("\n")
}