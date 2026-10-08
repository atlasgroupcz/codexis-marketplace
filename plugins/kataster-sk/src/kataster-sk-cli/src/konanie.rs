//! Status of a cadastral proceeding (konanie) from the ESKN portal's public
//! "informovanie sa o stave spracovania podania" form.
//!
//! The portal has no JSON API for this: the page is a JSF form that is read
//! (session cookie + `ViewState` + workplace list) and submitted like a browser
//! would. Everything here is parsing and choosing; the requests themselves live
//! in [`crate::http`].

use anyhow::{anyhow, bail, Result};
use serde_json::{json, Value};

use crate::http::KatasterClient;

const FORM_ID: &str = "<form id=\"bodyForm\"";
const VIEW_STATE_NAME: &str = "name=\"javax.faces.ViewState\"";
const SELECT_ID: &str = "id=\"select_pracovisko_input\"";
const RESULT_ANCHOR: &str = "id=\"btn_zisti_stav_s\"";
const NOT_FOUND: &str = "Nenájdené";
const OFFICE_PREFIX: &str = "Okresný úrad ";
const OFFICE_SUFFIX: &str = ", katastrál";
const SNIPPET_MAX_CHARS: usize = 300;

#[derive(Debug, PartialEq)]
pub struct Pracovisko {
    pub id: u32,
    pub name: String,
}

#[derive(Debug)]
pub struct FormPage {
    pub view_state: String,
    pub pracoviska: Vec<Pracovisko>,
}

/// The portal's answer, field by field. `sentence` is always the portal's own
/// wording; the other fields are `None` when the sentence has an unknown shape.
#[derive(Debug, PartialEq)]
pub struct Konanie {
    pub cislo: String,
    pub pracovisko: Option<String>,
    pub najdene: Option<bool>,
    pub datum_podania: Option<String>,
    pub k_datu: Option<String>,
    pub stav: Option<String>,
    pub sentence: String,
}

impl Konanie {
    pub fn to_json(&self) -> Value {
        json!({
            "cislo": self.cislo,
            "pracovisko": self.pracovisko,
            "najdene": self.najdene,
            "datum_podania": self.datum_podania,
            "k_datu": self.k_datu,
            "stav": self.stav,
            "sprava": self.sentence,
        })
    }
}

/// `pracoviska [query]`: the workplaces the portal accepts, optionally filtered.
pub fn list_pracoviska(client: &KatasterClient, query: Option<&str>) -> Result<Value> {
    let page = parse_form_page(&client.portal_get()?.body)?;
    let needle = query.map(fold).filter(|q| !q.is_empty());
    let items: Vec<Value> = page
        .pracoviska
        .iter()
        .filter(|p| needle.as_ref().is_none_or(|n| fold(&p.name).contains(n)))
        .map(|p| json!({"id": p.id, "name": p.name}))
        .collect();
    Ok(Value::Array(items))
}

/// `konanie <pracovisko> <number>`: the status JSON plus the chat source for it.
pub fn lookup(
    client: &KatasterClient,
    pracovisko: &str,
    number: &str,
) -> Result<(Value, Vec<Value>)> {
    let cislo = normalize_cislo(number)?;
    let fetched = client.portal_get()?;
    let page = parse_form_page(&fetched.body)?;
    let office = resolve_pracovisko(&page.pracoviska, pracovisko)?;
    let id = office.id.to_string();
    let html = client.portal_post(
        &fetched.cookie,
        &[
            ("bodyForm", "bodyForm"),
            ("select_pracovisko_focus", ""),
            ("select_pracovisko_input", id.as_str()),
            ("kon-no", cislo.as_str()),
            ("btn_zisti_stav", ""),
            ("javax.faces.ViewState", page.view_state.as_str()),
        ],
    )?;
    let konanie = parse_result(&html)?;
    let sources = vec![source_entry(&client.portal_url(), &konanie)];
    Ok((konanie.to_json(), sources))
}

