use clap::Parser;
use dagrs::{init_logger, DagEngine};
use log::*;

#[derive(Parser)]
#[clap(version)]
/// Command Line input
struct Args {
    /// YAML file path
    file: String,
    /// Log file path
    #[clap(short, long)]
    logpath: Option<String>,
}

fn main() {
    let args = Args::parse();
    let dagrs: DagEngine = DagEngine::new();

    init_logger(args.logpath.as_deref());

    if let Err(e) = dagrs.run_from_yaml(&args.file) {
        error!("[Error] {}", e);
    }
}

#[test]
fn test_dag() {
    let res = DagEngine::new()
        .run_from_yaml("test/test_dag2.yaml")
        .unwrap();
    assert_eq!(res, true)
}

#[test]
fn test_runscript() {
    let res = DagEngine::new()
        .run_from_yaml("test/test_dag1.yaml")
        .unwrap();
    assert_eq!(res, true)
}

#[test]
fn test_value_pass1() {
    use std::fs::File;
    use std::io::Read;

    let res = DagEngine::new()
        .run_from_yaml("test/test_value_pass1.yaml")
        .unwrap();
    assert_eq!(res, true);

    let mut buf = String::new();
    File::open("./test/test_value_pass1.txt")
        .expect("Test Fails, File not exist.")
        .read_to_string(&mut buf)
        .expect("Test Fails, Read file fails.");
    
    assert_eq!(buf, "10\n");
}

#[test]
fn test_value_pass2() {
    use std::fs::File;
    use std::io::Read;

    let res = DagEngine::new()
        .run_from_yaml("test/test_value_pass2.yaml")
        .unwrap();
    assert_eq!(res, true);

    let mut buf1 = String::new();
    File::open("./test/test_value_pass2.txt")
        .expect("Test Fails, File not exist.")
        .read_to_string(&mut buf1)
        .expect("Test Fails, Read file fails.");

    let mut buf2 = String::new();
        File::open("./README.md")
            .expect("Test Fails, File not exist.")
            .read_to_string(&mut buf2)
            .expect("Test Fails, Read file fails.");
    
    assert_eq!(buf1, buf2);
}


#[test]
fn test_loop() {
    let res = DagEngine::new()
        .run_from_yaml("test/test_loop1.yaml")
        .unwrap();
    assert_eq!(res, false)
}

#[test]
fn test_complex_loop() {
    let res = DagEngine::new()
        .run_from_yaml("test/test_loop2.yaml")
        .unwrap();
    assert_eq!(res, false)
}

#[test]
fn test_format_error1() {
    use dagrs::{DagError, YamlError, YamlFormatError};
    let res = DagEngine::new().run_from_yaml("test/test_error1.yaml");

    assert!(matches!(
        res,
        Err(DagError::YamlError(YamlError::YamlFormatError(
            YamlFormatError::NoName(_)
        )))
    ));
}

#[test]
fn test_format_error2() {
    use dagrs::{DagError, YamlError, YamlFormatError};
    let res = DagEngine::new().run_from_yaml("test/test_error2.yaml");

    assert!(matches!(
        res,
        Err(DagError::YamlError(YamlError::YamlFormatError(
            YamlFormatError::StartWordError
        )))
    ));
}

#[test]
fn test_rely_error() {
    use dagrs::{DagError, RunningError};
    let res = DagEngine::new().run_from_yaml("test/test_error3.yaml");

    assert!(matches!(
        res,
        Err(DagError::RunningError(RunningError::RelyTaskIllegal(_)))
    ));
}

#[test]
fn test_no_runscript() {
    use dagrs::{DagError, YamlError, YamlFormatError};
    let res = DagEngine::new().run_from_yaml("test/test_error4.yaml");

    assert!(matches!(
        res,
        Err(DagError::YamlError(YamlError::YamlFormatError(
            YamlFormatError::RunScriptError(_)
        )))
    ));
}
