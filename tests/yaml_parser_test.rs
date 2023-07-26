use std::collections::HashMap;

use dagrs::{FileContentError, Parser, ParserError, YamlParser, YamlTaskError};

#[test]
fn file_not_found_test() {
    let no_such_file = YamlParser.parse_tasks("./no_such_file.yaml",HashMap::new());
    assert!(matches!(no_such_file, Err(ParserError::FileNotFound(_))));
}

#[test]
fn illegal_yaml_content() {
    let illegal_content = YamlParser.parse_tasks("tests/config/illegal_content.yaml",HashMap::new());
    assert!(matches!(
        illegal_content,
        Err(ParserError::FileContentError(
            FileContentError::IllegalYamlContent(_)
        ))
    ));
}

#[test]
fn empty_content() {
    let empty_content = YamlParser.parse_tasks("tests/config/empty_file.yaml",HashMap::new());
    assert!(matches!(empty_content,Err(ParserError::FileContentError(FileContentError::Empty(_)))))
}

#[test]
fn yaml_no_start_with_dagrs() {
    let forget_dagrs = YamlParser.parse_tasks("tests/config/no_start_with_dagrs.yaml",HashMap::new());
    assert!(matches!(forget_dagrs,Err(ParserError::YamlTaskError(YamlTaskError::StartWordError))));
}

#[test]
fn yaml_task_no_name() {
    let no_task_name = YamlParser.parse_tasks("tests/config/no_task_name.yaml",HashMap::new());
    assert!(matches!(no_task_name,Err(ParserError::YamlTaskError(YamlTaskError::NoNameAttr(_)))));
}

#[test]
fn yaml_task_not_found_precursor() {
    let not_found_pre = YamlParser.parse_tasks("tests/config/precursor_not_found.yaml",HashMap::new());
    assert!(matches!(not_found_pre,Err(ParserError::YamlTaskError(YamlTaskError::NotFoundPrecursor(_)))));
}

#[test]
fn yaml_task_no_script_config() {
    let script = YamlParser.parse_tasks("tests/config/no_script.yaml",HashMap::new());
    assert!(matches!(script,Err(ParserError::YamlTaskError(YamlTaskError::NoScriptAttr(_)))));
}

#[test]
fn correct_parse() {
    let tasks = YamlParser.parse_tasks("tests/config/correct.yaml",HashMap::new());
    assert!(tasks.is_ok());
}
