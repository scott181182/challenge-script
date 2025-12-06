use challenge_script::run_challenges;

mod utils;

#[test]
fn test_recursive_all() {
    test_all_challenges!("./tests/data/test2.yml");
}

#[test]
fn test_recursive() {
    test_challenges!("./tests/data/test2.yml", "args");
}
