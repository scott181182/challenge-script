use challenge_script::{challenge::{ChallengeCaseError, ChallengeExecutionError, CommandParseError, StringReferenceError}, run_challenge, ChallengeFileError, ProgramError};



#[test]
fn test_success_1() {
    run_challenge("./tests/data/test1.yml", vec!["test".to_owned()]).unwrap();
}

#[test]
fn test_success_2() {
    run_challenge("./tests/data/test2.yml", vec!["group1".to_owned(), "test".to_owned()]).unwrap();
    run_challenge("./tests/data/test2.yml", vec!["group2".to_owned(), "test".to_owned()]).unwrap();
}

#[test]
fn test_success_3() {
    run_challenge("./tests/data/test3", vec!["group1".to_owned(), "test".to_owned()]).unwrap();
}

#[test]
fn test_error_expected() {
    let res = run_challenge("./tests/data/test3", vec!["group1".to_owned(), "error".to_owned()]);

    if let Err(ProgramError::ExecutionError(ChallengeExecutionError::UnexpectedOutput{ expected, actual})) = res {
        assert_eq!(expected, "test_1");
        assert_eq!(actual, "test_2");
    } else {
        panic!("Unexpected result: {:?}", res);
    }
}

#[test]
fn test_error_case_not_found_root() {
    let res = run_challenge("./tests/data/test3", vec!["group2".to_owned(), "error".to_owned()]);

    if let Err(ProgramError::InputCaseError(ChallengeCaseError::CaseNotFound { case, config_name })) = res {
        assert_eq!(config_name, "Test 3");
        assert_eq!(case, "group2");
    } else {
        panic!("Unexpected result: {:?}", res);
    }
}

#[test]
fn test_error_case_not_found_nested() {
    let res = run_challenge("./tests/data/test3", vec!["group1".to_owned(), "nonexistent".to_owned()]);

    if let Err(ProgramError::InputCaseError(ChallengeCaseError::CaseNotFound { case, config_name })) = res {
        assert_eq!(config_name, "group1");
        assert_eq!(case, "nonexistent");
    } else {
        panic!("Unexpected result: {:?}", res);
    }
}

#[test]
fn test_error_case_not_enough_cases() {
    let res = run_challenge("./tests/data/test3", vec!["group1".to_owned()]);

    assert!(matches!(res, Err(ProgramError::InputCaseError(ChallengeCaseError::NotEnoughCases))));
}

#[test]
fn test_error_empty_command() {
    let res = run_challenge("./tests/data/bad.yml", vec!["empty_command".to_owned(), "doesn't matter".to_owned()]);

    assert!(matches!(res, Err(ProgramError::ExecutionError(ChallengeExecutionError::BadCommand(CommandParseError::EmptyCommand)))));
}

#[test]
fn test_error_malformed_command() {
    let res = run_challenge("./tests/data/bad.yml", vec!["malformed_command".to_owned(), "whatever".to_owned()]);

    if let Err(ProgramError::ExecutionError(ChallengeExecutionError::BadCommand(CommandParseError::MalformedString(cmd)))) = res {
        assert_eq!(cmd, "echo \"not closed");
    } else {
        panic!("Unexpected result: {:?}", res);
    }
}

#[test]
fn test_error_input_file_not_found() {
    let res = run_challenge("./tests/data/bad.yml", vec!["bad_input".to_owned(), "nonexistent".to_owned()]);

    assert!(matches!(res, Err(ProgramError::ExecutionError(ChallengeExecutionError::BadStringReference(StringReferenceError::FileRead(_))))));
}

#[test]
fn test_error_challenge_file_not_found() {
    let res = run_challenge("./tests/data/empty/challenge.yml", vec![]);

    assert!(matches!(res, Err(ProgramError::InputFileError(ChallengeFileError::FileDoesNotExist(_)))));
}
#[test]
fn test_error_challenge_file_not_found_in_directory() {
    let res = run_challenge("./tests/data/empty", vec![]);

    assert!(matches!(res, Err(ProgramError::InputFileError(ChallengeFileError::FileNotFoundInDirectory(_)))));
}
