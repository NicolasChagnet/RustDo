use crate::{
    get_completed_todos, io,
    md_utils::export_to_md,
    model::{Action, MyDate, SortingMethod, Todo, TodoCollection},
    storage,
};
use anyhow::Result;
use std::cmp::Ordering::{self, Equal};
use std::env;

// Extracts the TODO collection from the vector of tuples.
pub fn get_todo_tuple(todos_tup: Vec<(String, Todo)>) -> TodoCollection {
    let (_, vec_todos): (Vec<_>, Vec<_>) = todos_tup.into_iter().unzip();
    vec_todos
}

// This function is the main TODO listing screen
// Handles various actions on selected individual TODO elements
pub fn navigate_todos(
    db: &mut storage::DatabaseModel,
    start_position: usize,
    mut sorting_method: SortingMethod,
) -> Result<()> {
    let mut pos = start_position; // Starting position of the arrow
    loop {
        let todos_db = storage::get_todos(db)?; // Gets TODOs from DB
        let mut todos = get_todo_tuple(todos_db);
        // Sorts the TODO collection
        match sorting_method {
            SortingMethod::Priority => sort_todos_by_priority_desc(&mut todos),
            SortingMethod::Due => sort_todos_by_due_date_asc(&mut todos),
            SortingMethod::Created => sort_todos_by_created_date_asc(&mut todos),
        }
        // Reads action from user
        let navigation = io::screen_navigate_todos(&mut todos, pos)?;
        if let Some((p, action)) = navigation {
            pos = p; // Update the position variable to where the use executed the action
            match action {
                // Each action calls the correct DB action
                Action::Add => {
                    io::show_cursor()?;
                    io::clear_term()?;
                    add_todo(db)?;
                }
                Action::Edit => {
                    io::show_cursor()?;
                    io::clear_term()?;
                    if !todos.is_empty() {
                        edit_todo(db, &todos[p])?;
                    }
                }
                Action::ToggleRead => {
                    if !todos.is_empty() {
                        todos[p].toggle_read(); //Mark TODO read
                        storage::update_todo(db, &todos[p])?;
                    }
                }
                Action::IncreasePriority => {
                    if !todos.is_empty() {
                        todos[p].increase_priority(); //Mark TODO read
                        storage::update_todo(db, &todos[p])?;
                    }
                }
                Action::DecreasePriority => {
                    if !todos.is_empty() {
                        todos[p].decrease_priority(); //Mark TODO read
                        storage::update_todo(db, &todos[p])?;
                    }
                }
                Action::IncreaseProgress => {
                    if !todos.is_empty() {
                        todos[p].increase_progress(); //Mark TODO read
                        storage::update_todo(db, &todos[p])?;
                    }
                }
                Action::DecreaseProgress => {
                    if !todos.is_empty() {
                        todos[p].decrease_progress(); //Mark TODO read
                        storage::update_todo(db, &todos[p])?;
                    }
                }
                Action::Delete => {
                    if !todos.is_empty() {
                        storage::delete_todo(db, &todos[p])?;
                    }
                }
                Action::DeleteCompleted => {
                    if !todos.is_empty() {
                        delete_completed(db)?;
                    }
                }
                Action::Sort(new_sort_method) => {
                    sorting_method = new_sort_method;
                }
                Action::Export => export_to_md(&todos)?,
                Action::Reload => (),
            }
            continue;
        }
        // If the navigation is None, this means exit the loop
        // First we export to the markdown file
        if env::var("EXPORT_ON_EXIT")
            .unwrap_or("false".to_string())
            .parse::<bool>()
            .unwrap_or(false)
        {
            export_to_md(&todos)?;
        }
        break;
    }
    Ok(())
}

// Deletes all completed TODOs
pub fn delete_completed(db: &mut storage::DatabaseModel) -> Result<()> {
    let todos_db = get_completed_todos(db)?; // Loads all completed TODOs
    let todos = get_todo_tuple(todos_db); // Extract the TodoCollection
    for todo in todos {
        storage::delete_todo(db, &todo)? // Delete each todo
    }
    Ok(())
}

