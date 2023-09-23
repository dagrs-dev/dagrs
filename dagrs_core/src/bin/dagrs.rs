use std::collections::HashMap;

use clap::Parser;
use dagrs_core::{Dag, log, LogLevel};

#[derive(Parser, Debug)]
#[command(name= "dagrs",version= "0.2.0")]
struct Args{
    /// Log output file, the default is to print to the terminal.
    #[arg(long)]
    log_path: Option<String>,
    /// yaml configuration file path.
    #[arg(long)]
    yaml: String,
    /// Log level, the default is Info.
    #[arg(long)]
    log_level:Option<String>
}

fn main() {
    let args = Args::parse();
    let log_level=args.log_level.map_or(LogLevel::Info,|level|{
        match level.as_str() {
            "debug" => {LogLevel::Debug}
            "info" => {LogLevel::Info}
            "warn" => {LogLevel::Warn}
            "error" => {LogLevel::Error}
            "off" => {LogLevel::Off}
            _ => {
                println!("The logging level can only be [debug,info,warn,error,off]");
                std::process::abort();
            }
        }
    });
    let _initialized = match args.log_path {
        None => {log::init_logger(log_level,None)}
        Some(path) => {log::init_logger(log_level,Some(std::fs::File::create(path).unwrap()))}
    };
    let yaml_path=args.yaml;
    let mut dag=Dag::with_yaml(yaml_path.as_str(),HashMap::new()).unwrap();
    assert!(dag.start().unwrap());
}
