mod cli;
mod http;
mod ico;
mod orsr;

fn main() {
    std::process::exit(match cli::run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("ERROR: {e:#}");
            2
        }
    });
}
