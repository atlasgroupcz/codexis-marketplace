//! The system-wide `sbirka-listin-cli` binary — a thin shell over the crate's
//! `cli` module.

fn main() {
    std::process::exit(match sbirka_listin_cli::cli::run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("ERROR: {e:#}");
            2
        }
    });
}
