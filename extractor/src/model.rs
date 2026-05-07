//! site/public/data.json의 직렬화 모델.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub schema_version: u32,
    pub generated_at: String,
    pub summary: Summary,
    pub documents: Vec<Document>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Summary {
    pub total_documents: usize,
    pub total_rows: usize,
    pub avg_confidence: f32,
    pub by_region: std::collections::BTreeMap<String, usize>,
    pub by_format: std::collections::BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub univ: String,
    pub region: String,
    pub year: Option<u32>,
    pub track: Option<Track>,
    pub selection_types: Vec<String>,
    pub source: Source,
    pub rows: Vec<Row>,
    pub ocr_text: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Track {
    Susi,   // 수시
    Jeongsi, // 정시
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub filename: String,
    pub format: String,        // pdf/xlsx/hwp/jpg/png/html
    pub url: String,           // 상대 경로 (사이트 기준)
    pub size_bytes: u64,
    pub official_url: Option<String>,
    pub license_status: LicenseStatus,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LicenseStatus {
    Confirmed,
    Pending,
    LinkOnly,
    Denied,
}

impl Default for LicenseStatus {
    fn default() -> Self { Self::Pending }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub department: String,
    pub selection: Option<String>,
    pub applicants: Option<u32>,
    pub competition_rate: Option<f32>,
    pub grade_50pct: Option<f32>,
    pub grade_70pct: Option<f32>,
    pub grade_avg: Option<f32>,
    pub grade_min: Option<f32>,
    pub raw_cells: Vec<String>,
    pub extraction_confidence: f32, // 0.0 ~ 1.0
    /// PDF preprocess 결과의 페이지 인덱스 (0-based). XLSX는 None.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
}
