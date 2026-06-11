mod cli;
mod config;
mod error;
mod http;
mod sources;

fn main() {
    std::process::exit(match cli::run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("ERROR: {e:#}");
            2
        }
    });
}
