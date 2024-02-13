use dotenv::dotenv;
// use dialoguer::Select;
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
            WriteLogger::new(LevelFilter::Debug, Config::default(), File::create(log_file).expect("Error opening log file!")),
        ]
    ).unwrap();

    // Initiating database
    let mut db = connect_db().unwrap();

    // Loading screen
    clear_term().unwrap_or_else(|e| warn!("{}", e));
    navigate_todos(&mut db, 0, SortingMethod::Due).unwrap_or_else(|e| warn!("{}", e));
    clear_term().unwrap_or_else(|e| warn!("{}", e));
}
