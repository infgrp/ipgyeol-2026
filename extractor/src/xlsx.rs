//! XLSX 표 추출.
//!
//! 첫 시트의 모든 행을 읽고, "한글로 시작하는 모집단위 행"을 휴리스틱으로 골라
//! Row를 구성한다. 시트마다 헤더 위치/병합 셀이 다르므로 1차 자동화에 한정한다.

use crate::model::Row;
use anyhow::Result;
use calamine::{open_workbook_auto, Data, Reader};
use std::path::Path;

pub struct XlsxExtraction {
    pub rows: Vec<Row>,
    pub warnings: Vec<String>,
}

pub fn extract(path: &Path) -> Result<XlsxExtraction> {
    let mut warnings = Vec::new();
    let mut wb = match open_workbook_auto(path) {
        Ok(w) => w,
        Err(e) => {
            warnings.push(format!("XLSX 열기 실패: {e}"));
            return Ok(XlsxExtraction { rows: vec![], warnings });
        }
    };

    let sheets = wb.sheet_names().to_vec();
    let mut all_rows: Vec<Row> = Vec::new();

    for name in &sheets {
        let range = match wb.worksheet_range(name) {
            Ok(r) => r,
            Err(e) => {
                warnings.push(format!("시트 '{name}' 읽기 실패: {e}"));
                continue;
            }
        };
        for row_iter in range.rows() {
            let cells: Vec<String> = row_iter.iter().map(stringify).collect();
            if cells.is_empty() { continue; }

            // 모집단위 후보: 첫 셀이 한글로 시작
            let first = cells.iter().find(|c| !c.trim().is_empty());
            let Some(first) = first else { continue };
            let starts_korean = first.chars().next()
                .map(|c| ('가'..='힣').contains(&c)).unwrap_or(false);
            if !starts_korean { continue; }

            let nums: Vec<f32> = cells.iter()
                .filter_map(|c| c.replace(',', "").parse::<f32>().ok())
                .collect();
            if nums.len() < 2 { continue; }

            let mut grades: Vec<f32> = nums.iter().copied()
                .filter(|n| (1.0..=9.5).contains(n)).collect();
            grades.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let competition_rate = nums.iter().copied()
                .find(|n| *n >= 0.5 && *n < 1000.0 && n.fract() != 0.0
                    && !((1.0..=9.5).contains(n)));
            let applicants = nums.iter().copied()
                .find(|n| n.fract() == 0.0 && *n >= 10.0 && *n < 100000.0)
                .map(|n| n as u32);

            let row = Row {
                department: first.trim().to_string(),
                selection: cells.get(1).cloned().filter(|s| !s.is_empty()),
                applicants,
                competition_rate,
                grade_50pct: grades.first().copied(),
                grade_70pct: grades.get(1).copied(),
                grade_avg: grades.get(2).copied(),
                grade_min: grades.last().copied(),
                raw_cells: cells,
                extraction_confidence: if nums.len() >= 4 { 0.7 } else { 0.4 },
                page: None,
            };
            all_rows.push(row);
        }
    }

    if all_rows.is_empty() {
        warnings.push("XLSX 표 추출 실패: 데이터 행을 인식하지 못했습니다.".to_string());
    }

    Ok(XlsxExtraction { rows: all_rows, warnings })
}

fn stringify(d: &Data) -> String {
    match d {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => format!("{f}"),
        Data::Int(i) => format!("{i}"),
        Data::Bool(b) => format!("{b}"),
        Data::DateTime(dt) => format!("{dt}"),
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("#ERR:{e:?}"),
    }
}
