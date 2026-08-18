use serde_json::{json, Value};

// NATURE_OF_LAND_USE_ID is the 1-10 sequential internal id of the KN code
// list ordered by the official kód druhu pozemku (vyhláška č. 461/2009 Z. z.,
// príloha 1). Mapping verified empirically per value by cross-referencing the
// same parcels between the ESKN ArcGIS identify service and the ESKN portal
// parcel search (2026-06); the layers expose no coded-value domains.
const DRUH_POZEMKU: [(u64, &str); 10] = [
    (2, "Orná pôda"),
    (3, "Chmeľnica"),
    (4, "Vinica"),
    (5, "Záhrada"),
    (6, "Ovocný sad"),
    (7, "Trvalý trávny porast"),
    (10, "Lesný pozemok"),
    (11, "Vodná plocha"),
    (13, "Zastavaná plocha a nádvorie"),
    (14, "Ostatná plocha"),
];

pub fn enrich_detail(body: &str) -> String {
    let Ok(mut value) = serde_json::from_str::<Value>(body) else {
        return body.to_string();
    };
    let Some(features) = value.get_mut("features").and_then(Value::as_array_mut) else {
        return body.to_string();
    };
    for feature in features {
        let Some(attributes) = feature.get_mut("attributes").and_then(Value::as_object_mut) else {
            continue;
        };
        let Some(id) = attributes
            .get("NATURE_OF_LAND_USE_ID")
            .and_then(Value::as_u64)
        else {
            continue;
        };
        if let Some(&(code, label)) = id.checked_sub(1).and_then(|i| DRUH_POZEMKU.get(i as usize)) {
            attributes.insert("DRUH_POZEMKU".into(), json!(label));
            attributes.insert("DRUH_POZEMKU_KOD".into(), json!(code));
        }
    }
    serde_json::to_string(&value).unwrap_or_else(|_| body.to_string())
}
