use chrono::prelude::*;
use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::date_utils::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    id: Uuid,
    title: String,
    priority: u32,
    created: NaiveDateTime,
    due: Option<NaiveDate>,
    completed: bool
}

pub const MAXPRIORITY: u32 = 3;

impl Todo {
    // Initialization of Todo struct
    pub fn new(title: &str, priority: u32, due_date_opt: Option<&str>) -> Todo {
        Todo {
            id: Uuid::now_v7(),
            title: title.to_owned(),
            priority,
            created: Local::now().naive_local(),
            due: {
                match due_date_opt {
                    None => None,
                    Some(due_date) => convert_str_valid_date(due_date).ok()
                }
            },
            completed: false
        }
    }
    // methods to access private properties
    pub fn is_complete(&self) -> bool { self.completed }
    pub fn get_id(&self) -> Uuid { self.id }
    pub fn get_title(&self) -> &str { &self.title }
    pub fn get_due_date(&self) -> Option<NaiveDate> { self.due }
    pub fn get_created_date(&self) -> NaiveDateTime { self.created }
    pub fn get_priority(&self) -> u32 { self.priority }
}

pub enum SortingMethod {
    Priority,
    Due,
    Created
}

pub enum Action {
    ToggleRead,
    Delete,
    IncreasePriority,
    DecreasePriority,
    Reload
}

pub enum KeyEvent {
    Back,
    Sort,
    NavigateUp,
    NavigateDown,
    ToggleRead,
    Delete,
    IncreasePriority,
    DecreasePriority
}