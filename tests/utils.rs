#[macro_export]
macro_rules! test_challenge {
    ($path:literal, $($parts:literal),+) => {
        run_challenge($path, vec![$($parts.to_owned()),+]).unwrap();
    };
}

#[macro_export]
macro_rules! test_challenges {
    ($path:literal, $($parts:literal),+) => {
        run_challenges($path, vec![$($parts.to_owned()),+]).unwrap();
    };
}
#[macro_export]
macro_rules! test_all_challenges {
    ($path:literal) => {
        run_challenges($path, vec![]).unwrap();
    };
}
