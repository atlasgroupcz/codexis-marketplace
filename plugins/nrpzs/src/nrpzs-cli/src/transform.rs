use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

use crate::util::norm;

/// Resolve an obor péče input (a code like "L42" or a name like "urologie")
/// to its code, using the obory-pece číselník. Exact code → exact name → substring.
pub fn obor_code(codelist: &[Value], input: &str) -> Option<String> {
    let want = input.trim();
    for item in codelist {
        if let Some(kod) = item.get("Kod").and_then(Value::as_str) {
            if kod.eq_ignore_ascii_case(want) {
                return Some(kod.to_string());
            }
        }
    }
    let nw = norm(want);
    for item in codelist {
        if let (Some(kod), Some(nazev)) = (
            item.get("Kod").and_then(Value::as_str),
            item.get("Nazev").and_then(Value::as_str),
        ) {
            if norm(nazev) == nw {
                return Some(kod.to_string());
            }
        }
    }
    for item in codelist {
        if let (Some(kod), Some(nazev)) = (
            item.get("Kod").and_then(Value::as_str),
            item.get("Nazev").and_then(Value::as_str),
        ) {
            if norm(nazev).contains(&nw) {
                return Some(kod.to_string());
            }
        }
    }
    None
}

/// Keep rows whose `code_field` equals `arg` (case-insensitive) or whose
/// `name_field` contains `arg` (accent-insensitive). For client-side kraj/okres
/// filtering - the NRPZS API has no such query parameter.
pub fn filter_region(
    rows: Vec<Value>,
    code_field: &str,
    name_field: &str,
    arg: &str,
) -> Vec<Value> {
    let a = arg.trim();
    let na = norm(a);
    rows.into_iter()
        .filter(|r| {
            if let Some(code) = r.get(code_field).and_then(Value::as_str) {
                if code.eq_ignore_ascii_case(a) {
                    return true;
                }
            }
            if let Some(name) = r.get(name_field).and_then(Value::as_str) {
                if !na.is_empty() && norm(name).contains(&na) {
                    return true;
                }
            }
            false
        })
        .collect()
}

/// Resolve a právní forma code to a Czech name.
pub fn pravni_forma_nazev(kod: &str) -> Option<&'static str> {
    Some(match kod {
        "101" => "fyzická osoba podnikající dle živnostenského zákona",
        "111" => "veřejná obchodní společnost",
        "112" => "společnost s ručením omezeným",
        "113" => "komanditní společnost",
        "121" => "akciová společnost",
        "205" => "družstvo",
        "301" => "státní podnik",
        "325" => "organizační složka státu",
        "331" => "příspěvková organizace",
        _ => return None,
    })
}

/// Resolve a typ osoby code to a Czech name.
pub fn typ_osoby_nazev(kod: &str) -> Option<&'static str> {
    Some(match kod {
        "1" => "fyzická osoba",
        "2" => "právnická osoba",
        _ => return None,
    })
}

