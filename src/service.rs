use polodb_core::Database;
use crate::{io, model::Todo, storage};
use log::*;
use anyhow::Result;
use std::cmp::Ordering::{self, Equal};

pub fn get_all_todos(db: &Database) -> Result<Option<Vec<Todo>>> {
    let todos = storage::get_todos(db)?;
    let mut todos_filtered = Vec::new();
    for todo in todos {
        match todo {
            Ok(todo_content) => todos_filtered.push(todo_content),
            Err(_) => {
                warn!("Error reading TODO, skipping...")
            }
        }
    }
    if todos_filtered.len() > 0 {
        Ok(Some(todos_filtered))
    } else {
        Ok(None)
    }
}

// Lists all TODOs
pub fn list_todos(db: &Database) -> Result<()> {
    let todos_filtered = get_all_todos(db)?;
    match todos_filtered {
        Some(mut v) => {
            sort_todos_by_created_date_asc(&mut v);
            io::write_todos(&mut v)?;
        },
        None => io::write_no_todos()?
    }
    Ok(())
}

// Deletes all todos which are also completed
pub fn delete_completed(db: &Database) -> Result<()> {
    let todos_filtered = get_all_todos(db)?;
    match todos_filtered {
        Some(todos) => {
            let todos_completed_iter = todos
                .iter()
                .filter(|todo| todo.is_complete());
            for todo in todos_completed_iter {
                storage::delete_todo(db, todo)?
            }
        },
        None => ()
    }
    Ok(())
}

// Returns none if string is empty
fn convert_empty_str_option(s: &str) -> Option<&str> {
    match s.is_empty() {
        false => Some(s),
        true => None
    }
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

// Shows all TODOs, ask for which to toggle read/unread
pub fn list_mark_read(db: &Database) -> Result<()> {
    let todos_filtered = get_all_todos(db)?;
    match todos_filtered {
        Some(todos) => {
            let titles: Vec<_> = todos
                .iter()
                .map(|x| io::get_title_complete(x)).collect();

            let selections = io::select_change_status(&titles)?;
            for selection in selections.iter() {
                storage::update_status(db, &todos[*selection])?;
            }
        },
        None => io::write_no_todos()?
    }
    Ok(())
}

// Shows all TODOs, ask for which to delete
pub fn list_delete(db: &Database) -> Result<()> {
    let todos_filtered = get_all_todos(db)?;
    match todos_filtered {
        Some(todos) => {
            let titles: Vec<_> = todos
                .iter()
                .map(|x| io::get_title_complete(x)).collect();

            let selections = io::select_change_status(&titles)?;
            for selection in selections.iter() {
                storage::delete_todo(db, &todos[*selection])?;
            }
        },
        None => io::write_no_todos()?
    }
    Ok(())
}

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

pub fn sort_todos_by_priority_desc(todos: &mut Vec<Todo>) {
    todos.sort_by(sort_by_priority)
} 

pub fn sort_todos_by_due_date_asc(todos: &mut Vec<Todo>) {
    todos.sort_by(sort_by_due_date)
} 

pub fn sort_todos_by_created_date_asc(todos: &mut Vec<Todo>) {
    todos.sort_by(sort_by_created_date)
} 