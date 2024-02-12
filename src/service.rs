use polodb_core::{ClientCursor, Database};
use crate::{get_completed_todos, io, model::{Action, Todo}, storage};
use log::*;
use anyhow::Result;
use std::cmp::Ordering::{self, Equal};

// This function filters out errors when reading todos and returns the todos
pub fn filter_error_todos(todos: ClientCursor<Todo>) -> Result<Option<Vec<Todo>>> {
    let mut todos_filtered = Vec::new();
    for todo in todos {
        match todo {
            Ok(todo_content) => todos_filtered.push(todo_content),
            Err(_) => {
                warn!("Error reading TODO, skipping...")
            }
        }
    }
    if !todos_filtered.is_empty() {
        Ok(Some(todos_filtered))
    } else {
        Ok(None)
    }
}

// Queries the DB, returns all TODO objects (errors in reading the DB only lead to a warning)
pub fn get_all_todos(db: &Database) -> Result<Option<Vec<Todo>>> {
    let todos = storage::get_todos(db)?;
    filter_error_todos(todos)
}

// This function is the main TODO listing screen
// Handles various actions on selected individual TODO elements
pub fn navigate_todos(db: &Database, start_position: usize) -> Result<()> {
    let todos_filtered = get_all_todos(db)?; // Gets TODOs from DB
    match todos_filtered {
        Some(mut todos) => {
            sort_todos_by_due_date_asc(&mut todos); // Sorts TODOs by due date
            // This shows the navigation screen and recovers an action to execute
            let navigation = io::screen_navigate_todos(&mut todos, start_position)?;
            if let Some((p, action)) = navigation {
                match action {
                    // Each action is directed to the correct DB action
                    Action::ToggleRead => {
                        storage::toggle_read(db, &todos[p])?;
                    },
                    Action::IncreasePriority => {
                        storage::increase_priority(db, &todos[p])?;
                    },
                    Action::DecreasePriority => {
                        storage::decrease_priority(db, &todos[p])?;
                    },
                    Action::IncreaseProgress => {
                        storage::increase_progress(db, &todos[p])?;
                    },
                    Action::DecreaseProgress => {
                        storage::decrease_progress(db, &todos[p])?;
                    },
                    Action::Delete => {
                        storage::delete_todo(db, &todos[p])?;
                    },
                    Action::Reload => ()
                }
                // We then reload the screen
                navigate_todos(db, p)?;
            }
        },
        None => io::write_no_todos()? // If no TODOs are in DB, we print a simpler screen
    }
    Ok(())
}

pub fn delete_completed(db: &Database) -> Result<()> {
    let todos = get_completed_todos(db)?; // Loads all TODOs
    let todos_filtered = filter_error_todos(todos)?;
    if let Some(todos) = todos_filtered {
        // Deletes TODO in DB
        for todo in todos {
            storage::delete_todo(db, &todo)?
        }
    }
    Ok(())
}

// Handles the TODO addition page
// Returns early to menu if no title is set
pub fn add_todo(db: &Database) -> Result<()> {
    let title = io::input_title()?;
    if title.is_empty() {
        return Ok(());
    }
    let due_date_str = io::input_due_date()?;
    let priority = io::input_priority()?;

    let due_date = convert_empty_str_option(&due_date_str);

    let todo = Todo::new( &title, priority, due_date);
    storage::insert_todo(db, todo)?;
    Ok(())
} 

// Custom comparisons between TODO elements depending on various criteria
fn sort_by_priority(l: &Todo, r: &Todo) -> Ordering {
    let compare_complete = l.is_complete().cmp(&r.is_complete());
    if compare_complete == Equal {
        let compare_priority = l.get_priority().cmp(&r.get_priority()).reverse();
        if compare_priority == Equal {
            l.get_created_date().cmp(&r.get_created_date())
        } else {
            compare_priority
        }
    } else {
        compare_complete
    }
}

fn sort_by_due_date(l: &Todo, r: &Todo) -> Ordering {
    let compare_complete = l.is_complete().cmp(&r.is_complete());
    if compare_complete == Equal {
        let ld_o = l.get_due_date();
        let rd_o = r.get_due_date();
        match (ld_o, rd_o) {
            (Some(ld), Some(rd)) => ld.cmp(&rd),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => l.get_created_date().cmp(&r.get_created_date())
        }
    } else {
        compare_complete
    }
}

fn sort_by_created_date(l: &Todo, r: &Todo) -> Ordering {
    let compare_complete = l.is_complete().cmp(&r.is_complete());
    if compare_complete == Equal {
        l.get_created_date().cmp(&r.get_created_date())
    } else {
        compare_complete
    }
}

// Functions to sort TODO collections
pub fn sort_todos_by_priority_desc(todos: &mut [Todo]) {
    todos.sort_by(sort_by_priority)
} 

pub fn sort_todos_by_due_date_asc(todos: &mut [Todo]) {
    todos.sort_by(sort_by_due_date)
} 

pub fn sort_todos_by_created_date_asc(todos: &mut [Todo]) {
    todos.sort_by(sort_by_created_date)
} 

// Returns none if string is empty
fn convert_empty_str_option(s: &str) -> Option<&str> {
    match s.is_empty() {
        false => Some(s),
        true => None
    }
}