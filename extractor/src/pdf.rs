//! PDF 처리 모듈 (재구성).
//!
//! 변경 이유: Rust 단독 PDF 표 추출은 한국어 입시결과 자료의 형식 편차로
//! 인해 신뢰성이 낮다(시뮬레이션 결과 36% 수준). 따라서 PDF의 행 추출은
//! Python 사전 처리기(`scripts/preprocess_pdf.py`, PyMuPDF blocks +
//! y-band 결합)에서 수행하고, 본 모듈은 그 결과 JSON을 읽어
//! `crate::model::Row`로 변환한다. 사전 처리 결과가 없으면 빈 행 + 경고를
//! 반환한다(원본 링크는 어쨌든 사이트에 노출됨).
//!
//! 기대 입력:
//!   data/preprocessed/<같은 상대경로>.rows.json
//! 입력 스키마:
//!   {
//!     "schema_version": 1,
//!     "source_rel": "강원/...",
//!     "page_count": 7,
//!     "rows": [ {department, applicants, competition_rate,
//!                grade_50pct, grade_70pct, grade_min,
//!                raw_line, extraction_confidence, page}, ... ],
//!     "warnings": [...]
//!   }

use crate::model::Row;
use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

pub struct PdfExtraction {
    pub rows: Vec<Row>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PreprocessFile {
    #[serde(default)]
    rows: Vec<PreprocessRow>,
    #[serde(default)]
    warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PreprocessRow {
    department: String,
    #[serde(default)]
    applicants: Option<u32>,
    #[serde(default)]
    competition_rate: Option<f32>,
    #[serde(default)]
    grade_50pct: Option<f32>,
    #[serde(default)]
    grade_70pct: Option<f32>,
    #[serde(default)]
    grade_min: Option<f32>,
    #[serde(default)]
    raw_line: String,
    #[serde(default)]
    extraction_confidence: f32,
    #[serde(default)]
    page: Option<u32>,
}

/// `pdf_path`는 `data/raw/...` 아래의 파일. 상응하는 preprocess JSON은
/// `data/preprocessed/<같은 상대경로>.rows.json`에 있어야 한다.
pub fn extract(pdf_path: &Path, raw_root: &Path, preprocess_root: &Path) -> Result<PdfExtraction> {
    let mut warnings = Vec::new();
    let rel = match pdf_path.strip_prefix(raw_root) {
        Ok(p) => p,
        Err(_) => {
            warnings.push(format!(
                "PDF 경로가 data/raw 하위가 아닙니다: {}",
                pdf_path.display()
            ));
            return Ok(PdfExtraction { rows: vec![], warnings });
        }
    };

    // <rel>.rows.json
    let mut json_path = preprocess_root.join(rel);
    let new_name = format!(
        "{}.rows.json",
        json_path.file_name().unwrap_or_default().to_string_lossy()
    );
    json_path.set_file_name(new_name);

    if !json_path.is_file() {
        warnings.push(format!(
            "preprocess 결과가 없습니다: {} — `python3 scripts/preprocess_pdf.py data` 실행 필요",
            json_path.display()
        ));
        return Ok(PdfExtraction { rows: vec![], warnings });
    }

    let text = std::fs::read_to_string(&json_path)?;
    let pre: PreprocessFile = match serde_json::from_str(&text) {
        Ok(p) => p,
        Err(e) => {
            warnings.push(format!("preprocess JSON 파싱 실패: {e}"));
            return Ok(PdfExtraction { rows: vec![], warnings });
        }
    };

    warnings.extend(pre.warnings);
    let rows: Vec<Row> = pre
        .rows
        .into_iter()
        .map(|r| Row {
            department: r.department,
            selection: None,
            applicants: r.applicants,
            competition_rate: r.competition_rate,
            grade_50pct: r.grade_50pct,
            grade_70pct: r.grade_70pct,
            grade_avg: None,
            grade_min: r.grade_min,
            raw_cells: if r.raw_line.is_empty() { vec![] } else { vec![r.raw_line] },
            extraction_confidence: r.extraction_confidence,
            page: r.page,
        })
        .collect();

    Ok(PdfExtraction { rows, warnings })
}
