use std::collections::HashSet;
use std::sync::LazyLock;

use regex::Regex;
use serde::Serialize;

static BLOCK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n\s*\n").unwrap());
static INSERTED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)\{\+\+(.*?)\+\+\}").unwrap());
static DELETED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)\{--(.*?)--\}").unwrap());
static ENUMERATOR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\(\d+[a-z]*\)|[a-z]{1,2}\)|\d+[a-z]*\.)\s*").unwrap());
static FOOTNOTE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)\)\s*").unwrap());
static REFERENCE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(§|odst\.|odstavc[eiůích]*|odstavec|písm\.|písmen[eaou]?|bod[uůy]?|čl\.|článk[uůy])\s*(\d+[a-z]*|[a-z]{1,2}\))",
    )
    .unwrap()
});
static LETTER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?-u:\b)[a-z]{1,2}\)").unwrap());
static ESCAPE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\([\\`*_{}\[\]()#+\-.!])").unwrap());
static SPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

const SIMILAR: f64 = 0.6;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum DiffItem {
    Deleted {
        #[serde(rename = "where")]
        location: Option<String>,
        text: String,
    },
    Inserted {
        #[serde(rename = "where")]
        location: Option<String>,
        text: String,
    },
    Changed {
        #[serde(rename = "where")]
        location: Option<String>,
        before: String,
        after: String,
        references_only: bool,
    },
    Renumbered {
        before: Option<String>,
        after: Option<String>,
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffOverview {
    pub items: Vec<DiffItem>,
    pub renumbering_only: bool,
}

struct Entry {
    block: usize,
    text: String,
    enumerator: Option<String>,
    body: String,
    location: Option<String>,
}

/// Reads a CriticMarkup diff paragraph by paragraph: realigns paragraphs that only
/// moved, so a deleted paragraph, an inserted one, a rewritten one and a mere
/// renumbering each come out as their own item.
pub fn diff_overview(diff_markdown: Option<&str>) -> Option<DiffOverview> {
    let diff_markdown = diff_markdown.filter(|diff| !diff.is_empty())?;
    let blocks: Vec<&str> = BLOCK
        .split(diff_markdown)
        .filter(|block| !block.trim_start().starts_with('#'))
        .collect();
    let marked: Vec<bool> = blocks
        .iter()
        .map(|block| block.contains("{--") || block.contains("{++"))
        .collect();
    let before = entries(
        blocks
            .iter()
            .map(|block| clean(&DELETED.replace_all(&INSERTED.replace_all(block, ""), "$1"))),
    );
    let after = entries(
        blocks
            .iter()
            .map(|block| clean(&INSERTED.replace_all(&DELETED.replace_all(block, ""), "$1"))),
    );
    let mut items = Vec::new();
    let mut used_before = HashSet::new();
    let mut used_after = HashSet::new();
    for same_block in [true, false] {
        for (index, old) in before.iter().enumerate() {
            if used_before.contains(&index) {
                continue;
            }
            let fresh = after.iter().enumerate().position(|(candidate, entry)| {
                !used_after.contains(&candidate)
                    && entry.body == old.body
                    && (!same_block || entry.block == old.block)
            });
            if let Some(candidate) = fresh {
                let fresh = &after[candidate];
                used_before.insert(index);
                used_after.insert(candidate);
                if old.enumerator != fresh.enumerator && (marked[old.block] || marked[fresh.block])
                {
                    items.push(DiffItem::Renumbered {
                        before: old.location.clone(),
                        after: fresh.location.clone(),
                        text: fresh.text.clone(),
                    });
                }
            }
        }
    }
    for (index, old) in before.iter().enumerate() {
        if used_before.contains(&index) {
            continue;
        }
        let fresh = after.iter().enumerate().position(|(candidate, entry)| {
            !used_after.contains(&candidate) && entry.block == old.block
        });
        if let Some(candidate) = fresh {
            let fresh = &after[candidate];
            if similarity(&old.body, &fresh.body) >= SIMILAR {
                used_before.insert(index);
                used_after.insert(candidate);
                items.push(DiffItem::Changed {
                    location: fresh.location.clone(),
                    before: old.text.clone(),
                    after: fresh.text.clone(),
                    references_only: neutral(&old.text) == neutral(&fresh.text),
                });
            }
        }
    }
    for (index, old) in before.iter().enumerate() {
        if !used_before.contains(&index) {
            items.push(DiffItem::Deleted {
                location: old.location.clone(),
                text: old.text.clone(),
            });
        }
    }
    for (index, fresh) in after.iter().enumerate() {
        if !used_after.contains(&index) {
            items.push(DiffItem::Inserted {
                location: fresh.location.clone(),
                text: fresh.text.clone(),
            });
        }
    }
    items.sort_by_key(|item| block_of(item, &before, &after));
    let renumbering_only = !items.is_empty()
        && items.iter().all(|item| {
            matches!(
                item,
                DiffItem::Renumbered { .. }
                    | DiffItem::Changed {
                        references_only: true,
                        ..
                    }
            )
        });
    Some(DiffOverview {
        items,
        renumbering_only,
    })
}

fn block_of(item: &DiffItem, before: &[Entry], after: &[Entry]) -> usize {
    let text = match item {
        DiffItem::Changed { after, .. } => after,
        DiffItem::Deleted { text, .. }
        | DiffItem::Inserted { text, .. }
        | DiffItem::Renumbered { text, .. } => text,
    };
    after
        .iter()
        .chain(before)
        .find(|entry| &entry.text == text)
        .map_or(0, |entry| entry.block)
}

/// One entry per non-empty paragraph, labelled with the paragraph, letter and point it sits in.
fn entries(texts: impl Iterator<Item = String>) -> Vec<Entry> {
    let mut paragraph: Option<String> = None;
    let mut letter: Option<String> = None;
    let mut point: Option<String> = None;
    let mut result = Vec::new();
    for (block, text) in texts.enumerate() {
        if text.is_empty() {
            continue;
        }
        if let Some(footnote) = FOOTNOTE.captures(&text) {
            let body = text[footnote.get(0).map_or(0, |whole| whole.end())..].to_owned();
            let location = Some(format!("poznámka {})", &footnote[1]));
            result.push(Entry {
                block,
                text,
                enumerator: None,
                body,
                location,
            });
            continue;
        }
        let enumerator = ENUMERATOR
            .captures(&text)
            .map(|captures| captures[1].to_owned());
        match enumerator.as_deref() {
            Some(value) if value.starts_with('(') => {
                paragraph = Some(value[1..value.len() - 1].to_owned());
                letter = None;
                point = None;
            }
            Some(value) if value.ends_with(')') => {
                letter = Some(value.to_owned());
                point = None;
            }
            Some(value) => point = Some(value[..value.len() - 1].to_owned()),
            None => {}
        }
        let location = [
            paragraph.as_ref().map(|value| format!("odst. {value}")),
            letter.as_ref().map(|value| format!("písm. {value}")),
            point.as_ref().map(|value| format!("bod {value}")),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
        let body = ENUMERATOR.replace(&text, "").into_owned();
        result.push(Entry {
            block,
            text,
            enumerator,
            body,
            location: Some(location).filter(|location| !location.is_empty()),
        });
    }
    result
}

fn clean(text: &str) -> String {
    SPACE
        .replace_all(&ESCAPE.replace_all(text, "$1"), " ")
        .trim()
        .to_owned()
}

/// The text with paragraph numbers and cross-references blanked out.
fn neutral(text: &str) -> String {
    let text = ENUMERATOR.replace(text, "#");
    let text = REFERENCE.replace_all(&text, "$1 #");
    LETTER.replace_all(&text, "#)").into_owned()
}

fn similarity(left: &str, right: &str) -> f64 {
    let words = |text: &str| -> HashSet<String> {
        let lower = text.to_lowercase();
        let mut words: HashSet<String> = lower.split_whitespace().map(str::to_owned).collect();
        if words.is_empty() {
            words.insert(String::new());
        }
        words
    };
    let (left, right) = (words(left), words(right));
    let shared = left.intersection(&right).count();
    shared as f64 / left.len().min(right.len()).max(1) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inserted(location: &str, text: &str) -> DiffItem {
        DiffItem::Inserted {
            location: Some(location.into()),
            text: text.into(),
        }
    }

    #[test]
    fn inserted_paragraphs_are_their_own_items() {
        let overview = diff_overview(Some(
            "### § 38g (CHANGED)\n\n(5) Beze změny.\n\n{++(6) Nově podá přiznání každý.++}\n\n{++(7) A také paušalista.++}",
        ));

        assert_eq!(
            overview,
            Some(DiffOverview {
                items: vec![
                    inserted("odst. 6", "(6) Nově podá přiznání každý."),
                    inserted("odst. 7", "(7) A také paušalista."),
                ],
                renumbering_only: false,
            })
        );
    }

    #[test]
    fn a_word_level_diff_realigns_into_a_deleted_and_a_renumbered_paragraph() {
        let overview = diff_overview(Some(
            &[
                "(2) Daní na vstupu se rozumí daň.",
                "(3) {--U--}{++Nárok++} {--pořízení automobilu se za daň považuje částka 420 000 Kč.--}{++vzniká okamžikem plnění.++}",
                "{--(4) Nárok vzniká okamžikem plnění.--}",
            ]
            .join("\n\n"),
        ))
        .unwrap();

        assert_eq!(overview.items.len(), 2);
        assert!(overview.items.contains(&DiffItem::Deleted {
            location: Some("odst. 3".into()),
            text: "(3) U pořízení automobilu se za daň považuje částka 420 000 Kč.".into(),
        }));
        assert!(overview.items.contains(&DiffItem::Renumbered {
            before: Some("odst. 4".into()),
            after: Some("odst. 3".into()),
            text: "(3) Nárok vzniká okamžikem plnění.".into(),
        }));
        assert!(!overview.renumbering_only);
    }

    #[test]
    fn a_change_of_letters_and_references_alone_is_renumbering() {
        let overview = diff_overview(Some(
            "d) plnění v § 54 odst. 1 písm. a) až {--j),--}{++i),++} {--l)--}{++k)++} až {--t)--}{++r)++}, nebo",
        ))
        .unwrap();

        assert_eq!(overview.items.len(), 1);
        assert!(matches!(
            &overview.items[0],
            DiffItem::Changed { location: Some(location), references_only: true, .. } if location == "písm. d)"
        ));
        assert!(overview.renumbering_only);
    }

    #[test]
    fn a_changed_amount_is_a_substantive_change() {
        let overview = diff_overview(Some(
            "(1) Přiznání podá, kdo měl příjmy nad {--50--}{++100++} 000 Kč.",
        ));

        assert_eq!(
            overview,
            Some(DiffOverview {
                items: vec![DiffItem::Changed {
                    location: Some("odst. 1".into()),
                    before: "(1) Přiznání podá, kdo měl příjmy nad 50 000 Kč.".into(),
                    after: "(1) Přiznání podá, kdo měl příjmy nad 100 000 Kč.".into(),
                    references_only: false,
                }],
                renumbering_only: false,
            })
        );
    }

    #[test]
    fn items_serialize_with_kind_and_where() {
        let item = DiffItem::Changed {
            location: None,
            before: "a".into(),
            after: "b".into(),
            references_only: true,
        };

        assert_eq!(
            serde_json::to_value(item).unwrap(),
            serde_json::json!({ "kind": "changed", "where": null, "before": "a", "after": "b", "referencesOnly": true })
        );
    }

    #[test]
    fn without_a_diff_there_is_nothing_to_say() {
        assert_eq!(diff_overview(None), None);
        assert_eq!(diff_overview(Some("")), None);
    }
}