fn source_entry(url: &str, konanie: &Konanie) -> Value {
    let mut entry = serde_json::Map::new();
    entry.insert("__kind".into(), json!("url"));
    entry.insert("url".into(), json!(url));
    entry.insert(
        "title".into(),
        json!(format!("Stav konania {}", konanie.cislo)),
    );
    entry.insert(
        "snippet".into(),
        json!(konanie
            .sentence
            .chars()
            .take(SNIPPET_MAX_CHARS)
            .collect::<String>()),
    );
    Value::Object(entry)
}

/// `D-číslo/rok` with a register letter, e.g. `V-4500/2026`; returned upper-cased.
pub fn normalize_cislo(input: &str) -> Result<String> {
    let value = input.trim().to_uppercase();
    let valid = value.split_once('-').is_some_and(|(letter, rest)| {
        letter.len() == 1
            && letter.chars().all(|c| c.is_ascii_alphabetic())
            && rest.split_once('/').is_some_and(|(number, year)| {
                !number.is_empty()
                    && number.chars().all(|c| c.is_ascii_digit())
                    && year.len() == 4
                    && year.chars().all(|c| c.is_ascii_digit())
            })
    });
    if !valid {
        bail!("proceeding number must look like V-123/2026 (register letter, number, year), got '{input}'");
    }
    Ok(value)
}

pub fn parse_form_page(html: &str) -> Result<FormPage> {
    let form = html
        .find(FORM_ID)
        .ok_or_else(|| anyhow!("the ESKN status form was not found in the portal page"))?;
    let after_form = &html[form..];
    let view_state = after_form
        .find(VIEW_STATE_NAME)
        .and_then(|at| between(&after_form[at..], "value=\"", "\""))
        .ok_or_else(|| anyhow!("the ESKN status form has no ViewState"))?
        .to_string();

    let select = html
        .find(SELECT_ID)
        .map(|at| &html[at..])
        .and_then(|tail| tail.find("</select>").map(|end| &tail[..end]))
        .ok_or_else(|| anyhow!("the ESKN workplace list was not found in the portal page"))?;
    let pracoviska = parse_options(select);
    if pracoviska.is_empty() {
        bail!("the ESKN workplace list is empty");
    }
    Ok(FormPage {
        view_state,
        pracoviska,
    })
}

fn parse_options(select: &str) -> Vec<Pracovisko> {
    const OPEN: &str = "<option value=\"";
    let mut out = Vec::new();
    let mut rest = select;
    while let Some(at) = rest.find(OPEN) {
        rest = &rest[at + OPEN.len()..];
        let Some(quote) = rest.find('"') else { break };
        let id = &rest[..quote];
        let Some(name) = between(rest, ">", "<") else {
            break;
        };
        if let Ok(id) = id.parse::<u32>() {
            out.push(Pracovisko {
                id,
                name: decode_entities(name).trim().to_string(),
            });
        }
    }
    out
}

/// A numeric query is a workplace id; anything else matches names ignoring case
/// and diacritics, falling back to an exact short name ("Kosice" is not
/// "Košice-okolie").
pub fn resolve_pracovisko<'a>(list: &'a [Pracovisko], query: &str) -> Result<&'a Pracovisko> {
    let query = query.trim();
    if query.is_empty() {
        bail!("workplace is empty");
    }
    if let Ok(id) = query.parse::<u32>() {
        return list
            .iter()
            .find(|p| p.id == id)
            .ok_or_else(|| anyhow!("unknown workplace id {id} - list them with `pracoviska`"));
    }
    let needle = fold(query);
    let matches: Vec<&Pracovisko> = list
        .iter()
        .filter(|p| fold(&p.name).contains(&needle))
        .collect();
    match matches.as_slice() {
        [] => bail!("no workplace matches '{query}' - list them with `pracoviska`"),
        [one] => Ok(one),
        many => {
            let exact: Vec<&&Pracovisko> = many
                .iter()
                .filter(|p| fold(short_name(&p.name)) == needle)
                .collect();
            if let [one] = exact.as_slice() {
                return Ok(one);
            }
            let names: Vec<String> = many
                .iter()
                .map(|p| format!("{} {}", p.id, p.name))
                .collect();
            bail!(
                "'{query}' matches several workplaces, pass the id instead: {}",
                names.join("; ")
            )
        }
    }
}

