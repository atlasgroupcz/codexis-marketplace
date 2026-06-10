mod cli;
mod http;

fn main() {
    std::process::exit(match cli::run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("ERROR: {e:#}");
            2
        }
    });
}
