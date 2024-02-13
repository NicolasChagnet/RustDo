use std::env;
use polodb_core::{bson::doc, bson::to_document, ClientCursor, Collection, Database};
// use serde::Serialize;
use crate::model::{Todo, MAXPRIORITY};
use anyhow::{Result, Context};
use log::*;

// DB connection function
pub fn connect_db() -> Result<Database> {
    let filename = env::var("DB_FILE").expect("Error loading DB filename from environment variables!");
    let db = Database::open_file(filename)
        .with_context(|| "Error opening database!")?;
    Ok(db)
}

// Get the main TODO collection
pub fn get_collection(db: &Database) -> Result<Collection<Todo>> {
    let collection_name = env::var("DB_COLLECTION")
        .with_context(|| "Error loading the collection filename from environment variables!")?;
    Ok(db.collection::<Todo>(&collection_name))
}

// Queries the DB for all TODOs
pub fn get_todos(db: &Database) -> Result<ClientCursor<Todo>> {
    let collection: Collection<Todo> = get_collection(db)?;
    let query = collection.find(None)
        .with_context(|| "Error fetching the database!")?;
    Ok(query)
}

// Queries the DB for all TODOs
pub fn get_completed_todos(db: &Database) -> Result<ClientCursor<Todo>> {
    let collection: Collection<Todo> = get_collection(db)?;
    let query = collection.find(doc! {
        "completed": true
    })
        .with_context(|| "Error fetching the database!")?;
    Ok(query)
}

// Inserts a TODO object inside the DB
pub fn insert_todo(db: &Database, todo: Todo) -> Result<()> {
    let collection: Collection<Todo> = get_collection(db)?;
    collection
        .insert_one(todo)
        .with_context(|| "Error inserting TODO inside DB!")?;
    Ok(())
}

// Changes the completed status of a DB TODO objects
pub fn toggle_read(db: &Database, todo: &Todo) -> Result<()> {
    let collection: Collection<Todo> = get_collection(db)?;
    collection.update_one(doc! {
        "id": todo.get_id().to_string()
    }, doc! {
        "$set": doc! {
            "completed": !todo.is_complete()
        }
    }).with_context(|| "Error updating the entry!")?;
    Ok(())
}

// Changes the completed status of a DB TODO objects
pub fn update_todo(db: &Database, todo_replace: &Todo) -> Result<()> {
    let collection: Collection<Todo> = get_collection(db)?;
    let doc = to_document(todo_replace)?;
    collection.update_one(doc! {
        "id": todo_replace.get_id().to_string()
    }, doc! {
        "$set": doc
    }).with_context(|| "Error updating the entry!")?;
    Ok(())
}

// Various DB update functions to increase/decrease the priority or progress
pub fn increase_priority(db: &Database, todo: &Todo) -> Result<()> {
    let collection: Collection<Todo> = get_collection(db)?;
    collection.update_one(doc! {
        "id": todo.get_id().to_string()
    }, doc! {
        "$set": doc! {
            "priority": std::cmp::min(todo.get_priority() + 1, MAXPRIORITY)
        }
    }).with_context(|| "Error updating the entry!")?;
    Ok(())
}

pub fn decrease_priority(db: &Database, todo: &Todo) -> Result<()> {
    let collection: Collection<Todo> = get_collection(db)?;
    collection.update_one(doc! {
        "id": todo.get_id().to_string()
    }, doc! {
        "$set": doc! {
            "priority": std::cmp::max(todo.get_priority() as i32 - 1, 0) as u32
        }
    }).with_context(|| "Error updating the entry!")?;
    Ok(())
}

pub fn increase_progress(db: &Database, todo: &Todo) -> Result<()> {
    let collection: Collection<Todo> = get_collection(db)?;
    collection.update_one(doc! {
        "id": todo.get_id().to_string()
    }, doc! {
        "$set": doc! {
            "progress": serde_json::to_string(&todo.get_progress().up())?.parse::<u32>()?
        }
    }).with_context(|| "Error updating the entry!")?;
    Ok(())
}

pub fn decrease_progress(db: &Database, todo: &Todo) -> Result<()> {
    let collection: Collection<Todo> = get_collection(db)?;
    collection.update_one(doc! {
        "id": todo.get_id().to_string()
    }, doc! {
        "$set": doc! {
            "progress": serde_json::to_string(&todo.get_progress().down())?.parse::<u32>()?
        }
    }).with_context(|| "Error updating the entry!")?;
    Ok(())
}

// Deletes a TODO object from DB
pub fn delete_todo(db: &Database, todo: &Todo) -> Result<()> {
    debug!("(Storage) Deleting TODO {:?}", todo);
    let collection: Collection<Todo> = get_collection(db)?;
    let deletion = collection.delete_one(doc! {
        "id": todo.get_id().to_string()
    }).with_context(|| "Error updating the entry!")?;
    debug!("(Storage) Deleted TODO {:?}", deletion);
    Ok(())
}