# Rust todo app
Minimal CLI todo application written in Rust.

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

# Installation

## Cargo install

You can install this crate with Cargo by doing

```
cargo install rustdo
```

## From source

You can build this crate from source using the following commands:
```
> git clone https://github.com/NicolasChagnet/RustDo.git
> cd rustdo
> cargo build
> cp target/release/rustdo /dest/path
```
In the last command, you can copy the binary to any folder contained in your `$PATH`.

# Usage

To launch the application, simply run the following command
```
rustdo
```
You can then navigate between todos using up/down arrows, add a new todo with `a`, edit existing todos with `e` and sort the list of todos using `s`. When hovering over a todo, you can also mark it as read using `x`, delete it with `z`, change its progress status using the left/right arrows and change its priority status using `+`/`-`. The key `m` exports the todos to markdown file, `Z` deletes all completed todos and `enter` leaves the application.

The database is located in the OS specific data folder given by the method `config_dir()` from `directories::ProjectDirs` in the [directories](https://crates.io/crates/directories) crate. A config file `rustdo_config` can be modified in the folder defined by `config_dir()` with the following parameters

```
MD_FILE="$HOME/rustdo.md"
EXPORT_ON_EXIT=false
DEFAULT_SORT="due"|"priority"|"created"
```