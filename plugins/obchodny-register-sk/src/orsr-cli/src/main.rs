mod cli;
mod config;
mod http;
mod ico;
mod orsr;
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
