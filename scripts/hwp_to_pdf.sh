#!/usr/bin/env bash
# data/raw 내 모든 .hwp / .hwpx 를 data/converted 로 PDF 변환.
# 권장: LibreOffice + hwp filter (한컴 제공) 또는 한컴오피스 매크로.
#
# LibreOffice가 설치되어 있고, hwp import filter가 활성화된 상태에서:
#   sudo apt-get install -y libreoffice
#   (한컴 hwp filter 별도 설치 — 배포판마다 다름)
#
# 다음 환경에서는 동작이 보장되지 않으니, 한컴오피스에서 [도구 → 일괄 변환] 사용을 권장한다.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")"/.. && pwd)"
SRC="$ROOT/data/raw"
DEST="$ROOT/data/converted"
mkdir -p "$DEST"

if ! command -v soffice >/dev/null 2>&1; then
  echo "[hwp_to_pdf] LibreOffice(soffice)를 찾지 못했습니다."
  echo "  - macOS: brew install --cask libreoffice"
  echo "  - Ubuntu: sudo apt-get install -y libreoffice"
  echo "  - 또는 한컴오피스 [도구 → 일괄 변환]을 사용하세요."
  exit 2
fi

count=0
while IFS= read -r -d '' f; do
  rel="${f#"$SRC/"}"
  outdir="$DEST/$(dirname "$rel")"
  mkdir -p "$outdir"
  echo " - 변환: $rel"
  soffice --headless --convert-to pdf --outdir "$outdir" "$f" >/dev/null
  count=$((count+1))
done < <(find "$SRC" -type f \( -iname '*.hwp' -o -iname '*.hwpx' \) -print0)

echo "[hwp_to_pdf] $count개 변환 완료. 결과: $DEST"
