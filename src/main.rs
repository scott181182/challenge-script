use challenge_script::{run_challenge, run_challenges};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to the challenge folder or challenge file
    challenge: String,
    /// Challenge case (or nested parts and case) to run
    cases: Vec<String>,

    /// Run all nested parts and cases under the specified file and/or group.
    #[arg(short, long)]
    recursive: bool,
}

fn main() {
    let args = Args::parse();

    let res = if args.recursive {
        run_challenges(args.challenge, args.cases)
    } else {
        run_challenge(args.challenge, args.cases)
    };

    if let Err(err) = res {
        eprintln!("{err}");
    }
}
