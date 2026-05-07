// MiniSearch 인덱스 구성. 한국어 토크나이저 보강(부분일치 우선).

import MiniSearch from 'minisearch';

export function buildIndex(docs) {
  const index = new MiniSearch({
    idField: 'id',
    fields: ['univ', 'region', 'selection_types_joined', 'rows_text'],
    storeFields: ['univ', 'region'],
    // 한국어: 공백 + 구두점 + 한글 자모 분리 없는 단순 토크나이저.
    // 부분일치(prefix)는 search 시 옵션으로.
    tokenize: (text) => (text || '')
      .toLowerCase()
      .split(/[\s,.、。()\[\]·•/\\|]+/)
      .filter(Boolean),
    searchOptions: { boost: { univ: 3, selection_types_joined: 2 } },
  });

  const items = docs.map(d => ({
    id: d.id,
    univ: d.univ,
    region: d.region,
    selection_types_joined: (d.selection_types || []).join(' '),
    rows_text: (d.rows || []).map(r => r.department).join(' '),
  }));
  index.addAll(items);
  return index;
}
