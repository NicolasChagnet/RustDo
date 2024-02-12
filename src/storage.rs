use std::env;
use polodb_core::{bson::doc, ClientCursor, Collection, Database};
use crate::model::{Todo, MAXPRIORITY};
use rand::{distributions::Alphanumeric, Rng};
use anyhow::{Result, Context};

const NSTR: usize = 6;


pub fn gen_random_string() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(NSTR)
        .map(char::from)
        .collect()
}

pub fn get_new_index(db: &Database) -> Result<String> {
    let collection = get_collection(db)?;
    let mut random_str = gen_random_string();
    let mut query = collection.find_one(doc! {
        "id": random_str.clone()
    }).with_context(|| "Error fetching the database!")?;
    loop {
        match query {
            None => return Ok(random_str),
            Some(_)=> {
                random_str = gen_random_string();
                query = collection.find_one(doc! {
                    "id": random_str.clone()
                }).with_context(|| "Error fetching the database!")?;
            }
        }
    }
}

pub fn connect_db() -> Result<Database> {
    let filename = env::var("DB_FILE").expect("Error loading DB filename from environment variables!");
    let db = Database::open_file(filename)
        .with_context(|| "Error opening database!")?;
    Ok(db)
}

pub fn get_collection(db: &Database) -> Result<Collection<Todo>> {
    let collection_name = env::var("DB_COLLECTION")
        .with_context(|| "Error loading the collection filename from environment variables!")?;
    Ok(db.collection::<Todo>(&collection_name))
}

pub fn get_todos(db: &Database) -> Result<ClientCursor<Todo>> {
    let collection: Collection<Todo> = get_collection(db)?;
    let query = collection.find(None)
        .with_context(|| "Error fetching the database!")?;
    Ok(query)
}

pub fn insert_todo(db: &Database, todo: Todo) -> Result<()> {
    let collection: Collection<Todo> = get_collection(db)?;
    collection
        .insert_one(todo)
        .with_context(|| "Error inserting TODO inside DB!")?;
    Ok(())
}

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

pub fn delete_todo(db: &Database, todo: &Todo) -> Result<()> {
    let collection: Collection<Todo> = get_collection(db)?;
    collection.delete_one(doc! {
        "id": todo.get_id().to_string()
    }).with_context(|| "Error updating the entry!")?;
    Ok(())
}