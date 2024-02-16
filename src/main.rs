use rustdo::*;
use std::env;

fn main() {
    // Loading environment variables
    load_env().unwrap_or(eprintln!("Error loading the configuration file!"));

    // Initiating database
    let mut db = connect_db().unwrap();

    let default_sort = match env::var("DEFAULT_SORT")
        .unwrap_or("due".to_string())
        .as_str()
    {
        "priority" => SortingMethod::Priority,
        "created" => SortingMethod::Created,
        _ => SortingMethod::Due,
    };

    // Loading screen
    clear_term().unwrap_or_else(|e| eprintln!("{}", e));
    navigate_todos(&mut db, 0, default_sort).unwrap_or_else(|e| eprintln!("{}", e));
    clear_term().unwrap_or_else(|e| eprintln!("{}", e));
}
