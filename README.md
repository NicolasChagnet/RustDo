# Rust todo app
Minimal CLI todo application written in Rust. The following features are currently supported:

- CLI interaction using [console](https://crates.io/crates/console) and [dialoguer](https://crates.io/crates/dialoguer).
- Persistent storage in lightweight local JSON database using [JasonDB](https://crates.io/crates/jasondb).
- Time and date handling using [chrono](https://crates.io/crates/chrono).
- Error handling using [anyhow](https://crates.io/crates/anyhow).
- Configuration using environment files through [dotenv](https://crates.io/crates/dotenv).

# Features

- Add, edit, delete TODOs with title, due date, progress status, priority
- Interactively increase/decrease priority and progress levels
- Toggle complete TODOs, delete all completed TODOs in one command
- Sort by due date, priority, created date