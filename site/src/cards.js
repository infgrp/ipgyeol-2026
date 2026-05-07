// 결과 카드 렌더링. 학생 모드/교사 모드에서 표시 깊이가 다르다.

export function renderCards(state, root, { onOpen }) {
  root.innerHTML = '';
  if (state.filtered.length === 0) {
    const p = document.createElement('p');
    p.textContent = '조건에 해당하는 자료가 없습니다.';
    p.style.color = 'var(--muted)';
    root.appendChild(p);
    return;
  }
  for (const doc of state.filtered.slice(0, 200)) {
    root.appendChild(card(doc, state.mode, onOpen));
  }
  if (state.filtered.length > 200) {
    const note = document.createElement('p');
    note.textContent = `… 외 ${state.filtered.length - 200}개. 검색·필터로 좁혀 보세요.`;
    note.style.color = 'var(--muted)';
    root.appendChild(note);
  }
}

function card(doc, mode, onOpen) {
  const el = document.createElement('article');
  el.className = 'card';

  const trackLabel = doc.track === 'susi' ? '수시' : doc.track === 'jeongsi' ? '정시' : '기타';
  const h = document.createElement('h3');
  h.textContent = `${doc.univ} · ${doc.year ?? '?'} ${trackLabel}`;
  el.appendChild(h);

  const meta = document.createElement('div');
  meta.className = 'meta';
  meta.textContent = `${doc.region} · ${doc.source.format.toUpperCase()} · ${formatBytes(doc.source.size_bytes)}`;
  el.appendChild(meta);

  const badges = document.createElement('div');
  badges.className = 'badges';
  for (const t of doc.selection_types || []) {
    const b = document.createElement('span'); b.className = 'badge'; b.textContent = t; badges.appendChild(b);
  }
  if (doc.source.license_status === 'pending') {
    const b = document.createElement('span'); b.className = 'badge warn'; b.textContent = '허가 대기'; badges.appendChild(b);
  } else if (doc.source.license_status === 'link_only') {
    const b = document.createElement('span'); b.className = 'badge warn'; b.textContent = '링크만'; badges.appendChild(b);
  } else if (doc.source.license_status === 'denied') {
    const b = document.createElement('span'); b.className = 'badge bad'; b.textContent = '비공개'; badges.appendChild(b);
  }
  el.appendChild(badges);

  // 교사 모드에서 행 표시
  if (mode === 'pro' && doc.rows && doc.rows.length) {
    el.appendChild(rowsTable(doc.rows.slice(0, 10)));
  }

  if (doc.warnings && doc.warnings.length) {
    const w = document.createElement('div');
    w.className = 'meta';
    w.style.color = 'var(--warn)';
    w.textContent = `⚠ ${doc.warnings[0]}`;
    el.appendChild(w);
  }

  const actions = document.createElement('div');
  actions.className = 'actions';

  const openBtn = document.createElement('button');
  openBtn.textContent = '원본 열기';
  openBtn.disabled = doc.source.license_status === 'denied';
  openBtn.addEventListener('click', () => onOpen(doc));
  actions.appendChild(openBtn);

  if (doc.source.official_url) {
    const a = document.createElement('a');
    a.href = doc.source.official_url; a.target = '_blank'; a.rel = 'noopener';
    a.textContent = '공식 페이지';
    actions.appendChild(a);
  }
  el.appendChild(actions);
  return el;
}

function rowsTable(rows) {
  const t = document.createElement('table');
  const head = document.createElement('thead');
  head.innerHTML = '<tr><th>모집단위</th><th>경쟁률</th><th>50%</th><th>70%</th><th>최저</th></tr>';
  t.appendChild(head);
  const body = document.createElement('tbody');
  for (const r of rows) {
    const tr = document.createElement('tr');
    const conf = r.extraction_confidence ?? 0;
    const lc = conf < 0.5 ? 'low-conf' : '';
    tr.innerHTML = `
      <td>${escape(r.department || '')}</td>
      <td class="${lc}">${fmt(r.competition_rate)}</td>
      <td class="${lc}">${fmt(r.grade_50pct)}</td>
      <td class="${lc}">${fmt(r.grade_70pct)}</td>
      <td class="${lc}">${fmt(r.grade_min)}</td>
    `;
    body.appendChild(tr);
  }
  t.appendChild(body);
  return t;
}

function fmt(v) { return (v == null) ? '-' : (Number.isFinite(v) ? v.toFixed(2) : String(v)); }
function escape(s) { return s.replace(/[&<>]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c])); }
function formatBytes(b) {
  if (!b) return '?';
  const u = ['B', 'KB', 'MB', 'GB'];
  let i = 0; while (b >= 1024 && i < u.length - 1) { b /= 1024; i++; }
  return `${b.toFixed(b < 10 ? 1 : 0)} ${u[i]}`;
}
