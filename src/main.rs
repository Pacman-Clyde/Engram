use clap::Parser;

fn main() {
    let cli = engram::cli::Cli::parse();
    if let Err(e) = engram::cli::dispatch(cli) {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
