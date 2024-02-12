use std::cmp::Ordering;

use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use crate::{
    date_utils::{validate_regex, FORMAT_DATE}, 
    model::{Action, KeyEvent, SortingMethod, Todo}, 
    service, Progress, MAXPRIORITY
};
use chrono::{Local, NaiveDate};
use console::{style,Term,StyledObject,Key};
use anyhow::{Context,Result};

pub fn input_title(prewrite: Option<&str>) -> Result<String> {
    let input = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Title (leave empty to go back): ")
        .with_initial_text(prewrite.unwrap_or("").to_string())
        .allow_empty(true)
        .interact_text()
        .with_context(|| "Error reading input!")?;
    Ok(input)
}

pub fn input_due_date(prewrite: Option<NaiveDate>) -> Result<String> {
    let prewrite_str = match prewrite {
        Some(date) => date.format(FORMAT_DATE).to_string(),
        None => "".to_string()
    };
    let input =Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Due date [dd-mm(-YYYY)]: ")
        .allow_empty(true)
        .with_initial_text(prewrite_str)
        .validate_with(validate_regex)
        .interact_text()
        .with_context(|| "Error reading text!")?;
    Ok(input.to_lowercase())
}

pub fn input_priority(init_position: usize) -> Result<u32> {
    let priorities: Vec<u32> = (0..=MAXPRIORITY).collect();
    
    let selection = Select::new()
        .with_prompt("Select a priority level")
        .items(&priorities)
        .default(init_position)
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
            if todo.is_complete() {
                return Some(base.dim());
            }
            match due.cmp(&today) {
                Ordering::Greater => Some(base.green()),
                Ordering::Equal => Some(base.yellow()),
                Ordering::Less => Some(base.red())
            }
        })
}

pub fn write_todo(todo: &Todo, is_position: bool) -> Result<()> {
    let term = Term::stdout();
    let title = get_title_complete(todo);
    let date = get_due_date(todo);
    let priority = get_priority_symbol(todo.get_priority());
    let initial_character = match is_position {
        true => ">",
        false => " "
    };
    let progress = todo.get_progress();
    let progress_bar = if todo.is_complete() {
        "[########]"
    } else {
        match progress {
            Progress::Zero => "[        ]",
            Progress::Quarter => "[##      ]",
            Progress::Half => "[####    ]",
            Progress::ThreeQuarter => "[######  ]",
            Progress::Full => "[########]",
            
        }
    };
    let str_write = match date {
        Some(date_str) => format!("{} {} {} - Due: {} - Progress: {}", initial_character, priority, title, date_str, progress_bar),
        None => format!("{} {} {} - Progress: {}", initial_character, priority, title, progress_bar)
    };
    term.write_line(&str_write).with_context(|| "Error while writing line!")?;
    Ok(())
}

pub fn clear_term() -> Result<()> {
    let term = Term::stdout();
    term.clear_screen().with_context(|| "Error clearing screen!")?;
    Ok(())
}

pub fn clear_lines(n: usize) -> Result<()> {
    let term = Term::stdout();
    term.clear_last_lines(n).with_context(|| "Error clearing screen!")?;
    Ok(())
}

pub fn show_cursor() -> Result<()> {
    let term = Term::stdout();
    term.show_cursor().with_context(|| "Error showing cursor!")?;
    Ok(())
}

pub fn hide_cursor() -> Result<()> {
    let term = Term::stdout();
    term.hide_cursor().with_context(|| "Error showing cursor!")?;
    Ok(())
}

pub fn write_no_todos() -> Result<()> {
    let term = Term::stdout();
    clear_term()?;
    term.write_line("No TODOs to show!")?;
    wait_backspace_key()?;
    Ok(())
}

fn get_priority_symbol(p: u32) -> String {
    match p {
        0 => "_".to_string(),
        _ => (1..=p).map(|_| "!").collect::<String>()
    }
}

pub fn select_change_status(titles: &[StyledObject<&str>]) -> Result<Vec<usize>> {
    let multi_select = MultiSelect::new()
        .with_prompt("Change the status of an TODO?")
        .items(titles)
        .interact()
        .with_context(|| "Error selecting options!")?;
    Ok(multi_select)
}

pub fn wait_backspace_key() -> Result<()> {
    let term = Term::stdout();
    term.write_line("Press Backspace to go back...")
        .with_context(|| "Error writing line!")?;
    loop {
        let key = term
        .read_key()
        .with_context(|| "Error reading key!")?;
        if key == Key::Backspace {
            break;
        }
    }
    Ok(())
}

