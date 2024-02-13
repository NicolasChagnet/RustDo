use polodb_core::{ClientCursor, Database};
use crate::{get_completed_todos, io, model::{Action, Todo}, storage, md_utils::export_to_md};
use log::*;
use anyhow::Result;
use std::cmp::Ordering::{self, Equal};

// This function filters out errors when reading todos and returns the todos
pub fn filter_error_todos(todos: ClientCursor<Todo>) -> Result<Vec<Todo>> {
    let mut todos_filtered = Vec::new();
    for todo in todos {
        match todo {
            Ok(todo_content) => todos_filtered.push(todo_content),
            Err(_) => {
                warn!("Error reading TODO, skipping...")
            }
        }
    }
    Ok(todos_filtered)
    // if !todos_filtered.is_empty() {
    //     Ok(Some(todos_filtered))
    // } else {
    //     Ok(None)
    // }
}

// Queries the DB, returns all TODO objects (errors in reading the DB only lead to a warning)
pub fn get_all_todos(db: &Database) -> Result<Vec<Todo>> {
    let todos = storage::get_todos(db)?;
    filter_error_todos(todos)
}

// This function is the main TODO listing screen
// Handles various actions on selected individual TODO elements
pub fn navigate_todos(db: &Database, start_position: usize) -> Result<()> {
    let mut pos = start_position;
    loop {
        let mut todos = get_all_todos(db)?; // Gets TODOs from DB
        sort_todos_by_due_date_asc(&mut todos);
        let navigation = io::screen_navigate_todos(&mut todos, pos)?;
        if let Some((p, action)) = navigation {
            pos = p;
            match action {
                // Each action is directed to the correct DB action
                Action::Add => {
                    io::show_cursor()?;
                    io::clear_term()?;
                    debug!("(Service) Adding TODO");
                    add_todo(db)?;
                },
                Action::Edit => {
                    io::show_cursor()?;
                    io::clear_term()?;
                    if !todos.is_empty() {
                        debug!("(Service) Editing TODO {:?}", &todos[p]);
                        edit_todo(db, &todos[p])?;
                    }
                },
                Action::ToggleRead => {
                    if !todos.is_empty() {
                        debug!("(Service) Toggle read TODO {:?}", &todos[p]);
                        storage::toggle_read(db, &todos[p])?;
                    }
                },
                Action::IncreasePriority => {
                    if !todos.is_empty() {
                        debug!("(Service) Up prio TODO {:?}", &todos[p]);
                        storage::increase_priority(db, &todos[p])?;
                    }
                },
                Action::DecreasePriority => {
                    if !todos.is_empty() {
                        debug!("(Service) Down prio TODO {:?}", &todos[p]);
                        storage::decrease_priority(db, &todos[p])?;
                    }
                },
                Action::IncreaseProgress => {
                    if !todos.is_empty() {
                        storage::increase_progress(db, &todos[p])?;
                    }
                },
                Action::DecreaseProgress => {
                    if !todos.is_empty() {
                        storage::decrease_progress(db, &todos[p])?;
                    }
                },
                Action::Delete => {
                    if !todos.is_empty() {
                        debug!("(Service) Deleting TODO {:?}", &todos[p]);
                        storage::delete_todo(db, &todos[p])?;
                    }
                },
                Action::DeleteCompleted => {
                    if !todos.is_empty() {
                        debug!("(Service) Deleting all completed");
                        delete_completed(db)?;
                    }
                },
                Action::Export => export_todos(db)?,
                Action::Reload => ()
            }
            continue;
        }
        break;
    }
    Ok(())
}

pub fn delete_completed(db: &Database) -> Result<()> {
    let todos = get_completed_todos(db)?; // Loads all TODOs
    let todos_filtered = filter_error_todos(todos)?;
    // if let Some(todos) = todos_filtered {
    //     // Deletes TODO in DB
    //     for todo in todos {
    //         storage::delete_todo(db, &todo)?
    //     }
    // }
    for todo in todos_filtered {
        storage::delete_todo(db, &todo)?
    }
    Ok(())
}

// Handles the TODO addition page
// Returns early to menu if no title is set
pub fn add_todo(db: &Database) -> Result<()> {
    let title = io::input_title(None)?;
    if title.is_empty() {
        return Ok(());
    }
    let due_date_str = io::input_due_date(None)?;
    let priority = io::input_priority(0)?;

    let due_date = convert_empty_str_option(&due_date_str);

    let todo = Todo::new( &title, priority, due_date);
    storage::insert_todo(db, todo)?;
    Ok(())
}

// Handles the TODO edition page
// Returns early to menu if no title is set
pub fn edit_todo(db: &Database, todo: &Todo) -> Result<()> {
    let title = io::input_title(Some(todo.get_title()))?;
    if title.is_empty() {
        return Ok(());
    }
    let due_date_str = io::input_due_date(todo.get_due_date())?;
    let priority = io::input_priority(todo.get_priority() as usize)?;

    let due_date = convert_empty_str_option(&due_date_str);

    let mut todo_replace = Todo::new( &title, priority, due_date);
    todo_replace.set_id(todo.get_id());
    storage::update_todo(db, &todo_replace)?;
    Ok(())
} 
// Export all TODOs in database
pub fn export_todos(db: &Database) -> Result<()> {
    let todos_list= get_all_todos(db)?;
    export_to_md(&todos_list)?;
    // if let Some(todos) = todos_list {
    //     export_to_md(&todos)?;
    // }
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