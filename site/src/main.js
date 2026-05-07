// 진입점: data.json 로드 → 검색 인덱스 구성 → UI 부착.

import { buildIndex } from './search.js';
import { mountFilters } from './filters.js';
import { renderCards } from './cards.js';
import { openViewer, closeViewer } from './viewer.js';

const state = {
  docs: [],
  filtered: [],
  index: null,
  query: '',
  filters: { region: new Set(), track: new Set(), selection_type: new Set() },
  mode: 'student',
};

async function loadData() {
  const url = `${import.meta.env.BASE_URL}data.json`;
  try {
    const res = await fetch(url);
    if (!res.ok) throw new Error(`data.json 로드 실패: ${res.status}`);
    return await res.json();
  } catch (e) {
    console.error(e);
    return { schema_version: 1, summary: {}, documents: [] };
  }
}

function applyFilters() {
  const q = state.query.trim();
  let candidates;
  if (q && state.index) {
    const hits = state.index.search(q, { prefix: true, fuzzy: 0.15 });
    const hitIds = new Set(hits.map(h => h.id));
    candidates = state.docs.filter(d => hitIds.has(d.id));
  } else {
    candidates = state.docs.slice();
  }
  const f = state.filters;
  const fRegion = f.region.size ? candidates.filter(d => f.region.has(d.region)) : candidates;
  const fTrack = f.track.size ? fRegion.filter(d => f.track.has(d.track ?? 'unknown')) : fRegion;
  const fSel = f.selection_type.size
    ? fTrack.filter(d => d.selection_types.some(s => f.selection_type.has(s)))
    : fTrack;
  state.filtered = fSel;
  renderSummary();
  renderCards(state, document.getElementById('results'), { onOpen: openViewer });
}

function renderSummary() {
  const el = document.getElementById('summary');
  if (!el) return;
  const total = state.docs.length;
  const shown = state.filtered.length;
  el.textContent = `총 ${total}개 자료 중 ${shown}개 표시`;
}

function bindUI() {
  document.getElementById('q').addEventListener('input', e => {
    state.query = e.target.value;
    applyFilters();
  });
  document.getElementById('viewer-close').addEventListener('click', closeViewer);
  window.addEventListener('hashchange', updateMode);
  updateMode();
}

function updateMode() {
  state.mode = location.hash === '#/pro' ? 'pro' : 'student';
  document.querySelectorAll('.mode-link').forEach(a => {
    a.classList.toggle('active', a.dataset.mode === state.mode);
  });
  applyFilters();
}

async function main() {
  const data = await loadData();
  state.docs = data.documents ?? [];
  state.index = buildIndex(state.docs);
  mountFilters(state, document.getElementById('filters'), applyFilters);
  bindUI();
  applyFilters();
}

main();
