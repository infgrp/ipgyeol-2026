// 다축 필터 칩(지역·트랙·전형). 클릭 토글로 다중 선택.

export function mountFilters(state, root, onChange) {
  const regions = unique(state.docs.map(d => d.region));
  const tracks = unique(state.docs.map(d => d.track ?? 'unknown'));
  const types = unique(state.docs.flatMap(d => d.selection_types || []));

  root.innerHTML = '';
  root.appendChild(group('지역', regions, state.filters.region, onChange));
  root.appendChild(group('전형 구분', tracks.map(trackLabel), state.filters.track, onChange, trackValue));
  root.appendChild(group('전형 종류', types, state.filters.selection_type, onChange));
}

function group(title, labels, selectedSet, onChange, valueOf) {
  const wrap = document.createElement('div');
  wrap.className = 'filter-group';
  wrap.style.display = 'flex';
  wrap.style.flexWrap = 'wrap';
  wrap.style.gap = '4px';
  wrap.style.alignItems = 'center';

  const head = document.createElement('span');
  head.textContent = title;
  head.style.fontSize = '12px';
  head.style.color = 'var(--muted)';
  head.style.marginRight = '4px';
  wrap.appendChild(head);

  for (const label of labels) {
    const value = valueOf ? valueOf(label) : label;
    const chip = document.createElement('button');
    chip.className = 'chip' + (selectedSet.has(value) ? ' active' : '');
    chip.textContent = label;
    chip.addEventListener('click', () => {
      if (selectedSet.has(value)) selectedSet.delete(value);
      else selectedSet.add(value);
      chip.classList.toggle('active');
      onChange();
    });
    wrap.appendChild(chip);
  }
  return wrap;
}

function unique(arr) {
  return [...new Set(arr.filter(Boolean))].sort();
}

function trackLabel(t) {
  if (t === 'susi') return '수시';
  if (t === 'jeongsi') return '정시';
  return '기타';
}
function trackValue(label) {
  if (label === '수시') return 'susi';
  if (label === '정시') return 'jeongsi';
  return 'unknown';
}
