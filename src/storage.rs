use std::env;
// use polodb_core::{bson::doc, bson::to_document, ClientCursor, Collection, Database};
// use serde::Serialize;
use jasondb::*;
use crate::model::Todo;
use anyhow::{Result, Context};
use log::*;

pub type DatabaseModel = Database<Todo>;

// DB connection function
pub fn connect_db() -> Result<DatabaseModel> {
    let filename = env::var("DB_FILE").expect("Error loading DB filename from environment variables!");
    let db: DatabaseModel = Database::new(filename)
        .with_context(|| "Error opening database!")?;
    Ok(db)
}


// Queries the DB for all TODOs
pub fn get_todos(db: &mut DatabaseModel) -> Result<Vec<(String, Todo)>> {
    let query = db.iter()
        .filter_map(|x| x.ok())
        .collect::<Vec<(String, Todo)>>();
    Ok(query)
}

// Queries the DB for all completed TODOs
pub fn get_completed_todos(db: &mut DatabaseModel) -> Result<Vec<(String, Todo)>>{
    // let collection: Collection<Todo> = get_collection(db)?;
    let query: Vec<(String, Todo)> = db.query(query!(completed == true))?
        .filter_map(|x| x.ok())
        .collect::<Vec<(String, Todo)>>();
    Ok(query)
}

// Inserts a TODO object inside the DB
pub fn insert_todo(db: &mut DatabaseModel, todo: &Todo) -> Result<()> {
    db
        .set(todo.get_id(), todo)
        .with_context(|| "Error inserting TODO inside DB!")?;
    Ok(())
}

// Updates the DB element associated with a TODO object
pub fn update_todo(db: &mut DatabaseModel, todo_replace: &Todo) -> Result<()> {
     db.set(todo_replace.get_id(), todo_replace).with_context(|| "Error updating the entry!")?;
     Ok(())
 }


// Deletes a TODO object from DB
pub fn delete_todo(db: &mut DatabaseModel, todo: &Todo) -> Result<()> {
    debug!("(Storage) Deleting TODO {:?}", todo);
    // let collection: Collection<Todo> = get_collection(db)?;
    db.delete(todo.get_id()).with_context(|| "Error updating the entry!")?;
    Ok(())
}