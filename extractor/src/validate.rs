//! 추출 결과 sanity check. 비정상 값을 warnings로 누적.

use crate::model::{Document, Row};

pub fn check(doc: &mut Document) {
    let mut warnings = Vec::new();

    if doc.univ.is_empty() {
        warnings.push("대학명을 식별하지 못했습니다.".to_string());
    }
    if doc.year.is_none() {
        warnings.push("연도를 식별하지 못했습니다.".to_string());
    }

    let mut bad = 0usize;
    for r in &doc.rows {
        if !sanity_row(r) { bad += 1; }
    }
    if bad > 0 {
        warnings.push(format!("{bad}개 행이 sanity 검증 실패 (등급 범위·경쟁률 음수 등)."));
    }

    doc.warnings.extend(warnings);
}

fn sanity_row(r: &Row) -> bool {
    if let Some(c) = r.competition_rate { if !(0.0..=500.0).contains(&c) { return false; } }
    for g in [r.grade_50pct, r.grade_70pct, r.grade_avg, r.grade_min].into_iter().flatten() {
        if !(1.0..=9.5).contains(&g) { return false; }
    }
    if let Some(a) = r.applicants { if a > 1_000_000 { return false; } }
    true
}