fn short_name(name: &str) -> &str {
    let name = name.strip_prefix(OFFICE_PREFIX).unwrap_or(name);
    name.split(OFFICE_SUFFIX).next().unwrap_or(name)
}

pub fn parse_result(html: &str) -> Result<Konanie> {
    let tail = html
        .find(RESULT_ANCHOR)
        .map(|at| &html[at..])
        .unwrap_or(html);
    let sentence = tail
        .find("Konanie ")
        .and_then(|at| tail[at..].split('<').next())
        .map(|s| decode_entities(s).trim().to_string())
        .ok_or_else(|| {
            anyhow!("the portal returned no status - check the workplace and the number")
        })?;
    Ok(parse_sentence(&sentence))
}

/// Two shapes are known:
/// `Konanie V-1/2026 zo dňa 02.01.2026 je ku dňu 07.10.2026, na pracovisku X v stave: vybavené.`
/// `Konanie V-1/2026 na pracovisku X: Nenájdené.`
/// Anything else keeps only the portal's sentence.
fn parse_sentence(sentence: &str) -> Konanie {
    let mut konanie = Konanie {
        cislo: String::new(),
        pracovisko: None,
        najdene: None,
        datum_podania: None,
        k_datu: None,
        stav: None,
        sentence: sentence.to_string(),
    };
    let Some((cislo, rest)) = sentence
        .strip_prefix("Konanie ")
        .and_then(|s| s.split_once(' '))
    else {
        return konanie;
    };
    konanie.cislo = cislo.to_string();

    if let Some(rest) = rest.strip_prefix("na pracovisku ") {
        if let Some((office, stav)) = rest.rsplit_once(": ") {
            konanie.pracovisko = Some(office.to_string());
            let stav = stav.trim_end_matches('.').to_string();
            konanie.najdene = Some(stav != NOT_FOUND);
            konanie.stav = Some(stav);
        }
        return konanie;
    }

    let parsed = rest
        .strip_prefix("zo dňa ")
        .and_then(|r| r.split_once(" je ku dňu "))
        .and_then(|(podanie, r)| {
            let (k_datu, r) = r.split_once(", na pracovisku ")?;
            let (office, stav) = r.rsplit_once(" v stave: ")?;
            Some((podanie, k_datu, office, stav))
        });
    if let Some((podanie, k_datu, office, stav)) = parsed {
        konanie.datum_podania = Some(podanie.to_string());
        konanie.k_datu = Some(k_datu.to_string());
        konanie.pracovisko = Some(office.to_string());
        konanie.najdene = Some(true);
        konanie.stav = Some(stav.trim_end_matches('.').to_string());
    }
    konanie
}

/// The text between the first `start` and the next `end` after it.
fn between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let from = text.find(start)? + start.len();
    let len = text[from..].find(end)?;
    Some(&text[from..from + len])
}

