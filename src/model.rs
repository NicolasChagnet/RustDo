use chrono::prelude::*;
use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use serde_repr::*;
use uuid::Uuid;
use crate::date_utils::*;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u32)]
pub enum Progress {
    Zero = 0,
    Quarter = 25,
    Half = 50,
    ThreeQuarter = 75,
    Full = 100
}

impl Progress {
    pub fn up(&self) -> Self {
        use Progress::*;
        match *self {
            Zero => Quarter,
            Quarter => Half,
            Half => ThreeQuarter,
            ThreeQuarter => Full,
            Full => Full
        }
    }
    pub fn down(&self) -> Self {
        use Progress::*;
        match *self {
            Zero => Zero,
            Quarter => Zero,
            Half => Quarter,
            ThreeQuarter => Half,
            Full => ThreeQuarter
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    id: Uuid,
    title: String,
    priority: u32,
    created: NaiveDateTime,
    due: Option<NaiveDate>,
    completed: bool,
    progress: Progress
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
            completed: false,
            progress: Progress::Zero
        }
    }
    // methods to access private properties
    pub fn is_complete(&self) -> bool { self.completed }
    pub fn get_id(&self) -> Uuid { self.id }
    pub fn get_title(&self) -> &str { &self.title }
    pub fn get_due_date(&self) -> Option<NaiveDate> { self.due }
    pub fn get_created_date(&self) -> NaiveDateTime { self.created }
    pub fn get_priority(&self) -> u32 { self.priority }
    pub fn get_progress(&self) -> &Progress { &self.progress }
}

// impl std::convert::TryFrom<u32> for Progress {
//     type Error = ();
//     fn try_from(value: u32) -> Result<Self, Self::Error> {
//         match value {
//             0 => Ok(Progress::Zero),
//             25 => Ok(Progress::Quarter),
//             50 => Ok(Progress::Half),
//             75 => Ok(Progress::ThreeQuarter),
//             100 => Ok(Progress::Full),
//             _ => Err(())
//         }
//     }
// }

// impl std::convert::TryInto<u32> for Progress {
//     type Error = ();
//     fn try_into(self) -> Result<u32, Self::Error> {
//         match self {
//             Zero => Ok(0),
//             Quarter => Ok(25),
//             Half => Ok(50),
//             ThreeQuarter => Ok(75),
//             Full => Ok(100)
//         }
//     }
// }

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
    Reload,
    IncreaseProgress,
    DecreaseProgress
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
    DecreaseProgress
}