//! Shared library for the sbirka-listin plugin: the register scraping, document
//! fetch, watcher, and notification logic used by both binaries - the
//! system-wide `sbirka-listin-cli` and the plugin-component `sbirka-listin-cgi`
//! handler run in place by the daemon.

pub mod api;
pub mod cli;
pub mod dokument;
pub mod http;
pub mod notify;
pub mod parse;
pub mod sources;
pub mod watch;
