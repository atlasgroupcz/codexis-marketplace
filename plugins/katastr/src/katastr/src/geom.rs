//! The square around a definition point, for the Polygon endpoints.
//!
//! EPSG:5514 (S-JTSK) is metric, so the radius is added directly with no
//! conversion. The polygon must be closed - the last point repeats the first.
//!
//! Coordinates only. The path they are embedded in is built by `lookup`, like
//! every other path.

/// A closed square of radius `meters` around a point, as a JSON array of
/// points. The shape matches what `SeznamSouradnic` expects.
pub fn square_around(x: f64, y: f64, meters: f64) -> String {
    let pts: Vec<serde_json::Value> = [
        (x - meters, y - meters),
        (x + meters, y - meters),
        (x + meters, y + meters),
        (x - meters, y + meters),
        (x - meters, y - meters),
    ]
    .iter()
    .map(|(px, py)| serde_json::json!({ "x": px, "y": py }))
    .collect();
    serde_json::Value::Array(pts).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_has_five_points_and_is_closed() {
        let s = square_around(-494110.17, -1116432.13, 50.0);
        let v: serde_json::Value = serde_json::from_str(&s).expect("valid JSON");
        let arr = v.as_array().expect("array of points");
        assert_eq!(arr.len(), 5, "four corners plus the closing point");
        assert_eq!(arr[0], arr[4], "polygon must be closed");
    }

    #[test]
    fn square_extends_by_the_radius_in_both_axes() {
        let s = square_around(0.0, 0.0, 100.0);
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        let xs: Vec<f64> = v
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["x"].as_f64().unwrap())
            .collect();
        let ys: Vec<f64> = v
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["y"].as_f64().unwrap())
            .collect();
        assert_eq!(xs.iter().cloned().fold(f64::MIN, f64::max), 100.0);
        assert_eq!(xs.iter().cloned().fold(f64::MAX, f64::min), -100.0);
        assert_eq!(ys.iter().cloned().fold(f64::MIN, f64::max), 100.0);
        assert_eq!(ys.iter().cloned().fold(f64::MAX, f64::min), -100.0);
    }
}