/// Aggregate all `mista-poskytovani` rows of one IČO into a single profile.
pub fn profil(ico: &str, rows: &[Value]) -> Value {
    let first = rows.first();
    let s = |k: &str| {
        first
            .and_then(|r| r.get(k))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };

    let mut mista = BTreeSet::new();
    let mut zarizeni = BTreeSet::new();
    let mut obory = BTreeSet::new();
    let mut kraje = BTreeSet::new();

    let mut order: Vec<String> = Vec::new();
    let mut by_misto: BTreeMap<String, serde_json::Map<String, Value>> = BTreeMap::new();
    let mut obory_by_misto: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for r in rows {
        let get = |k: &str| r.get(k).and_then(Value::as_str).unwrap_or("");
        let mid = get("MistoPoskytovaniId").to_string();
        if !mid.is_empty() {
            mista.insert(mid.clone());
        }
        let zid = get("ZdravotnickeZarizeniId").to_string();
        if !zid.is_empty() {
            zarizeni.insert(zid);
        }
        let obor = get("OborPece");
        if !obor.is_empty() {
            obory.insert(obor.to_string());
        }
        let kraj = get("Kraj");
        if !kraj.is_empty() {
            kraje.insert(kraj.to_string());
        }

        if mid.is_empty() {
            continue;
        }
        if !by_misto.contains_key(&mid) {
            order.push(mid.clone());
            let adresa = format!(
                "{} {}, {} {}",
                get("Ulice"),
                get("CisloDomovniOrientacni"),
                get("Psc"),
                get("Obec")
            );
            let mut m = serde_json::Map::new();
            m.insert("mistoPoskytovaniId".into(), json!(mid));
            m.insert("zarizeniId".into(), json!(get("ZdravotnickeZarizeniId")));
            m.insert("nazev".into(), json!(get("NazevZarizeni")));
            m.insert("adresa".into(), json!(adresa.trim()));
            m.insert("kraj".into(), json!(get("Kraj")));
            m.insert("okres".into(), json!(get("Okres")));
            by_misto.insert(mid.clone(), m);
            obory_by_misto.insert(mid.clone(), BTreeSet::new());
        }
        if !obor.is_empty() {
            obory_by_misto
                .get_mut(&mid)
                .unwrap()
                .insert(obor.to_string());
        }
    }

    let provozovny: Vec<Value> = order
        .iter()
        .map(|mid| {
            let mut m = by_misto.get(mid).unwrap().clone();
            let o: Vec<Value> = obory_by_misto
                .get(mid)
                .unwrap()
                .iter()
                .map(|x| json!(x))
                .collect();
            m.insert("obory".into(), json!(o));
            Value::Object(m)
        })
        .collect();

    let pravni_forma_kod = s("PravniFormaKod");
    let typ_osoby = s("TypOsoby");
    json!({
        "ico": ico,
        "nazev": s("NazevZarizeni"),
        "pravniFormaKod": pravni_forma_kod,
        "pravniForma": pravni_forma_nazev(&pravni_forma_kod).map(serde_json::Value::from).unwrap_or(serde_json::Value::Null),
        "typOsoby": typ_osoby,
        "typOsobyNazev": typ_osoby_nazev(&typ_osoby).map(serde_json::Value::from).unwrap_or(serde_json::Value::Null),
        "pocetProvozoven": mista.len(),
        "pocetZarizeni": zarizeni.len(),
        "obory": obory.into_iter().collect::<Vec<_>>(),
        "kraje": kraje.into_iter().collect::<Vec<_>>(),
        "kontakt": {
            "telefon": s("PoskytovatelTelefon"),
            "email": s("PoskytovatelEmail"),
            "web": s("PoskytovatelWeb"),
            "fax": s("PoskytovatelFax"),
        },
        "provozovny": provozovny,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn codelist() -> Vec<serde_json::Value> {
        vec![
            json!({"Kod":"L15","Nazev":"chirurgie"}),
            json!({"Kod":"L42","Nazev":"urologie"}),
            json!({"Kod":"L12","Nazev":"gynekologie a porodnictví"}),
        ]
    }

    #[test]
    fn obor_code_matches_exact_code() {
        assert_eq!(obor_code(&codelist(), "L42").as_deref(), Some("L42"));
        assert_eq!(obor_code(&codelist(), "l42").as_deref(), Some("L42"));
    }

    #[test]
    fn obor_code_matches_name_accent_insensitive() {
        assert_eq!(obor_code(&codelist(), "Urologie").as_deref(), Some("L42"));
        assert_eq!(
            obor_code(&codelist(), "gynekologie").as_deref(),
            Some("L12")
        );
    }

    #[test]
    fn obor_code_unknown_is_none() {
        assert_eq!(obor_code(&codelist(), "zubař"), None);
    }

    fn rows() -> Vec<serde_json::Value> {
        vec![
            json!({"NazevZarizeni":"A","KrajCode":"CZ020","Kraj":"Středočeský kraj"}),
            json!({"NazevZarizeni":"B","KrajCode":"CZ010","Kraj":"Hlavní město Praha"}),
            json!({"NazevZarizeni":"C","KrajCode":"CZ020","Kraj":"Středočeský kraj"}),
        ]
    }

    #[test]
    fn filter_region_by_code() {
        let out = filter_region(rows(), "KrajCode", "Kraj", "CZ020");
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn filter_region_by_name_accent_insensitive() {
        let out = filter_region(rows(), "KrajCode", "Kraj", "stredocesky");
        assert_eq!(out.len(), 2);
        let out2 = filter_region(rows(), "KrajCode", "Kraj", "Praha");
        assert_eq!(out2.len(), 1);
    }

    fn provider_rows() -> Vec<serde_json::Value> {
        vec![
            json!({"Ico":"27085031","NazevZarizeni":"ON Příbram","PravniFormaKod":"121","TypOsoby":"2",
                   "MistoPoskytovaniId":"1","ZdravotnickeZarizeniId":"10","OborPece":"urologie",
                   "Kraj":"Středočeský kraj","Okres":"Příbram","Ulice":"Komenského","CisloDomovniOrientacni":"646",
                   "Psc":"26242","Obec":"Rožmitál","PoskytovatelTelefon":"+420318641123","PoskytovatelEmail":"x@onp.cz","PoskytovatelWeb":"http://onp.cz","PoskytovatelFax":""}),
            json!({"Ico":"27085031","NazevZarizeni":"ON Příbram","PravniFormaKod":"121","TypOsoby":"2",
                   "MistoPoskytovaniId":"1","ZdravotnickeZarizeniId":"10","OborPece":"chirurgie",
                   "Kraj":"Středočeský kraj","Okres":"Příbram","Ulice":"Komenského","CisloDomovniOrientacni":"646",
                   "Psc":"26242","Obec":"Rožmitál"}),
            json!({"Ico":"27085031","NazevZarizeni":"ON Příbram","MistoPoskytovaniId":"2","ZdravotnickeZarizeniId":"11",
                   "OborPece":"urologie","Kraj":"Středočeský kraj","Okres":"Příbram","Ulice":"Gen. Tesaříka","CisloDomovniOrientacni":"80","Psc":"26101","Obec":"Příbram"}),
        ]
    }

    #[test]
    fn profil_aggregates_counts_and_sets() {
        let p = profil("27085031", &provider_rows());
        assert_eq!(p["ico"], json!("27085031"));
        assert_eq!(p["pocetProvozoven"], json!(2));
        assert_eq!(p["pocetZarizeni"], json!(2));
        assert_eq!(p["obory"], json!(["chirurgie", "urologie"]));
        assert_eq!(p["kraje"], json!(["Středočeský kraj"]));
        assert_eq!(p["kontakt"]["telefon"], json!("+420318641123"));
        assert_eq!(p["provozovny"].as_array().unwrap().len(), 2);
        assert_eq!(
            p["provozovny"][0]["obory"],
            json!(["chirurgie", "urologie"])
        );
        assert_eq!(p["pravniForma"], json!("akciová společnost"));
        assert_eq!(p["typOsobyNazev"], json!("právnická osoba"));
    }

    #[test]
    fn pravni_forma_nazev_known_code() {
        assert_eq!(pravni_forma_nazev("121"), Some("akciová společnost"));
    }

    #[test]
    fn pravni_forma_nazev_unknown_code_is_none() {
        assert!(pravni_forma_nazev("999").is_none());
    }

    #[test]
    fn typ_osoby_nazev_known_code() {
        assert_eq!(typ_osoby_nazev("2"), Some("právnická osoba"));
    }
}
