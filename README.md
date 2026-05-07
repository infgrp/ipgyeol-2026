# 2026 대학 입시결과 인덱스 (전국)

전국 대학 입학처에서 발행한 2026학년도 수시·정시 입시결과 자료를
지역별로 모아 검색·필터·열람할 수 있는 정적 웹사이트.

- **PDF 사전 처리**: Python (`scripts/preprocess_pdf.py`, PyMuPDF blocks + y-band 결합)
  → `data/preprocessed/<rel>.rows.json`
- **추출 파이프라인**: Rust CLI (`extractor/`) — XLSX 파싱 + preprocess JSON 머지 + 정합 검증
  → `site/public/data.json`
- **프런트엔드**: Vite + 바닐라 JS (`site/`) — MiniSearch 검색, PDF.js 인라인 뷰어
- **호스팅**: GitHub Pages + GitHub Actions 자동 빌드 (Python → Rust → Vite)

> Rust 단독으로 한국어 PDF 표를 정확히 추출하기는 어렵다(시뮬레이션 36% 수준).
> PyMuPDF의 좌표 기반 블록 추출이 사실상 표준이라 Python을 사전 단계에 둔다.
> 자세한 분석은 [`docs/extract-report-001.md`](./docs/extract-report-001.md) 참조.

> **추출 결과는 참고용이며, 모든 수치의 권위 있는 출처는 각 대학 입학처 원본 자료다.**
> 사이트는 항상 “원본 링크”를 함께 제공한다. 신뢰도가 낮은 셀은 회색·`?` 배지로 표시한다.

## 디렉터리 구조

```
ipgyeol-2026/
├── data/
│   ├── raw/         # 원본 (지역 폴더 그대로)
│   ├── converted/   # HWP→PDF 사전 변환본
│   ├── ocr/         # JPG OCR 텍스트
│   └── meta.yaml    # 자료별 출처/허가 상태
├── extractor/       # Rust CLI: data/ → site/public/data.json
├── site/            # Vite 정적 사이트
├── scripts/         # 사전 변환·초기화 스크립트
└── .github/workflows/build.yml
```

## 빠른 시작 (로컬)

### 1. 자료를 `data/raw/`로 이동

```bash
bash scripts/init_data.sh /path/to/원본폴더
```

원본폴더 = 부모 디렉터리(이 README의 `..`)이며, 8개 지역 폴더(강원/경상/…)를 포함한다.

### 2. HWP/JPG 사전 처리 (선택)

```bash
bash scripts/hwp_to_pdf.sh   # HWP → PDF (LibreOffice 또는 한컴오피스 필요)
bash scripts/ocr_jpg.sh      # JPG → 텍스트 (tesseract -l kor)
```

### 3-1. PDF 사전 처리 (Python, 필수)

Rust 추출기가 머지할 행 JSON을 생성한다. PyMuPDF가 필요하다.

```bash
pip install pymupdf
python3 scripts/preprocess_pdf.py data        # 새 PDF만 처리
python3 scripts/preprocess_pdf.py data --force # 전체 강제 재생성
```

생성 위치: `data/preprocessed/<지역>/<원본>.pdf.rows.json`

### 3-2. 추출 (Rust)

```bash
cd extractor
cargo run --release -- scan ../data --out ../site/public/data.json
```

Rust 추출기는 XLSX를 직접 파싱하고, PDF는 위 preprocess JSON을 머지한다.
preprocess JSON이 없으면 해당 자료는 “0행 + 경고”로 처리되며 원본 링크만 노출된다.

### 4. 사이트 빌드 & 미리보기

```bash
cd site
npm install
npm run dev      # 개발 서버
npm run build    # docs/ 또는 dist/ 산출
```

## 배포 (GitHub Pages)

`main` 브랜치에 푸시하면 `.github/workflows/build.yml`이
Rust 추출 → Vite 빌드 → Pages 배포까지 자동 수행한다.
저장소 설정에서 Pages 소스를 “GitHub Actions”로 지정해야 한다.

## 신뢰도 정책 (중요)

- 자동 추출의 정확도는 PDF 표 형식 편차 때문에 100%가 될 수 없다.
- 각 행에는 `extraction_confidence ∈ [0,1]`이 부여된다.
- 비정상 값(예: 등급>9, 경쟁률<0)은 자동 플래깅되어 `warnings`에 누적된다.
- UI는 신뢰도 낮은 셀을 회색 + `?` 배지로 표시하고 원본 링크를 우선 노출한다.

## 라이선스

- **사이트 코드 (`extractor/`, `site/`, `scripts/`, `.github/`)**: MIT (`LICENSE` 참조)
- **`data/` 하위 자료**: 각 대학 입학처에 저작권이 있다. 자세한 사항은 [`NOTICE.md`](./NOTICE.md) 참조.