fn decode_entities(text: &str) -> String {
    text.replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

/// Lower-case with Slovak diacritics removed, so "Kosice" finds "Košice".
fn fold(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'ä' => 'a',
            'č' => 'c',
            'ď' => 'd',
            'é' => 'e',
            'í' => 'i',
            'ĺ' | 'ľ' => 'l',
            'ň' => 'n',
            'ó' | 'ô' => 'o',
            'ŕ' => 'r',
            'š' => 's',
            'ť' => 't',
            'ú' => 'u',
            'ý' => 'y',
            'ž' => 'z',
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    const FORM_PAGE: &str = r#"<html><body>
<form id="topForm" name="topForm" method="post"><input type="hidden" name="javax.faces.ViewState" id="j_id1:javax.faces.ViewState:0" value="WRONG-STATE" autocomplete="off" /></form>
<form id="bodyForm" name="bodyForm" method="post" action="/eskn-portal/x">
<input type="hidden" name="bodyForm" value="bodyForm" />
<select id="select_pracovisko_input" name="select_pracovisko_input" tabindex="-1"><option value="" data-escape="true"></option><option value="109" data-escape="true">Okresný úrad Bratislava, katastrálny odbor</option><option value="801" data-escape="true">Okresný úrad Košice, katastrálny odbor</option><option value="802" data-escape="true">Okresný úrad Košice-okolie, katastrálny odbor</option><option value="803" data-escape="true">Okresný úrad Košice-okolie, katastrálny odbor, pracovisko Moldava nad Bodvou</option><option value="999" data-escape="true">Okresný úrad A &amp; B, katastrálny odbor</option></select>
<input type="hidden" name="javax.faces.ViewState" id="j_id1:javax.faces.ViewState:3" value="ASgAiSUK+MNJ/uk5o2==" autocomplete="off" />
</form>
<select id="filtertable:j_idt2118_input"><option value="7">Other select</option></select>
</body></html>"#;

    const FOUND_PAGE: &str = r#"<button id="btn_zisti_stav">Zistiť stav</button><script id="btn_zisti_stav_s" type="text/javascript">PrimeFaces.cw("CommandButton");</script></div><div class="b_green text-center f-bold">Konanie V-4500/2026 zo dňa 25.02.2026 je ku dňu 07.10.2026, na pracovisku Okresný úrad Bratislava, katastrálny odbor v stave: vybavené.</div></main>"#;

    const FOUND_SENTENCE: &str = "Konanie V-4500/2026 zo dňa 25.02.2026 je ku dňu 07.10.2026, na pracovisku Okresný úrad Bratislava, katastrálny odbor v stave: vybavené.";

    fn offices() -> Vec<Pracovisko> {
        parse_form_page(FORM_PAGE).unwrap().pracoviska
    }

    #[test]
    fn form_page_yields_view_state_of_body_form_and_workplaces() {
        let page = parse_form_page(FORM_PAGE).unwrap();
        assert_eq!(page.view_state, "ASgAiSUK+MNJ/uk5o2==");
        let ids: Vec<u32> = page.pracoviska.iter().map(|p| p.id).collect();
        assert_eq!(ids, vec![109, 801, 802, 803, 999]);
        assert_eq!(
            page.pracoviska[4].name,
            "Okresný úrad A & B, katastrálny odbor"
        );
    }

    #[test]
    fn form_page_without_the_form_is_an_error() {
        let err = parse_form_page("<html>blocked</html>").unwrap_err();
        assert!(err.to_string().contains("status form was not found"));
    }

    #[test]
    fn form_page_without_workplaces_is_an_error() {
        let html = FORM_PAGE.replace("select_pracovisko_input\"", "other_input\"");
        assert!(parse_form_page(&html).is_err());
    }

    #[test]
    fn found_proceeding_is_split_into_fields() {
        let konanie = parse_result(FOUND_PAGE).unwrap();
        assert_eq!(
            konanie,
            Konanie {
                cislo: "V-4500/2026".into(),
                pracovisko: Some("Okresný úrad Bratislava, katastrálny odbor".into()),
                najdene: Some(true),
                datum_podania: Some("25.02.2026".into()),
                k_datu: Some("07.10.2026".into()),
                stav: Some("vybavené".into()),
                sentence: FOUND_SENTENCE.into(),
            }
        );
    }

    #[test]
    fn missing_proceeding_is_a_result_not_an_error() {
        let html = r#"<script id="btn_zisti_stav_s"></script><div>Konanie V-99999/2026 na pracovisku Okresný úrad Bratislava, katastrálny odbor: Nenájdené.</div>"#;
        let konanie = parse_result(html).unwrap();
        assert_eq!(konanie.najdene, Some(false));
        assert_eq!(konanie.stav.as_deref(), Some("Nenájdené"));
        assert_eq!(konanie.datum_podania, None);
        assert_eq!(
            konanie.pracovisko.as_deref(),
            Some("Okresný úrad Bratislava, katastrálny odbor")
        );
    }

    #[test]
    fn unknown_sentence_shape_keeps_the_portal_wording() {
        let html = r#"<script id="btn_zisti_stav_s"></script><div>Konanie V-1/2026 sa momentálne spracúva.</div>"#;
        let konanie = parse_result(html).unwrap();
        assert_eq!(konanie.cislo, "V-1/2026");
        assert_eq!(konanie.najdene, None);
        assert_eq!(konanie.stav, None);
        assert_eq!(konanie.sentence, "Konanie V-1/2026 sa momentálne spracúva.");
    }

    #[test]
    fn page_without_a_result_is_an_error() {
        let err = parse_result(FORM_PAGE).unwrap_err();
        assert!(err.to_string().contains("returned no status"));
    }

    #[test]
    fn unknown_prefix_sentence_has_no_fields() {
        let konanie = parse_sentence("Podanie bez čísla");
        assert_eq!(konanie.cislo, "");
        assert_eq!(konanie.najdene, None);
    }

    #[test]
    fn json_carries_every_field() {
        let json = parse_result(FOUND_PAGE).unwrap().to_json();
        assert_eq!(json["stav"], "vybavené");
        assert_eq!(json["datum_podania"], "25.02.2026");
        assert_eq!(json["najdene"], true);
        assert_eq!(json["sprava"], FOUND_SENTENCE);
    }

    #[test]
    fn cislo_is_trimmed_and_upper_cased() {
        assert_eq!(normalize_cislo(" v-4500/2026 ").unwrap(), "V-4500/2026");
    }

    #[test]
    fn malformed_cislo_is_rejected() {
        for bad in [
            "",
            "xyz",
            "4500/2026",
            "V-4500",
            "V-/2026",
            "V-45a/2026",
            "V-1/26",
            "VV-1/2026",
            "1-1/2026",
        ] {
            assert!(normalize_cislo(bad).is_err(), "{bad} should be rejected");
        }
    }

    #[test]
    fn workplace_is_found_by_name_without_diacritics() {
        let list = offices();
        assert_eq!(resolve_pracovisko(&list, "bratislava").unwrap().id, 109);
    }

    #[test]
    fn workplace_is_found_by_id() {
        let list = offices();
        assert_eq!(resolve_pracovisko(&list, "802").unwrap().id, 802);
        assert!(resolve_pracovisko(&list, "1").is_err());
    }

    #[test]
    fn workplace_exact_short_name_beats_prefix_matches() {
        let list = offices();
        assert_eq!(resolve_pracovisko(&list, "Kosice").unwrap().id, 801);
    }

    #[test]
    fn ambiguous_workplace_lists_candidates() {
        let list = offices();
        let err = resolve_pracovisko(&list, "okolie").unwrap_err().to_string();
        assert!(err.contains("802") && err.contains("803"), "{err}");
    }

    #[test]
    fn unknown_or_blank_workplace_is_rejected() {
        let list = offices();
        assert!(resolve_pracovisko(&list, "Praha").is_err());
        assert!(resolve_pracovisko(&list, "  ").is_err());
    }

    #[test]
    fn fold_removes_slovak_diacritics() {
        assert_eq!(
            fold("Šaľa Žilina Čadca Trenčín Ďurďošík"),
            "sala zilina cadca trencin durdosik"
        );
        assert_eq!(fold("ÁäéíĺňóôŕťúýŤ"), "aaeilnoortuyt");
    }

    #[test]
    fn source_points_to_the_status_page_with_the_portal_sentence() {
        let konanie = parse_result(FOUND_PAGE).unwrap();
        let entry = source_entry("https://portal/x", &konanie);
        assert_eq!(entry["__kind"], "url");
        assert_eq!(entry["url"], "https://portal/x");
        assert_eq!(entry["title"], "Stav konania V-4500/2026");
        assert_eq!(entry["snippet"], FOUND_SENTENCE);
    }

    /// Serves `responses` in order on a local port and sends each raw request
    /// (headers + body) back through the channel.
    fn serve(responses: Vec<String>) -> (String, mpsc::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/status", listener.local_addr().unwrap());
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().unwrap();
                let mut raw = Vec::new();
                let mut buf = [0u8; 4096];
                loop {
                    let n = stream.read(&mut buf).unwrap();
                    raw.extend_from_slice(&buf[..n]);
                    let text = String::from_utf8_lossy(&raw).to_string();
                    if let Some(end) = text.find("\r\n\r\n") {
                        let length = text[..end]
                            .lines()
                            .find_map(|l| {
                                l.to_lowercase()
                                    .strip_prefix("content-length:")
                                    .map(|v| v.trim().parse::<usize>().unwrap())
                            })
                            .unwrap_or(0);
                        if raw.len() >= end + 4 + length {
                            break;
                        }
                    }
                    if n == 0 {
                        break;
                    }
                }
                tx.send(String::from_utf8_lossy(&raw).to_string()).unwrap();
                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        (url, rx)
    }

    fn http_response(set_cookies: &[&str], body: &str) -> String {
        let cookies: String = set_cookies
            .iter()
            .map(|c| format!("Set-Cookie: {c}\r\n"))
            .collect();
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html;charset=UTF-8\r\n{cookies}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    #[test]
    fn lookup_replays_session_cookie_and_view_state_in_the_submit() {
        let (url, requests) = serve(vec![
            http_response(
                &[
                    "JSESSIONID=abc123; path=/; secure; HttpOnly",
                    "TS01=lb9; Path=/; Domain=.x; Secure",
                ],
                FORM_PAGE,
            ),
            http_response(&[], FOUND_PAGE),
        ]);
        let client = KatasterClient::new().unwrap().with_portal_url(&url);

        let (json, sources) = lookup(&client, "Bratislava", "v-4500/2026").unwrap();

        assert_eq!(json["stav"], "vybavené");
        assert_eq!(json["datum_podania"], "25.02.2026");
        assert_eq!(sources[0]["url"], url.as_str());
        let get = requests.recv().unwrap();
        assert!(get.starts_with("GET /status"), "{get}");
        let post = requests.recv().unwrap().to_lowercase();
        assert!(post.starts_with("post /status"), "{post}");
        assert!(
            post.contains("cookie: jsessionid=abc123; ts01=lb9"),
            "{post}"
        );
        let body = post.split("\r\n\r\n").nth(1).unwrap();
        assert!(body.contains("select_pracovisko_input=109"), "{body}");
        assert!(body.contains("kon-no=v-4500%2f2026"), "{body}");
        assert!(body.contains("btn_zisti_stav="), "{body}");
        assert!(
            body.contains("javax.faces.viewstate=asgaisuk%2bmnj%2fuk5o2%3d%3d"),
            "{body}"
        );
    }

    #[test]
    fn lookup_rejects_a_bad_number_before_any_request() {
        let client = KatasterClient::new()
            .unwrap()
            .with_portal_url("http://127.0.0.1:1/never");
        let err = lookup(&client, "Bratislava", "xyz").unwrap_err();
        assert!(err.to_string().contains("must look like"), "{err}");
    }

    #[test]
    fn pracoviska_lists_and_filters_workplaces() {
        let (url, _requests) = serve(vec![
            http_response(&[], FORM_PAGE),
            http_response(&[], FORM_PAGE),
        ]);
        let client = KatasterClient::new().unwrap().with_portal_url(&url);

        let all = list_pracoviska(&client, None).unwrap();
        assert_eq!(all.as_array().unwrap().len(), 5);

        let filtered = list_pracoviska(&client, Some("KOSICE")).unwrap();
        let ids: Vec<u64> = filtered
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["id"].as_u64().unwrap())
            .collect();
        assert_eq!(ids, vec![801, 802, 803]);
    }
}