// Handles the TODO addition page
// Returns early to menu if no title is set
pub fn add_todo(db: &mut storage::DatabaseModel) -> Result<()> {
    let title = io::input_title(None)?; // Prompts title
    if title.is_empty() {
        return Ok(()); // Early return
    }
    let due_date_str = io::input_due_date(&None)?; // Prompts due date
    let priority = io::input_priority(0)?; // Prompts for priority level

    let due_date = convert_empty_str_option(&due_date_str); // Converts due date
    let todo = Todo::new(&title, priority, due_date); // Create new TODO element

    storage::insert_todo(db, &todo)?; // Add to DB
    Ok(())
}

// Handles the TODO edition page
// Returns early to menu if no title is set
// Similar to add_todo but with presets set by the existing TODO
pub fn edit_todo(db: &mut storage::DatabaseModel, todo: &Todo) -> Result<()> {
    let title = io::input_title(Some(todo.get_title()))?;
    if title.is_empty() {
        return Ok(());
    }
    let due_date_str = io::input_due_date(todo.get_due_date())?;
    let priority = io::input_priority(todo.get_priority() as usize)?;

    let due_date = convert_empty_str_option(&due_date_str);
    let mut todo_replace = Todo::new(&title, priority, due_date);
    todo_replace.set_id(todo.get_id());

    storage::update_todo(db, &todo_replace)?;
    Ok(())
}

// Custom comparisons between TODO elements by priority
// 1. Incomplete before all complete TODOs
// 2. In case of equality sort by priority ordering, descending
// 3. Defaults back to created date as a last resort
fn sort_by_priority(l: &Todo, r: &Todo) -> Ordering {
    let compare_complete = l.is_complete().cmp(&r.is_complete());
    if compare_complete == Equal {
        let compare_priority = l.get_priority().cmp(&r.get_priority()).reverse();
        if compare_priority == Equal {
            l.get_created_date()
                .get_0()
                .cmp(&r.get_created_date().get_0())
                .reverse()
        } else {
            compare_priority
        }
    } else {
        compare_complete
    }
}

// Custom comparisons between TODO elements by due date
// 1. Incomplete before all complete TODOs
// 2. In case of equality sort by due date, ascending, None go last
// 3. If no due dates are set, default back to created date
fn sort_by_due_date(l: &Todo, r: &Todo) -> Ordering {
    let compare_complete = l.is_complete().cmp(&r.is_complete());
    if compare_complete == Equal {
        let ld_o = l.get_due_date();
        let rd_o = r.get_due_date();
        match (ld_o, rd_o) {
            (Some(MyDate(ld)), Some(MyDate(rd))) => ld.cmp(rd),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => l
                .get_created_date()
                .get_0()
                .cmp(&r.get_created_date().get_0())
                .reverse(),
        }
    } else {
        compare_complete
    }
}

// Custom comparisons between TODO elements by created date
// 1. Incomplete before all complete TODOs
// 2. In case of equality sort by created date, descending
fn sort_by_created_date(l: &Todo, r: &Todo) -> Ordering {
    let compare_complete = l.is_complete().cmp(&r.is_complete());
    if compare_complete == Equal {
        l.get_created_date()
            .get_0()
            .cmp(&r.get_created_date().get_0())
            .reverse()
    } else {
        compare_complete
    }
}

// Functions to sort TODO collections by applying the appropriate sorting function
pub fn sort_todos_by_priority_desc(todos: &mut [Todo]) {
    todos.sort_by(sort_by_priority)
}

pub fn sort_todos_by_due_date_asc(todos: &mut [Todo]) {
    todos.sort_by(sort_by_due_date)
}

pub fn sort_todos_by_created_date_asc(todos: &mut [Todo]) {
    todos.sort_by(sort_by_created_date)
}

// Wraps string with Some, None if the string is empty
fn convert_empty_str_option(s: &str) -> Option<&str> {
    match s.is_empty() {
        false => Some(s),
        true => None,
    }
}
