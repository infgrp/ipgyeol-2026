#!/usr/bin/env bash
# data/raw 내 JPG/PNG를 tesseract로 한국어 OCR 처리하여 data/ocr/<상대경로>.txt로 저장.
# 의존: tesseract(>=4) + 한국어 언어팩(kor)
#   macOS: brew install tesseract tesseract-lang
#   Ubuntu: sudo apt-get install -y tesseract-ocr tesseract-ocr-kor

set -euo pipefail

ROOT="$(cd "$(dirname "$0")"/.. && pwd)"
SRC="$ROOT/data/raw"
DEST="$ROOT/data/ocr"
mkdir -p "$DEST"

if ! command -v tesseract >/dev/null 2>&1; then
  echo "[ocr_jpg] tesseract를 찾지 못했습니다. 설치 후 다시 실행하세요."
  exit 2
fi

count=0
while IFS= read -r -d '' f; do
  rel="${f#"$SRC/"}"
  out="$DEST/${rel%.*}.txt"
  mkdir -p "$(dirname "$out")"
  echo " - OCR: $rel"
  tesseract "$f" "${out%.txt}" -l kor+eng --psm 6 >/dev/null 2>&1
  count=$((count+1))
done < <(find "$SRC" -type f \( -iname '*.jpg' -o -iname '*.jpeg' -o -iname '*.png' \) -print0)

echo "[ocr_jpg] $count개 OCR 완료. 결과: $DEST"
