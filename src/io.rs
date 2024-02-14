use crate::{
    date_utils::{validate_regex, FORMAT_DATE},
    model::{Action, KeyEvent, MyDate, SortingMethod, Todo},
    Progress, MAXPRIORITY,
};
use anyhow::{Context, Result};
use chrono::Local;
use console::{style, Key, StyledObject, Term};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::cmp::Ordering;

// Menu constant
const MENU: &str = "
-------------------------------------------------------------------------------------------
a: add      m: export to markdown\t
e: edit     x: toggle read/unread\t
s: sort     \u{00B1}: change priority\t
z: delete   Z: delete all completed\t
\u{21B5}: exit     \u{023f4}\u{023f5}: change progress";
const NERASE: usize = 5;

// Prompts user for title
pub fn input_title(prewrite: Option<&str>) -> Result<String> {
    let input = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Title (leave empty to go back): ")
        .with_initial_text(prewrite.unwrap_or("").to_string())
        .allow_empty(true)
        .interact_text()
        .with_context(|| "Error reading input!")?;
    Ok(input)
}

// Prompts user for due date
pub fn input_due_date(prewrite: &Option<MyDate>) -> Result<String> {
    let prewrite_str = match prewrite {
        Some(MyDate(date)) => date.format(FORMAT_DATE).to_string(),
        None => "".to_string(),
    };
    let input = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Due date [dd-mm(-YYYY)]: ")
        .allow_empty(true)
        .with_initial_text(prewrite_str)
        .validate_with(validate_regex)
        .interact_text()
        .with_context(|| "Error reading text!")?;
    Ok(input.to_lowercase())
}

// Prompts user for priority level
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

// Parses the title, strikethrough if complete
pub fn get_title_complete(todo: &Todo) -> StyledObject<&str> {
    let style_base = style(todo.get_title());
    match todo.is_complete() {
        true => style_base.strikethrough(),
        false => style_base,
    }
}

// Parses due date with the appropriate color depending on today's date
// Green -- future due date, Orange -- today is the due date, Red -- passed due date
// Dimmed gray color for completed TODOs
pub fn get_due_date(todo: &Todo) -> Option<StyledObject<String>> {
    let today = Local::now().date_naive();
    todo.get_due_date().as_ref().map_or_else(
        || None,
        |due| {
            let base = style(due.get_0().format(FORMAT_DATE).to_string());
            if todo.is_complete() {
                return Some(base.dim());
            }
            match due.get_0().cmp(&today) {
                Ordering::Greater => Some(base.green()),
                Ordering::Equal => Some(base.yellow()),
                Ordering::Less => Some(base.red()),
            }
        },
    )
}

// Converts progress status to string progress bar
pub fn get_progress_str(todo: &Todo) -> String {
    let ret_str = if todo.is_complete() {
        "[########]"
    } else {
        match todo.get_progress() {
            Progress::Zero => "[        ]",
            Progress::Quarter => "[##      ]",
            Progress::Half => "[####    ]",
            Progress::ThreeQuarter => "[######  ]",
            Progress::Full => "[########]",
        }
    };
    ret_str.to_string()
}

// Converts priority number to priority level string
pub fn get_priority_symbol(p: u32) -> String {
    match p {
        0 => "_".to_string(),
        _ => (1..=p).map(|_| "!").collect::<String>(),
    }
}

// Given a TODO element, prints the TODO onscreen
pub fn write_todo(todo: &Todo, is_position: bool) -> Result<()> {
    let term = Term::stdout();
    let title = get_title_complete(todo);
    let date = get_due_date(todo);
    let priority = get_priority_symbol(todo.get_priority());
    let initial_character = match is_position {
        true => ">",
        false => " ",
    };
    let progress_bar = get_progress_str(todo);
    let str_write = match date {
        Some(date_str) => format!(
            "{} {} {} - Due: {} - Progress: {}",
            initial_character, priority, title, date_str, progress_bar
        ),
        None => format!(
            "{} {} {} - Progress: {}",
            initial_character, priority, title, progress_bar
        ),
    };
    term.write_line(&str_write)
        .with_context(|| "Error while writing line!")?;
    Ok(())
}

// Clears terminal
pub fn clear_term() -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()
        .with_context(|| "Error clearing screen!")?;
    Ok(())
}

// Clears last n lines on terminal
pub fn clear_menu() -> Result<()> {
    let term = Term::stdout();
    term.clear_last_lines(NERASE)
        .with_context(|| "Error clearing screen!")?;
    Ok(())
}

// Show cursor
pub fn show_cursor() -> Result<()> {
    let term = Term::stdout();
    term.show_cursor()
        .with_context(|| "Error showing cursor!")?;
    Ok(())
}

// Hide cursor
pub fn hide_cursor() -> Result<()> {
    let term = Term::stdout();
    term.hide_cursor()
        .with_context(|| "Error showing cursor!")?;
    Ok(())
}

// This function prompts the user for input on sorthing methods
pub fn wait_sort_key() -> Result<Option<SortingMethod>> {
    let term = Term::stdout();
    term.write_line(
        "p: sort by priority   c: sort by date of creation\t
d: sort by due date   Backspace: go back
        ",
    )
    .with_context(|| "Error writing line!")?;
    loop {
        let key = term.read_key().with_context(|| "Error reading key!")?;
        match key {
            Key::Backspace => return Ok(None),
            Key::Char('p') => return Ok(Some(SortingMethod::Priority)),
            Key::Char('d') => return Ok(Some(SortingMethod::Due)),
            Key::Char('c') => return Ok(Some(SortingMethod::Created)),
            _ => continue,
        }
    }
}

