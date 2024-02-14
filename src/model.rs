use crate::date_utils::*;
use chrono::prelude::*;
use chrono::NaiveDate;
use humphrey_json::{error::ParseError, prelude::*, Value};
use uuid::Uuid;

pub type TodoCollection = Vec<Todo>;
pub type TodoCollectionRef = [Todo];

// Maximum priority level (inclusive)
pub const MAXPRIORITY: u32 = 3;

// The progress enum is used to track the progession of a given TODO
#[derive(FromJson, IntoJson, PartialEq, Debug)]
pub enum Progress {
    Zero = 0,
    Quarter = 25,
    Half = 50,
    ThreeQuarter = 75,
    Full = 100,
}
// Methods to simply edit the progress status of the enum
impl Progress {
    pub fn up(&self) -> Self {
        use Progress::*;
        match *self {
            Zero => Quarter,
            Quarter => Half,
            Half => ThreeQuarter,
            ThreeQuarter => Full,
            Full => Full,
        }
    }
    pub fn down(&self) -> Self {
        use Progress::*;
        match *self {
            Zero => Zero,
            Quarter => Zero,
            Half => Quarter,
            ThreeQuarter => Half,
            Full => ThreeQuarter,
        }
    }
}

// Some wrappers for chrono's NaiveDate, NaiveDateTime. Required to implement easy JSON serialization
pub struct MyDateTime(pub NaiveDateTime);
impl MyDateTime {
    pub fn get_0(&self) -> NaiveDateTime {
        self.0
    }
}
impl IntoJson for MyDateTime {
    fn to_json(&self) -> Value {
        Value::String(self.0.format(FORMAT_DATETIME).to_string())
    }
}
impl FromJson for MyDateTime {
    fn from_json(value: &Value) -> Result<Self, ParseError> {
        match value {
            Value::String(s) => {
                let convert_result = NaiveDateTime::parse_from_str(s, FORMAT_DATETIME);
                match convert_result {
                    Ok(datetime) => Ok(MyDateTime(datetime)),
                    Err(_) => Err(ParseError::TypeError),
                }
            }
            _ => Err(ParseError::TypeError),
        }
    }
}
impl std::fmt::Debug for MyDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MyDateTime")
            .field("DateTime", &self.0)
            .finish()
    }
}

pub struct MyDate(pub NaiveDate);
impl MyDate {
    pub fn get_0(&self) -> NaiveDate {
        self.0
    }
}
impl IntoJson for MyDate {
    fn to_json(&self) -> Value {
        Value::String(self.0.format(FORMAT_DATE).to_string())
    }
}
impl FromJson for MyDate {
    fn from_json(value: &Value) -> Result<Self, ParseError> {
        match value {
            Value::String(s) => {
                let convert_result = NaiveDate::parse_from_str(s, FORMAT_DATE);
                match convert_result {
                    Ok(date) => Ok(MyDate(date)),
                    Err(_) => Err(ParseError::TypeError),
                }
            }
            _ => Err(ParseError::TypeError),
        }
    }
}
impl std::fmt::Debug for MyDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MyDate").field("Date", &self.0).finish()
    }
}

// Main object: TODO
#[derive(Debug, FromJson, IntoJson)]
pub struct Todo {
    id: String,
    title: String,
    priority: u32,
    created: MyDateTime,
    due: Option<MyDate>,
    completed: bool,
    progress: Progress,
}

impl Todo {
    // Initialization of Todo struct
    pub fn new(title: &str, priority: u32, due_date_opt: Option<&str>) -> Todo {
        Todo {
            id: Uuid::now_v7().to_string(),
            title: title.to_owned(),
            priority,
            created: MyDateTime(Local::now().naive_local()),
            due: {
                match due_date_opt {
                    None => None,
                    Some(due_date) => {
                        let convert_date = convert_str_valid_date(due_date);
                        match convert_date {
                            Ok(date) => Some(MyDate(date)),
                            Err(_) => None,
                        }
                    }
                }
            },
            completed: false,
            progress: Progress::Zero,
        }
    }
    // methods to access/set private properties
    pub fn is_complete(&self) -> bool {
        self.completed
    }
    pub fn get_id(&self) -> &str {
        &self.id
    }
    pub fn get_title(&self) -> &str {
        &self.title
    }
    pub fn get_due_date(&self) -> &Option<MyDate> {
        &self.due
    }
    pub fn get_created_date(&self) -> &MyDateTime {
        &self.created
    }
    pub fn get_priority(&self) -> u32 {
        self.priority
    }
    pub fn get_progress(&self) -> &Progress {
        &self.progress
    }
    pub fn set_id(&mut self, id: &str) {
        self.id = id.to_string()
    }
    pub fn toggle_read(&mut self) {
        self.completed = !self.completed
    }
    pub fn increase_priority(&mut self) {
        self.priority = std::cmp::min(self.priority + 1, MAXPRIORITY)
    }
    pub fn decrease_priority(&mut self) {
        self.priority = std::cmp::max(self.priority as i32 - 1, 0) as u32
    }
    pub fn increase_progress(&mut self) {
        self.progress = self.progress.up()
    }
    pub fn decrease_progress(&mut self) {
        self.progress = self.progress.down()
    }
}
// Useful enums to keep track of actions/results/events of our functions
pub enum SortingMethod {
    Priority,
    Due,
    Created,
}

pub enum Action {
    ToggleRead,
    Delete,
    IncreasePriority,
    DecreasePriority,
    Reload,
    Sort(SortingMethod),
    IncreaseProgress,
    DecreaseProgress,
    Edit,
    Add,
    Export,
    DeleteCompleted,
}

pub enum KeyEvent {
    Back,
    Sort,
    NavigateUp,
    NavigateDown,
    ToggleRead,
    Delete,
    IncreasePriority,
    DecreasePriority,
    IncreaseProgress,
    DecreaseProgress,
    Edit,
    Add,
    Export,
    DeleteCompleted,
}
