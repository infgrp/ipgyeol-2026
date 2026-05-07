//! Row의 후처리·정규화.

use crate::model::Row;

pub fn polish(rows: Vec<Row>) -> Vec<Row> {
    rows.into_iter()
        .filter(|r| !r.department.is_empty())
        .map(|mut r| {
            r.department = r.department.trim().to_string();
            // 등급이 0.0 또는 9.5를 초과하면 제거 (잡음)
            if let Some(g) = r.grade_50pct { if !is_grade(g) { r.grade_50pct = None; } }
            if let Some(g) = r.grade_70pct { if !is_grade(g) { r.grade_70pct = None; } }
            if let Some(g) = r.grade_avg   { if !is_grade(g) { r.grade_avg   = None; } }
            if let Some(g) = r.grade_min   { if !is_grade(g) { r.grade_min   = None; } }
            r
        })
        .collect()
}

fn is_grade(g: f32) -> bool { (1.0..=9.5).contains(&g) }

pub fn slug(univ: &str, year: Option<u32>, track: &crate::model::Track) -> String {
    use sha2::{Digest, Sha256};
    let track_str = match track {
        crate::model::Track::Susi => "susi",
        crate::model::Track::Jeongsi => "jeongsi",
        crate::model::Track::Unknown => "etc",
    };
    let basis = format!("{}|{}|{}", univ, year.unwrap_or(0), track_str);
    let mut h = Sha256::new();
    h.update(basis.as_bytes());
    let digest = h.finalize();
    format!("{}-{}-{}", track_str, year.unwrap_or(0), hex::encode(&digest[..4]))
}
