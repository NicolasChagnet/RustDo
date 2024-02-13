use crate::{
    io::{get_priority_symbol, get_progress_str},
    model::{Todo, MyDate},
    date_utils::FORMAT_DATE
};
use std::{
    env,
    fs, io::Write
};
use anyhow::{Context, Result};

// Converts a TODO to markdown format
pub fn convert_todo_str(todo: &Todo) -> String {
    let completed_part = if todo.is_complete() { "[x]" } else { "[ ]" };
    let title = todo.get_title();
    let priority = get_priority_symbol(todo.get_priority());
    let due = match todo.get_due_date() {
        Some(MyDate(date)) => date.format(FORMAT_DATE).to_string(),
        None => "".to_string()
    };
    let progress = get_progress_str(todo);
    let created = todo.get_created_date().get_0().format(FORMAT_DATE).to_string();

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
    let total_write: String = todos
        .iter()
        .map(convert_todo_str)
        .collect();
    md_file.write_all(total_write.as_bytes())?;
    Ok(())
}
