use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use crate::{
    date_utils::{validate_regex, FORMAT_DATE}, model::{SortingMethod, Todo}, service
};
use chrono::Local;
use console::{style,Term,StyledObject,Key};
use anyhow::{Context,Result};

pub fn input_title() -> Result<String> {
    let input = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Title (leave empty to go back): ")
        .allow_empty(true)
        .interact_text()
        .with_context(|| "Error reading input!")?;
    Ok(input)
}

pub fn input_due_date() -> Result<String> {
    let input =Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Due date [dd-mm(-YYYY)]: ")
        .allow_empty(true)
        .validate_with(validate_regex)
        .interact_text()
        .with_context(|| "Error reading text!")?;
    Ok(input.to_lowercase())
}

pub fn input_priority() -> Result<u32> {
    let priorities = vec![0, 1, 2, 3];
    
    let selection = Select::new()
        .with_prompt("Select a priority level")
        .items(&priorities)
        .default(0)
        .interact()
        .with_context(|| "Error reading priorities")?;

    Ok(priorities[selection])
}


pub fn get_title_complete(todo: &Todo) -> StyledObject<&str> {
    let style_base = style(todo.get_title());
    match todo.is_complete() {
        true => style_base.strikethrough(),
        false => style_base
    }
}

pub fn get_due_date(todo: &Todo) -> Option<StyledObject<String>> {
    let today = Local::now().date_naive();
    todo.get_due_date().map_or_else(
        || None,
        |due| {
            let base = style(due.format(FORMAT_DATE).to_string());
            if due > today {
                Some(base.green())
            } else if due == today {
                Some(base.yellow())
            } else {
                Some(base.red())
            }
        })
}

pub fn write_todo(todo: &Todo) -> Result<()> {
    let term = Term::stdout();
    let title = get_title_complete(&todo);
    let date = get_due_date(todo);
    let priority = get_priority_symbol(todo.get_priority());
    match date {
        Some(v) => term
            .write_line(&format!("{} {} - Due: {}", priority, title, v))
            .with_context(|| "Error while writing line!")?,
        None => term
            .write_line(&format!("{} {}", priority, title))
            .with_context(|| "Error while writing line!")?
    };
    Ok(())
}

pub fn clear_term() -> Result<()> {
    let term = Term::stdout();
    term.clear_screen().with_context(|| "Error clearing screen!")?;
    Ok(())
}

pub fn write_todos(todos: &mut Vec<Todo>) -> Result<()> {
    clear_term()?;
    for todo in todos.iter() {
        write_todo(todo)?
    }
    let sorting = wait_sort_key()?;
    match sorting {
        Some(SortingMethod::Created) => {
            service::sort_todos_by_created_date_asc(todos);
            write_todos(todos)?;
        },
        Some(SortingMethod::Due) => {
            service::sort_todos_by_due_date_asc(todos);
            write_todos(todos)?;
        },
        Some(SortingMethod::Priority) => {
            service::sort_todos_by_priority_desc(todos);
            write_todos(todos)?;
        },
        None => ()
    }
    // wait_enter_key()?;
    Ok(())
}

pub fn write_no_todos() -> Result<()> {
    let term = Term::stdout();
    clear_term()?;
    term.write_line("No TODOs to show!")?;
    wait_enter_key()?;
    Ok(())
}

fn get_priority_symbol(p: u32) -> String {
    match p {
        0 => "_".to_string(),
        1 => "!".to_string(),
        2 => "!!".to_string(),
        3 => "!!!".to_string(),
        _ => unreachable!()
    }
}

// pub fn select_change_status(titles: &Vec<ANSIGenericString<'_, str>>) -> Vec<usize> {
pub fn select_change_status(titles: &Vec<StyledObject<&str>>) -> Result<Vec<usize>> {
    let multi_select = MultiSelect::new()
        .with_prompt("Change the status of an TODO?")
        .items(titles)
        .interact()
        .with_context(|| "Error selecting options!")?;
    Ok(multi_select)
}

pub fn wait_enter_key() -> Result<()> {
    let term = Term::stdout();
    term.write_line("Press Enter to continue...")
        .with_context(|| "Error writing line!")?;
    loop {
        let key = term
        .read_key()
        .with_context(|| "Error reading key!")?;
        if key == Key::Enter {
            break;
        }
    }
    Ok(())
}

// This function requests a key and returns the sorting type we wish to use
pub fn wait_sort_key() -> Result<Option<SortingMethod>> {
    let term = Term::stdout();
    term.write_line("Press Enter to continue, p to sort by priority, d by due date, c by date of create...")
        .with_context(|| "Error writing line!")?;
    loop {
        let key = term
            .read_key()
            .with_context(|| "Error reading key!")?;
        match key {
            Key::Enter => return Ok(None),
            Key::Char('p') => return Ok(Some(SortingMethod::Priority)),
            Key::Char('d') => return Ok(Some(SortingMethod::Due)),
            Key::Char('c') => return Ok(Some(SortingMethod::Created)),
            _ => continue
        }
    }
}