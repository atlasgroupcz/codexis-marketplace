//! The watcher domain shared by the CODEXIS "sledování" plugins (katastr,
//! insolvence, znamky, …): the change-log confirm mechanics and the
//! poll → persist → collect-result check orchestration over a keyed item store.
//!
//! A *capability* crate: it owns the parts of a watcher that are identical
//! across apps (the change-log confirm/count and the check loop), and leaves the
//! app-specific parts - fetching, diffing, the snapshot and change shapes - to
//! the plugin via the [`Watcher`](track::Watcher) trait. Independent of the
//! notify (`cdx-notify`) and CGI (`cdx-cgi`) capabilities; a plugin wires them
//! together (e.g. "check → if change → notify").

pub mod change;
pub mod error;
pub mod track;

pub use error::WatchError;
