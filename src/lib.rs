extern crate anymap2;
extern crate bimap;
extern crate clap;
extern crate deno_core;
extern crate log;
extern crate simplelog;
extern crate yaml_rust;
extern crate serde_json;
extern crate once_cell;

pub mod engine;
pub mod utils;
pub mod parser;
pub mod task;

// pub use engine::{DagEngine, DagError, Graph, RunningError, YamlError, YamlFormatError};
// pub use task::{
//     ComplexAction, Content, DefaultTask, Input, Output, SimpleAction,
//     YamlTask,
// };

use simplelog::*;
use std::{
    env,
    fs::{create_dir, File},
};

/// Init a logger.
///
/// # Example
/// ```rust
/// // Default path (HOME/.dagrs/dagrs.log)
/// dagrs::init_logger(None);
/// ```
/// ```rust
/// dagrs::init_logger(Some("./dagrs.log"));
/// ```
///
/// **Note**, this function shall only be called once.
///
/// Default logger is [Simplelog](https://crates.io/crates/simplelog), you can
/// also use other log implementations. Just remember to initialize them before
/// running dagrs.
pub fn init_logger(path: Option<&str>) {
    let log_path = path.map_or(
        env::var("HOME").map_or("./dagrs.log".to_owned(), |home| {
            create_dir(format!("{}/.dagrs", home)).unwrap_or(());
            format!("{}/.dagrs/dagrs.log", home)
        }),
        |s| s.to_owned(),
    );

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create(log_path).unwrap(),
        ),
    ])
    .unwrap();
}
