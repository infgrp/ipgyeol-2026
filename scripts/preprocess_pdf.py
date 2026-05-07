#!/usr/bin/env python3
"""
PDF 사전 처리기 (PyMuPDF blocks + y-band 결합).

data/raw/<rel>.pdf  ->  data/preprocessed/<rel>.rows.json

각 출력 JSON 스키마:
{
  "schema_version": 1,
  "source_rel": "강원/한림대ㅣ2026 수시 입결.pdf",
  "page_count": 7,
  "rows": [
    {
      "department": "의예과",
      "applicants": 234,
      "competition_rate": 12.34,
      "grade_50pct": 1.12,
      "grade_70pct": 1.34,
      "grade_min": 1.50,
      "raw_line": "...",
      "extraction_confidence": 0.7,
      "page": 3
    },
    ...
  ],
  "warnings": [...]
}

이후 Rust 추출기(`extractor/`)가 위 JSON을 읽어 메타데이터·검증과 결합한다.

사용:
  python3 scripts/preprocess_pdf.py data
  # data/raw 트리를 스캔하여 data/preprocessed에 *.rows.json 생성
"""
from __future__ import annotations
import argparse, json, re, sys, time
from pathlib import Path

try:
    import fitz  # PyMuPDF
except ImportError:
    print("pip install pymupdf 필요", file=sys.stderr)
    sys.exit(2)

RE_NUM = re.compile(r"-?\d+(?:\.\d+)?")

# y-band 결합 임계값 (포인트 단위). 4pt가 기본값.
Y_BAND_PT = 4.0


def collect_lines(pdf_path: Path):
    """PDF의 각 페이지에서 (page_idx, line_text)의 리스트를 반환."""
    out = []
    with fitz.open(pdf_path) as doc:
        for pi, page in enumerate(doc):
            blocks = page.get_text("blocks")
            entries = []
            for b in blocks:
                if len(b) < 5:
                    continue
                x0, y0, _, _, txt = b[0], b[1], b[2], b[3], b[4]
                if not txt:
                    continue
                # 한 블록에 여러 줄이 들어올 수 있음
                for li, ln in enumerate(txt.splitlines()):
                    ln = ln.strip()
                    if ln:
                        entries.append((y0 + li * 0.01, x0, ln))
            entries.sort()
            cur_y = None
            cur = []
            for y, x, txt in entries:
                if cur_y is None or abs(y - cur_y) < Y_BAND_PT:
                    cur.append((x, txt))
                    if cur_y is None:
                        cur_y = y
                else:
                    cur.sort()
                    out.append((pi, " ".join(t for _, t in cur)))
                    cur = [(x, txt)]
                    cur_y = y
            if cur:
                cur.sort()
                out.append((pi, " ".join(t for _, t in cur)))
        return out, doc.page_count


def confidence(nums, grades):
    s = 0.0
    if len(nums) >= 4:
        s += 0.4
    elif len(nums) >= 2:
        s += 0.2
    if len(grades) >= 2:
        s += 0.3
    if grades and all(1.0 <= g <= 7.0 for g in grades):
        s += 0.2
    return min(1.0, s)


def line_to_row(page, line):
    line = line.strip()
    if len(line) < 4:
        return None
    if not ('가' <= line[0] <= '힣'):
        return None
    nums_str = RE_NUM.findall(line)
    nums = []
    for s in nums_str:
        try:
            nums.append(float(s))
        except ValueError:
            pass
    if len(nums) < 2:
        return None
    m = RE_NUM.search(line)
    if not m:
        return None
    dept = line[:m.start()].strip()
    if not dept or len(dept) > 30:
        return None
    grades = []
    comp = None
    appl = None
    for n in nums:
        if 1.0 <= n <= 9.5:
            grades.append(n)
        elif n.is_integer() and 10 <= n < 100000 and appl is None:
            appl = int(n)
        elif comp is None and 0.5 <= n < 1000 and not (1.0 <= n <= 9.5):
            comp = n
    return {
        "department": dept,
        "applicants": appl,
        "competition_rate": comp,
        "grade_50pct": grades[0] if grades else None,
        "grade_70pct": grades[1] if len(grades) > 1 else None,
        "grade_min": grades[-1] if grades else None,
        "raw_line": line,
        "extraction_confidence": confidence(nums, grades),
        "page": page,
    }


def process_pdf(pdf_path: Path):
    warnings = []
    try:
        lines, page_count = collect_lines(pdf_path)
    except Exception as e:
        return {
            "schema_version": 1,
            "page_count": 0,
            "rows": [],
            "warnings": [f"PDF 처리 오류: {e}"],
        }
    if not lines:
        return {
            "schema_version": 1,
            "page_count": page_count,
            "rows": [],
            "warnings": ["PDF가 이미지/스캔본으로 보입니다 (텍스트 0)."],
        }
    rows = []
    for pi, ln in lines:
        r = line_to_row(pi, ln)
        if r is not None:
            rows.append(r)
    if not rows:
        warnings.append("표 추출 실패: 모집단위 행을 인식하지 못했습니다.")
    return {
        "schema_version": 1,
        "page_count": page_count,
        "rows": rows,
        "warnings": warnings,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("data_dir", type=Path,
                    help="data/ 루트 (raw/ preprocessed/ 부모)")
    ap.add_argument("--force", action="store_true",
                    help="기존 preprocessed JSON을 강제 재생성")
    args = ap.parse_args()

    raw_dir = args.data_dir / "raw"
    pre_dir = args.data_dir / "preprocessed"
    if not raw_dir.is_dir():
        print(f"data/raw가 없습니다: {raw_dir}", file=sys.stderr)
        sys.exit(2)
    pre_dir.mkdir(parents=True, exist_ok=True)

    t0 = time.time()
    n_total = n_skipped = n_written = n_failed = 0
    n_rows = 0
    for path in sorted(raw_dir.rglob("*.pdf")):
        n_total += 1
        rel = path.relative_to(raw_dir)
        # _files 등 자원 폴더는 스킵
        if any(p.endswith("_files") for p in rel.parts[:-1]):
            continue
        out_path = pre_dir / (str(rel) + ".rows.json")
        if out_path.exists() and not args.force:
            n_skipped += 1
            continue
        out_path.parent.mkdir(parents=True, exist_ok=True)
        result = process_pdf(path)
        result["source_rel"] = str(rel).replace("\\", "/")
        with out_path.open("w", encoding="utf-8") as f:
            json.dump(result, f, ensure_ascii=False, indent=2)
        n_written += 1
        n_rows += len(result["rows"])
        if not result["rows"]:
            n_failed += 1
        sys.stderr.write(f". {rel} -> {len(result['rows'])} rows\n")

    elapsed = time.time() - t0
    print()
    print("=== preprocess_pdf 완료 ===")
    print(f"  총 PDF: {n_total}, 작성 {n_written}, 스킵 {n_skipped}")
    print(f"  추출 행 합계: {n_rows}, 0행 PDF: {n_failed}")
    print(f"  소요: {elapsed:.1f}s")
    print(f"  출력: {pre_dir}")


if __name__ == "__main__":
    main()
