//! 파일명·경로에서 메타데이터(대학명·연도·전형)를 1차 추출.
//!
//! 본 저장소의 자료 명명 규칙은 다음과 같이 다양함:
//!   "한림대ㅣ2026 수시 입결.pdf"
//!   "동의대ㅣ2026학년도 수시모집 학생부교과, 실기실적 전형결과(최종).pdf"
//!   "영남대ㅣ2026학년도 정시모집 입학자 성적.xlsx"
//!   "2026 아주대 입결/KakaoTalk_20260420_162653595.jpg"
//!
//! 핵심 분리자:
//!   - 'ㅣ'(U+3163, 한글 호환자모) — 대학명과 본문 사이의 구분자로 빈번히 사용됨
//!   - 그 외 공백/하이픈/대괄호 등

use crate::model::Track;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct ParsedName {
    pub univ: Option<String>,
    pub year: Option<u32>,
    pub track: Track,
    pub selection_types: Vec<String>,
}

static RE_YEAR: Lazy<Regex> = Lazy::new(|| Regex::new(r"(20\d{2})").unwrap());

pub fn parse(path_components: &[String], filename_stem: &str) -> ParsedName {
    // 대학명: 1) 'ㅣ' 분리자 우선, 2) 폴더명에서 추측
    let univ = extract_univ(filename_stem)
        .or_else(|| guess_univ_from_components(path_components));

    // 연도
    let year = RE_YEAR.find(filename_stem).and_then(|m| m.as_str().parse().ok());

    // 트랙: 수시/정시 키워드
    let track = if filename_stem.contains("수시") {
        Track::Susi
    } else if filename_stem.contains("정시") {
        Track::Jeongsi
    } else {
        Track::Unknown
    };

    // 전형 분류
    let mut selection_types = Vec::new();
    let body = filename_stem;
    if body.contains("학생부교과") || body.contains("교과전형") || body.contains("교과형") {
        selection_types.push("학생부교과".to_string());
    }
    if body.contains("학생부종합") || body.contains("종합전형") || body.contains("종합형") {
        selection_types.push("학생부종합".to_string());
    }
    if body.contains("논술") {
        selection_types.push("논술".to_string());
    }
    if body.contains("실기") || body.contains("예체능") {
        selection_types.push("실기/예체능".to_string());
    }
    if body.contains("정시") && selection_types.is_empty() {
        selection_types.push("정시".to_string());
    }

    ParsedName { univ, year, track, selection_types }
}

fn extract_univ(stem: &str) -> Option<String> {
    // 'ㅣ' (U+3163) 분리자
    if let Some(idx) = stem.find('\u{3163}') {
        let head = stem[..idx].trim();
        if !head.is_empty() {
            return Some(normalize_univ(head));
        }
    }
    // 'ㅣ'가 없을 때 — 대표적으로 "OOOOO대" 패턴을 첫 토큰에서 시도
    let first_token = stem.split(|c: char| c.is_whitespace() || c == '_' || c == '-').next()?;
    if first_token.ends_with("대") || first_token.ends_with("교") {
        return Some(normalize_univ(first_token));
    }
    None
}

fn guess_univ_from_components(components: &[String]) -> Option<String> {
    // 가까운 폴더명에서 "OO대" 패턴 찾기 (예: "2026 아주대 입결")
    for comp in components.iter().rev() {
        let re = Regex::new(r"([가-힣A-Za-z]+(?:대학교|대))").unwrap();
        if let Some(m) = re.find(comp) {
            return Some(normalize_univ(m.as_str()));
        }
    }
    None
}

fn normalize_univ(s: &str) -> String {
    let s = s.trim();
    // "한림대학교" → "한림대" 정규화는 일부러 하지 않는다.
    // 사용자(교사)가 카드에서 어떤 표기를 보고싶은지 합의 후 별도 사전 적용 권장.
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_with_korean_separator() {
        let p = parse(&[], "한림대ㅣ2026 수시 입결");
        assert_eq!(p.univ.as_deref(), Some("한림대"));
        assert_eq!(p.year, Some(2026));
        assert_eq!(p.track, Track::Susi);
    }

    #[test]
    fn extracts_selection_types() {
        let p = parse(&[], "동의대ㅣ2026학년도 수시모집 학생부교과, 실기실적 전형결과(최종)");
        assert!(p.selection_types.contains(&"학생부교과".to_string()));
        assert!(p.selection_types.contains(&"실기/예체능".to_string()));
    }

    #[test]
    fn falls_back_to_folder() {
        let p = parse(
            &["경인".to_string(), "2026 아주대 입결".to_string()],
            "KakaoTalk_20260420_162653595",
        );
        assert_eq!(p.univ.as_deref(), Some("아주대"));
    }
}
