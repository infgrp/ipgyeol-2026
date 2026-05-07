#!/usr/bin/env bash
# 부모 디렉토리(또는 인자로 받은 경로)에서 8개 지역 폴더를 data/raw/로 복사.
# - 원본은 그대로 두고 복사함 (이동 아님). 안전 우선.
# - 이미 data/raw에 같은 이름의 디렉토리가 있으면 건너뜀.

set -euo pipefail

SRC="${1:-..}"
DEST="$(cd "$(dirname "$0")"/../data/raw && pwd)"

REGIONS=("강원" "경상" "경인" "교대" "서울" "전라 제주" "지방거점대" "충청")

echo "[init_data] SRC=$SRC"
echo "[init_data] DEST=$DEST"

for r in "${REGIONS[@]}"; do
  if [[ -d "$SRC/$r" ]]; then
    if [[ -d "$DEST/$r" ]]; then
      echo " - 건너뜀 (이미 존재): $r"
    else
      echo " - 복사: $r"
      cp -R "$SRC/$r" "$DEST/$r"
    fi
  else
    echo " - 없음 (skip): $r"
  fi
done

echo "[init_data] 완료. data/raw 내용:"
ls -la "$DEST"