// This function requests a key and returns the sorting type we wish to use
pub fn wait_sort_key() -> Result<Option<SortingMethod>> {
    let term = Term::stdout();
    term.write_line("
p: sort by priority   c: sort by date of creation\t
d: sort by due date   Backspace: go back
        ")
        .with_context(|| "Error writing line!")?;
    loop {
        let key = term
            .read_key()
            .with_context(|| "Error reading key!")?;
        match key {
            Key::Backspace => return Ok(None),
            Key::Char('p') => return Ok(Some(SortingMethod::Priority)),
            Key::Char('d') => return Ok(Some(SortingMethod::Due)),
            Key::Char('c') => return Ok(Some(SortingMethod::Created)),
            _ => continue
        }
    }
}

// This function requests a key and returns the sorting type we wish to use
pub fn wait_confirm_delete() -> Result<bool> {
    let term = Term::stdout();
    term.write_line("Confirm deletion? [y/N]")
        .with_context(|| "Error writing line!")?;
    loop {
        let key = term
            .read_key()
            .with_context(|| "Error reading key!")?;
        match key {
            Key::Char('y') => return Ok(true),
            Key::Enter | Key::Char('n') | Key::Char('N') => return Ok(false),
            _ => continue
        }
    }
}

// This function requests a key and returns the sorting type we wish to use
pub fn wait_key_event() -> Result<KeyEvent> {
    let term = Term::stdout();
    term.write_line("
e: edit     x: toggle read/unread\t
s: sort     \u{00B1}: change priority\t
z: delete   Backspace: go back
    ").with_context(|| "Error writing line!")?;
    loop {
        let key = term
            .read_key()
            .with_context(|| "Error reading key!")?;
        match key {
            Key::Backspace => return Ok(KeyEvent::Back),
            Key::Char('e') => return Ok(KeyEvent::Edit),
            Key::Char('s') => return Ok(KeyEvent::Sort),
            Key::Char('x') => return Ok(KeyEvent::ToggleRead),
            Key::Char('z') => return Ok(KeyEvent::Delete),
            Key::Char('+') => return Ok(KeyEvent::IncreasePriority),
            Key::Char('-') => return Ok(KeyEvent::DecreasePriority),
            Key::ArrowLeft => return Ok(KeyEvent::DecreaseProgress),
            Key::ArrowRight => return Ok(KeyEvent::IncreaseProgress),
            Key::ArrowUp => return Ok(KeyEvent::NavigateUp),
            Key::ArrowDown => return Ok(KeyEvent::NavigateDown),
            _ => continue
        }
    }
}

pub fn screen_navigate_todos(todos: &mut Vec<Todo>, position: usize) -> Result<Option<(usize, Action)>> {
    clear_term()?;
    hide_cursor()?;
    let size_todos = todos.len();
    for (idx,todo) in todos.iter().enumerate() {
        write_todo(todo, idx == position)?
    }
    let key_event = wait_key_event()?;
    match key_event {
        KeyEvent::Back => Ok(None),
        KeyEvent::Sort => {
            clear_lines(5)?;
            let sorting = wait_sort_key()?;
            match sorting {
                Some(SortingMethod::Created) => {
                    service::sort_todos_by_created_date_asc(todos);
                    screen_navigate_todos(todos, 0)
                },
                Some(SortingMethod::Due) => {
                    service::sort_todos_by_due_date_asc(todos);
                    screen_navigate_todos(todos, 0)
                },
                Some(SortingMethod::Priority) => {
                    service::sort_todos_by_priority_desc(todos);
                    screen_navigate_todos(todos, 0)
                },
                None => Ok(Some((position, Action::Reload)))
            }
        },
        KeyEvent::Delete => {
            clear_lines(1)?;
            let confirmation = wait_confirm_delete()?;
            match confirmation {
                true => Ok(Some((position, Action::Delete))),
                false => Ok(Some((position, Action::Reload)))
            }
        },
        KeyEvent::ToggleRead => Ok(Some((position, Action::ToggleRead))),
        KeyEvent::IncreasePriority => Ok(Some((position, Action::IncreasePriority))),
        KeyEvent::DecreasePriority => Ok(Some((position, Action::DecreasePriority))),
        KeyEvent::Edit => Ok(Some((position, Action::Edit))),
        KeyEvent::NavigateDown => screen_navigate_todos(todos, add_usize_module(position, size_todos)),
        KeyEvent::NavigateUp => screen_navigate_todos(todos, sub_usize_module(position, size_todos)),
        KeyEvent::IncreaseProgress => Ok(Some((position, Action::IncreaseProgress))),
        KeyEvent::DecreaseProgress => Ok(Some((position, Action::DecreaseProgress)))
    }
}

// Decrements usize without overflow
fn add_usize_module(x: usize, m: usize) -> usize {
    if x + 1 == m { 0 } else {x + 1}
}
// Increases usize without overflow
fn sub_usize_module(x: usize, m: usize) -> usize {
    if x == 0 {
        if m == 0 { 0 } else { m - 1 }
    } else {
        x - 1
    }
}