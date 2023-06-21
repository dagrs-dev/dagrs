use dagrs::{FileContentError, Parser, ParserError, YamlParser, YamlTaskError};

#[test]
fn file_not_found_test() {
    let no_such_file = YamlParser.parse_tasks("./no_such_file.yaml");
    assert!(matches!(no_such_file, Err(ParserError::FileNotFound(_))));
}

#[test]
fn illegal_yaml_content() {
    let illegal_content = YamlParser.parse_tasks("tests/config/illegal_content.yaml");
    assert!(matches!(
        illegal_content,
        Err(ParserError::FileContentError(
            FileContentError::IllegalYamlContent(_)
        ))
    ));
}

#[test]
fn empty_content() {
    let empty_content = YamlParser.parse_tasks("tests/config/empty_file.yaml");
    assert!(matches!(empty_content,Err(ParserError::FileContentError(FileContentError::Empty(_)))))
}

#[test]
fn yaml_no_start_with_dagrs() {
    let forget_dagrs = YamlParser.parse_tasks("tests/config/no_start_with_dagrs.yaml");
    assert!(matches!(forget_dagrs,Err(ParserError::YamlTaskError(YamlTaskError::StartWordError))));
}

#[test]
fn yaml_task_no_name() {
    let no_task_name = YamlParser.parse_tasks("tests/config/no_task_name.yaml");
    assert!(matches!(no_task_name,Err(ParserError::YamlTaskError(YamlTaskError::NoNameAttr(_)))));
}

#[test]
fn yaml_task_not_found_precursor() {
    let not_found_pre = YamlParser.parse_tasks("tests/config/precursor_not_found.yaml");
    assert!(matches!(not_found_pre,Err(ParserError::YamlTaskError(YamlTaskError::NotFoundPrecursor(_)))));
}

#[test]
fn yaml_task_no_run_config() {
    let no_run = YamlParser.parse_tasks("tests/config/no_run.yaml");
    assert!(matches!(no_run,Err(ParserError::YamlTaskError(YamlTaskError::NoRunAttr(_)))));
}

#[test]
fn yaml_task_no_run_type_config() {
    let no_run_type = YamlParser.parse_tasks("tests/config/no_type.yaml");
    assert!(matches!(no_run_type,Err(ParserError::YamlTaskError(YamlTaskError::NoTypeAttr(_)))));
}

#[test]
fn yaml_task_unsupported_type_config() {
    let unsupported_type = YamlParser.parse_tasks("tests/config/unsupported_type.yaml");
    assert!(matches!(unsupported_type,Err(ParserError::YamlTaskError(YamlTaskError::UnsupportedType(_)))));
}

#[test]
fn yaml_task_no_script_config() {
    let script = YamlParser.parse_tasks("tests/config/no_script.yaml");
    assert!(matches!(script,Err(ParserError::YamlTaskError(YamlTaskError::NoScriptAttr(_)))));
}

#[test]
fn correct_parse() {
    let tasks = YamlParser.parse_tasks("tests/config/correct.yaml");
    assert!(tasks.is_ok());
}
