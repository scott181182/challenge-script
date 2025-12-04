use challenge_script::run_challenge;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to the challenge folder or challenge file
    challenge: String,
    /// Challenge case (or nested parts and case) to run
    cases: Vec<String>,
}

fn main() {
    let args = Args::parse();

    if let Err(err) = run_challenge(args.challenge, args.cases) {
        eprintln!("{}", err);
    }
}
