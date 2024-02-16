use crate::model::Todo;
use anyhow::{bail, Context, Result};
use directories::ProjectDirs;
use dotenv;
use jasondb::*;
use std::path::PathBuf;

pub type DatabaseModel = Database<Todo>;
const APPNAME: &str = "rustdo";

const DEFAULT_CONFIG: &str = r#"MD_FILE="$HOME/rustdo.md"
EXPORT_ON_EXIT=false
DEFAULT_SORT="due"
"#;

// Loads environment variables from config file
pub fn load_env() -> Result<()> {
    // Locates the configuration folder
    let path_root = match ProjectDirs::from("", "", APPNAME) {
        Some(proj_dirs) => proj_dirs.config_dir().to_path_buf(),
        None => bail!("Cannot find configuration folder!"),
    };
    std::fs::create_dir_all(&path_root)?; // Creating the config directory if it does not exist!
    let config_filename = path_root.join("rustdo_config");
    if !config_filename.exists() {
        std::fs::write(&config_filename, DEFAULT_CONFIG)?
    }
    dotenv::from_filename(config_filename).ok(); // Loading environment file
    Ok(())
}

// Obtain location of database file
pub fn get_location_database() -> Result<PathBuf> {
    let path_root = match ProjectDirs::from("", "", APPNAME) {
        Some(proj_dirs) => proj_dirs.data_dir().to_path_buf(),
        None => bail!("Cannot find configuration folder!"),
    };
    std::fs::create_dir_all(&path_root)?; // Creating the config directory if it does not exist!
    Ok(path_root.join("rustdo_db.json"))
}

// DB connection function
pub fn connect_db() -> Result<DatabaseModel> {
    let filename = get_location_database()?;
    let db: DatabaseModel = Database::new(filename).with_context(|| "Error opening database!")?;
    Ok(db)
}

// Queries the DB for all TODOs
pub fn get_todos(db: &mut DatabaseModel) -> Result<Vec<(String, Todo)>> {
    let query = db
        .iter()
        .filter_map(|x| x.ok())
        .collect::<Vec<(String, Todo)>>();
    Ok(query)
}

// Queries the DB for all completed TODOs
pub fn get_completed_todos(db: &mut DatabaseModel) -> Result<Vec<(String, Todo)>> {
    let query: Vec<(String, Todo)> = db
        .query(query!(completed == true))?
        .filter_map(|x| x.ok())
        .collect::<Vec<(String, Todo)>>();
    Ok(query)
}

// Inserts a TODO object inside the DB
pub fn insert_todo(db: &mut DatabaseModel, todo: &Todo) -> Result<()> {
    db.set(todo.get_id(), todo)
        .with_context(|| "Error inserting TODO inside DB!")?;
    Ok(())
}

// Updates the DB element associated with a TODO object
pub fn update_todo(db: &mut DatabaseModel, todo_replace: &Todo) -> Result<()> {
    db.set(todo_replace.get_id(), todo_replace)
        .with_context(|| "Error updating the entry!")?;
    Ok(())
}

// Deletes a TODO object from DB
pub fn delete_todo(db: &mut DatabaseModel, todo: &Todo) -> Result<()> {
    db.delete(todo.get_id())
        .with_context(|| "Error updating the entry!")?;
    Ok(())
}
