//! data/raw 트리 순회 + 파일별 추출 결과를 Report로 결합.

use crate::model::{Document, LicenseStatus, Report, Source, Summary};
use crate::{normalize, pdf, source_meta, validate, xlsx};
use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Deserialize, Default)]
struct MetaYaml {
    /// key: data/raw 기준 상대 경로, value: 자료별 메타 오버라이드
    #[serde(default)]
    files: BTreeMap<String, FileMetaOverride>,
}

#[derive(Debug, Deserialize, Default, Clone)]
struct FileMetaOverride {
    #[serde(default)]
    univ: Option<String>,
    #[serde(default)]
    official_url: Option<String>,
    #[serde(default)]
    license_status: Option<String>,
    #[serde(default)]
    note: Option<String>,
}

pub fn run(data_dir: &Path, extract_body: bool) -> Result<Report> {
    let raw_dir = data_dir.join("raw");
    if !raw_dir.is_dir() {
        anyhow::bail!("data/raw 디렉토리를 찾을 수 없습니다: {}", raw_dir.display());
    }
    let preprocess_dir = data_dir.join("preprocessed");
    if !preprocess_dir.is_dir() {
        tracing::warn!(
            "data/preprocessed 디렉토리가 없습니다 — PDF 추출 결과가 비어 있을 수 있습니다. \
             scripts/preprocess_pdf.py를 먼저 실행하세요."
        );
    }

    let meta = load_meta(data_dir);
    let mut documents: Vec<Document> = Vec::new();

    for entry in WalkDir::new(&raw_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() { continue; }

        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        let format = match ext.as_str() {
            "pdf" | "xlsx" | "xls" | "hwp" | "hwpx" |
            "jpg" | "jpeg" | "png" | "html" | "htm" => ext.clone(),
            _ => continue,
        };

        let rel = path.strip_prefix(&raw_dir).unwrap_or(path);
        let key = rel.to_string_lossy().replace('\\', "/");
        let override_meta = meta.files.get(&key).cloned().unwrap_or_default();

        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let parents: Vec<String> = rel.parent()
            .map(|p| p.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect())
            .unwrap_or_default();
        let parsed = source_meta::parse(&parents, stem);

        let region = parents.first().cloned().unwrap_or_else(|| "기타".to_string());
        let univ = override_meta.univ
            .clone()
            .or(parsed.univ.clone())
            .unwrap_or_else(|| "(미식별)".to_string());

        let size_bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        let url = format!("data/raw/{key}");

        let license_status = match override_meta.license_status.as_deref() {
            Some("confirmed") => LicenseStatus::Confirmed,
            Some("link_only") => LicenseStatus::LinkOnly,
            Some("denied") => LicenseStatus::Denied,
            _ => LicenseStatus::Pending,
        };

        let mut rows = Vec::new();
        let mut warnings = Vec::new();

        if extract_body && license_status != LicenseStatus::Denied {
            match format.as_str() {
                "pdf" => {
                    match pdf::extract(path, &raw_dir, &preprocess_dir) {
                        Ok(r) => {
                            warnings.extend(r.warnings);
                            rows.extend(r.rows);
                        }
                        Err(e) => warnings.push(format!("PDF 처리 오류: {e}")),
                    }
                }
                "xlsx" | "xls" => {
                    match xlsx::extract(path) {
                        Ok(r) => {
                            warnings.extend(r.warnings);
                            rows.extend(r.rows);
                        }
                        Err(e) => warnings.push(format!("XLSX 처리 오류: {e}")),
                    }
                }
                "hwp" | "hwpx" => {
                    warnings.push("HWP는 사전 변환(scripts/hwp_to_pdf.sh) 후 PDF로 처리됩니다.".to_string());
                }
                "jpg" | "jpeg" | "png" => {
                    warnings.push("이미지 자료. OCR 결과는 data/ocr/에서 머지될 수 있습니다.".to_string());
                }
                "html" | "htm" => {
                    warnings.push("HTML 자료는 본 추출기에서 미지원. 링크만 노출.".to_string());
                }
                _ => {}
            }
        }

        rows = normalize::polish(rows);

        let mut doc = Document {
            id: normalize::slug(&univ, parsed.year, &parsed.track),
            univ,
            region,
            year: parsed.year,
            track: Some(parsed.track),
            selection_types: parsed.selection_types,
            source: Source {
                filename: path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string(),
                format,
                url,
                size_bytes,
                official_url: override_meta.official_url,
                license_status,
                note: override_meta.note,
            },
            rows,
            ocr_text: None,
            warnings,
        };
        validate::check(&mut doc);
        documents.push(doc);
    }

    Ok(Report {
        schema_version: 1,
        generated_at: chrono_now_iso(),
        summary: build_summary(&documents),
        documents,
    })
}