// This function prompts the user for confirmation
pub fn wait_confirm(message: &str) -> Result<bool> {
    let term = Term::stdout();
    term.write_line(format!("{} [y/N]", message).as_str())
        .with_context(|| "Error writing line!")?;
    loop {
        let key = term.read_key().with_context(|| "Error reading key!")?;
        match key {
            Key::Char('y') => return Ok(true),
            Key::Enter | Key::Char('n') | Key::Char('N') => return Ok(false),
            _ => continue,
        }
    }
}

fn confirm_message(message: &str) -> Result<()> {
    let term = Term::stdout();
    term.write_line(message)?;
    Ok(())
}

// This function prompts the user for input on actions to take
pub fn wait_key_event() -> Result<KeyEvent> {
    let term = Term::stdout();
    term.write_line(MENU)
        .with_context(|| "Error writing line!")?;
    loop {
        let key = term.read_key().with_context(|| "Error reading key!")?;
        match key {
            Key::Enter => return Ok(KeyEvent::Back),
            Key::Char('m') => return Ok(KeyEvent::Export),
            Key::Char('a') => return Ok(KeyEvent::Add),
            Key::Char('e') => return Ok(KeyEvent::Edit),
            Key::Char('s') => return Ok(KeyEvent::Sort),
            Key::Char('x') => return Ok(KeyEvent::ToggleRead),
            Key::Char('z') => return Ok(KeyEvent::Delete),
            Key::Char('+') => return Ok(KeyEvent::IncreasePriority),
            Key::Char('-') => return Ok(KeyEvent::DecreasePriority),
            Key::Char('Z') => return Ok(KeyEvent::DeleteCompleted),
            Key::ArrowLeft => return Ok(KeyEvent::DecreaseProgress),
            Key::ArrowRight => return Ok(KeyEvent::IncreaseProgress),
            Key::ArrowUp => return Ok(KeyEvent::NavigateUp),
            Key::ArrowDown => return Ok(KeyEvent::NavigateDown),
            _ => continue,
        }
    }
}

// Main printing function, prints TODOs, menu and handles simple actions, otherwise returns to service
pub fn screen_navigate_todos(
    todos: &mut Vec<Todo>,
    position: usize,
) -> Result<Option<(usize, Action)>> {
    clear_term()?;
    hide_cursor()?;
    let size_todos = todos.len();
    let pos_fixed = get_pos_overflow(position, size_todos);
    for (idx, todo) in todos.iter().enumerate() {
        write_todo(todo, idx == pos_fixed)?
    }
    let key_event = wait_key_event()?;
    match key_event {
        KeyEvent::Back => {
            show_cursor()?;
            Ok(None)
        }
        KeyEvent::Sort => {
            clear_menu()?;
            let sorting = wait_sort_key()?;
            match sorting {
                Some(sort) => Ok(Some((0, Action::Sort(sort)))),
                None => Ok(Some((pos_fixed, Action::Reload))),
            }
        }
        KeyEvent::Delete => {
            clear_menu()?;
            let confirmation = wait_confirm("Confirm deletion?")?;
            match confirmation {
                true => Ok(Some((pos_fixed, Action::Delete))),
                false => Ok(Some((pos_fixed, Action::Reload))),
            }
        }
        KeyEvent::DeleteCompleted => {
            clear_menu()?;
            let confirmation = wait_confirm("Confirm deletions?")?;
            match confirmation {
                true => Ok(Some((pos_fixed, Action::DeleteCompleted))),
                false => Ok(Some((pos_fixed, Action::Reload))),
            }
        }
        KeyEvent::ToggleRead => Ok(Some((pos_fixed, Action::ToggleRead))),
        KeyEvent::IncreasePriority => Ok(Some((pos_fixed, Action::IncreasePriority))),
        KeyEvent::DecreasePriority => Ok(Some((pos_fixed, Action::DecreasePriority))),
        KeyEvent::Edit => Ok(Some((pos_fixed, Action::Edit))),
        KeyEvent::Add => Ok(Some((pos_fixed, Action::Add))),
        KeyEvent::NavigateDown => {
            screen_navigate_todos(todos, add_usize_module(pos_fixed, size_todos))
        }
        KeyEvent::NavigateUp => {
            screen_navigate_todos(todos, sub_usize_module(pos_fixed, size_todos))
        }
        KeyEvent::IncreaseProgress => Ok(Some((pos_fixed, Action::IncreaseProgress))),
        KeyEvent::DecreaseProgress => Ok(Some((pos_fixed, Action::DecreaseProgress))),
        KeyEvent::Export => {
            clear_menu()?;
            confirm_message("Exporting...")?;
            std::thread::sleep(std::time::Duration::from_millis(400)); // Wait for the message to be visible
            Ok(Some((pos_fixed, Action::Export)))
        }
    }
}

// Decrements usize without overflow
fn add_usize_module(x: usize, m: usize) -> usize {
    if x + 1 == m {
        0
    } else {
        x + 1
    }
}
// Increases usize without overflow
fn sub_usize_module(x: usize, m: usize) -> usize {
    if x == 0 {
        if m == 0 {
            0
        } else {
            m - 1
        }
    } else {
        x - 1
    }
}

fn get_pos_overflow(p: usize, m: usize) -> usize {
    if m == 0 {
        return 0;
    }
    if p >= m {
        return m - 1;
    }
    p
}
