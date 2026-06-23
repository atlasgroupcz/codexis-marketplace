mod cli;
mod codelist;
mod dataset;
mod decode;
mod http;
mod index;
mod manifest;
mod paths;

fn main() {
    std::process::exit(match cli::run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("ERROR: {e:#}");
            2
        }
    });
}
