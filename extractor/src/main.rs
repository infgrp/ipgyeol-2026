//! 입시결과 자료 추출 CLI.
//!
//! 사용 예:
//!   extractor scan ../data --out ../site/public/data.json
//!
//! 동작:
//! 1) data/raw/<region>/<file> 트리를 순회하며 PDF/XLSX/HWP/JPG/HTML을 식별
//! 2) PDF/XLSX는 텍스트·표 추출 시도, 그 외는 메타데이터만 기록
//! 3) data/meta.yaml의 license_status 등을 머지
//! 4) 결과를 단일 JSON 문서로 직렬화

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod model;
mod source_meta;
mod pdf;
mod xlsx;
mod normalize;
mod validate;
mod scan;

#[derive(Parser, Debug)]
#[command(name = "extractor", version, about = "입시결과 PDF/XLSX 추출기")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,

    /// 로그 레벨 (trace/debug/info/warn/error)
    #[arg(long, global = true, default_value = "info")]
    log: String,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// data/ 디렉토리를 스캔하여 data.json을 생성
    Scan {
        /// data/ 루트
        data_dir: PathBuf,
        /// 출력 JSON 경로 (예: site/public/data.json)
        #[arg(long)]
        out: PathBuf,
        /// 본문 추출까지 수행 (기본 true). false면 메타만.
        #[arg(long, default_value_t = true)]
        extract_body: bool,
    },
    /// 단일 파일 추출 결과를 stdout에 인쇄 (디버그용)
    Inspect {
        path: PathBuf,
    },
}

fn init_tracing(level: &str) {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_new(level).unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(&cli.log);

    match cli.cmd {
        Cmd::Scan { data_dir, out, extract_body } => {
            let report = scan::run(&data_dir, extract_body)
                .with_context(|| format!("scan 실패: {}", data_dir.display()))?;

            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            let json = serde_json::to_string_pretty(&report)?;
            std::fs::write(&out, json).with_context(|| format!("출력 실패: {}", out.display()))?;

            tracing::info!(
                "완료: {} 자료, {} 표(rows), 평균 신뢰도 {:.2}",
                report.documents.len(),
                report.documents.iter().map(|d| d.rows.len()).sum::<usize>(),
                report.summary.avg_confidence
            );
        }
        Cmd::Inspect { path } => {
            let doc = scan::inspect_one(&path)?;
            println!("{}", serde_json::to_string_pretty(&doc)?);
        }
    }
    Ok(())
}