pub fn inspect_one(path: &Path) -> Result<Document> {
    // 단일 파일 디버그용 — data 트리 외부 파일도 허용
    let parent_components: Vec<String> = path.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let parsed = source_meta::parse(&parent_components, stem);

    let format = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let mut rows = Vec::new();
    let mut warnings = Vec::new();
    match format.as_str() {
        "pdf" => {
            warnings.push(
                "inspect는 단일 파일 검사용입니다. PDF는 \
                 `python3 scripts/preprocess_pdf.py data` 실행 후 \
                 `extractor scan data --out ...`로 처리하세요."
                    .to_string(),
            );
        }
        "xlsx" | "xls" => {
            let r = xlsx::extract(path)?;
            warnings.extend(r.warnings);
            rows.extend(r.rows);
        }
        _ => warnings.push(format!("미지원 포맷: {format}")),
    }
    rows = normalize::polish(rows);

    let mut doc = Document {
        id: normalize::slug(parsed.univ.as_deref().unwrap_or("?"), parsed.year, &parsed.track),
        univ: parsed.univ.unwrap_or_default(),
        region: "(inspect)".into(),
        year: parsed.year,
        track: Some(parsed.track),
        selection_types: parsed.selection_types,
        source: Source {
            filename: path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string(),
            format,
            url: path.display().to_string(),
            size_bytes: std::fs::metadata(path).map(|m| m.len()).unwrap_or(0),
            official_url: None,
            license_status: LicenseStatus::Pending,
            note: None,
        },
        rows,
        ocr_text: None,
        warnings,
    };
    validate::check(&mut doc);
    Ok(doc)
}

fn build_summary(docs: &[Document]) -> Summary {
    let mut by_region: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_format: BTreeMap<String, usize> = BTreeMap::new();
    let mut total_rows = 0usize;
    let mut conf_sum = 0f32;
    let mut conf_cnt = 0u32;

    for d in docs {
        *by_region.entry(d.region.clone()).or_default() += 1;
        *by_format.entry(d.source.format.clone()).or_default() += 1;
        total_rows += d.rows.len();
        for r in &d.rows {
            conf_sum += r.extraction_confidence;
            conf_cnt += 1;
        }
    }

    Summary {
        total_documents: docs.len(),
        total_rows,
        avg_confidence: if conf_cnt == 0 { 0.0 } else { conf_sum / conf_cnt as f32 },
        by_region,
        by_format,
    }
}

fn load_meta(data_dir: &Path) -> MetaYaml {
    let p: PathBuf = data_dir.join("meta.yaml");
    let Ok(text) = std::fs::read_to_string(&p) else { return MetaYaml::default(); };
    serde_yaml::from_str(&text).unwrap_or_default()
}

// 외부 의존 chrono를 추가하지 않기 위한 단순 ISO8601 (UTC) 생성.
fn chrono_now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    // 정밀 timestamp만; 표시는 사이트 측에서 포맷
    format!("unix:{secs}")
}
