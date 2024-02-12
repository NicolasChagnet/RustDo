use dotenv::dotenv;
use dialoguer::Select;
use todo::*;

use simplelog::*;
use log::*;
use std::{env, fs::File};

const DEFAULT_LOG: &str = "todo.log";

fn main() {
    // Loading environment variables
    dotenv().ok();
    // Logging system
    let log_file = match env::var("LOG_FILE") {
        Ok(v) => v.clone(),
        Err(_) => {
            error!("Error loading log filename from environment variables, switching to default...");
            DEFAULT_LOG.to_string()
        }
    };
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create(log_file).expect("Error opening log file!")),
        ]
    ).unwrap();

    // Initiating database
    let db = connect_db().unwrap();

    // Generating menu
    let menu_items = vec![
        "List TODOs", 
        "Add TODO", 
        "Delete completed", 
        "Exit"
    ];
    let n = menu_items.len();
    loop {
        clear_term().unwrap_or_else(|e| warn!("{}", e));
        // Present selection of menu choices
        let selection = Select::new()
            .with_prompt("What do you want to do?")
            .default(0)
            .items(&menu_items)
            .interact()
            .unwrap_or( n);
        // Handles the selction and forwards to appropriate branch
        match selection {
            0 => {
                navigate_todos(&db, 0).unwrap_or_else(|e| warn!("{}", e))
            },
            1 => {
                add_todo(&db).unwrap_or_else(|e| warn!("{}", e))
            },
            2 => {
                delete_completed(&db).unwrap_or_else(|e| warn!("{}", e))
            },
            3 => std::process::exit(0),
            _ => unreachable!()
        };
    }
}
