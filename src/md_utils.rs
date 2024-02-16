use crate::{
    date_utils::FORMAT_DATE,
    io::{get_priority_symbol, get_progress_str},
    model::{MyDate, Todo},
    // MyDateTime, Progress,
    // FORMAT_DATETIME,
};
use anyhow::{
    // bail,
    Context,
    Result,
};
// use chrono::{NaiveDate, NaiveDateTime};
// use once_cell::sync::Lazy;
// use regex::Regex;
use std::{env, fs, io::Write};

// Converts a TODO to markdown format
pub fn convert_todo_str(todo: &Todo) -> String {
    let completed_part = if todo.is_complete() { "[x]" } else { "[ ]" };
    let title = todo.get_title();
    let priority = get_priority_symbol(todo.get_priority());
    let due = match todo.get_due_date() {
        Some(MyDate(date)) => date.format(FORMAT_DATE).to_string(),
        None => "never".to_string(),
    };
    let progress = get_progress_str(todo);
    let created = todo
        .get_created_date()
        .get_0()
        .format(FORMAT_DATE)
        .to_string();

    format!(
        "- {} ({}) {} (due: {}) {} % {} % {}\n",
        completed_part,
        priority,
        title,
        due,
        progress,
        created,
        todo.get_id()
    )
}
// Exports all TODOs from a vector to markdown
pub fn export_to_md(todos: &[Todo]) -> Result<()> {
    let md_filename = env::var("MD_FILE").unwrap_or("./todo.md".to_string());
    let mut md_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(md_filename)
        .with_context(|| "Error opening markdown file!")?;
    let total_write: String = todos.iter().map(convert_todo_str).collect();
    md_file.write_all(total_write.as_bytes())?;
    Ok(())
}

// To finish!!

// pub fn extract_components_mdline(line_piece: &str) -> Vec<&str> {
//     static RETASK: Lazy<Regex> = Lazy::new(|| {
//         Regex::new(r"^-\s\[([\sx])\]\s\((!{1,3}|_)\)\s(.*)\s\(due:\s((?:\d{2}-\d{2}-\d{4})?)\)\s\[([#\s]{0,8})\]\s$").unwrap()
//     });
//     RETASK.find_iter(line_piece).map(|m| m.as_str()).collect()
// }

// pub fn parse_line_md(line: &str) -> Result<Todo> {
//     let split_str = line.split('%').collect::<Vec<&str>>();
//     if split_str.len() != 3 {
//         bail!("Parsing error for todo line in .md!")
//     }
//     let id = split_str[2];
//     let created = MyDateTime(NaiveDateTime::parse_from_str(
//         split_str[1],
//         FORMAT_DATETIME,
//     )?);

//     let blocks = extract_components_mdline(split_str[0]);
//     if blocks.len() < 5 {
//         bail!("Parsing error for todo line in .md!")
//     }
//     let completed = blocks[0].find('x').is_some();
//     let priority = blocks[1].matches('!').count() as u32;
//     let title = blocks[2];
//     let due = NaiveDate::parse_from_str(blocks[3], FORMAT_DATE);
//     let due_final = match due {
//         Ok(due_obj) => Some(MyDate(due_obj)),
//         Err(_) => None,
//     };
//     let count_hashtags = blocks[4].matches('#').count();
//     let progress = match count_hashtags {
//         0 => Progress::Zero,
//         2 => Progress::Quarter,
//         4 => Progress::Half,
//         6 => Progress::ThreeQuarter,
//         8 => Progress::Full,
//         _ => bail!("Wrong progress marker!"),
//     };
//     Ok(Todo::from_scratch(
//         id, title, priority, created, due_final, completed, progress,
//     ))
// }
