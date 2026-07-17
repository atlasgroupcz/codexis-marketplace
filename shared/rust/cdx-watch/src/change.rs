//! Change-log confirm mechanics shared by the watcher plugins. Each app keeps a
//! list of detected changes, every entry carrying a `confirmed_on` timestamp
//! that stays `None` until the user acknowledges it in the UI. The confirm and
//! unconfirmed-count logic is identical across apps; only the rest of each
//! change struct differs, so it is expressed over the [`ChangeRecord`] trait.

use crate::error::WatchError;

/// A change-log entry that can be marked confirmed. Unconfirmed until it carries
/// a timestamp.
pub trait ChangeRecord {
    /// Whether this entry has been acknowledged.
    fn is_confirmed(&self) -> bool;
    /// Mark this entry acknowledged at `timestamp`.
    fn set_confirmed(&mut self, timestamp: String);
}

/// Number of unconfirmed (unread) changes.
pub fn unconfirmed_count<C: ChangeRecord>(changes: &[C]) -> usize {
    changes.iter().filter(|c| !c.is_confirmed()).count()
}

/// Mark changes confirmed at `now`. `Some(i)` marks only index `i`; `None` marks
/// every unconfirmed entry. Returns how many were newly marked (0 when the
/// target was already confirmed). Errors when `index` is out of range.
pub fn confirm_changes<C: ChangeRecord>(
    changes: &mut [C],
    index: Option<usize>,
    now: &str,
) -> Result<usize, WatchError> {
    match index {
        Some(i) => {
            let len = changes.len();
            if i >= len {
                return Err(WatchError::IndexOutOfRange { index: i, len });
            }
            if changes[i].is_confirmed() {
                Ok(0)
            } else {
                changes[i].set_confirmed(now.to_string());
                Ok(1)
            }
        }
        None => {
            let mut marked = 0;
            for c in changes.iter_mut() {
                if !c.is_confirmed() {
                    c.set_confirmed(now.to_string());
                    marked += 1;
                }
            }
            Ok(marked)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Chg {
        confirmed: Option<String>,
    }
    impl ChangeRecord for Chg {
        fn is_confirmed(&self) -> bool {
            self.confirmed.is_some()
        }
        fn set_confirmed(&mut self, ts: String) {
            self.confirmed = Some(ts);
        }
    }
    fn changes(n: usize) -> Vec<Chg> {
        (0..n).map(|_| Chg { confirmed: None }).collect()
    }

    #[test]
    fn unconfirmed_count_counts_unmarked() {
        let mut c = changes(3);
        c[1].set_confirmed("t".into());
        assert_eq!(unconfirmed_count(&c), 2);
    }

    #[test]
    fn confirm_all_marks_every_unconfirmed_and_keeps_existing() {
        let mut c = changes(3);
        c[0].set_confirmed("old".into());
        let n = confirm_changes(&mut c, None, "now").unwrap();
        assert_eq!(n, 2);
        assert!(c.iter().all(|x| x.is_confirmed()));
        assert_eq!(c[0].confirmed.as_deref(), Some("old"));
        assert_eq!(c[2].confirmed.as_deref(), Some("now"));
    }

    #[test]
    fn confirm_index_marks_one() {
        let mut c = changes(3);
        let n = confirm_changes(&mut c, Some(1), "now").unwrap();
        assert_eq!(n, 1);
        assert!(!c[0].is_confirmed());
        assert_eq!(c[1].confirmed.as_deref(), Some("now"));
        assert!(!c[2].is_confirmed());
    }

    #[test]
    fn confirm_index_already_confirmed_marks_zero() {
        let mut c = changes(2);
        c[1].set_confirmed("old".into());
        let n = confirm_changes(&mut c, Some(1), "now").unwrap();
        assert_eq!(n, 0);
        assert_eq!(c[1].confirmed.as_deref(), Some("old"));
    }

    #[test]
    fn confirm_index_out_of_range_errors() {
        let mut c = changes(2);
        assert!(matches!(
            confirm_changes(&mut c, Some(5), "now"),
            Err(WatchError::IndexOutOfRange { index: 5, len: 2 })
        ));
    }
}
