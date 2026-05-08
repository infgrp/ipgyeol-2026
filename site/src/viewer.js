// 인라인 뷰어. PDF/이미지/HTML/XLSX를 분기.

export function openViewer(doc) {
  const v = document.getElementById('viewer');
  const t = document.getElementById('viewer-title');
  const b = document.getElementById('viewer-body');
  if (!v || !t || !b) return;

  t.textContent = `${doc.univ} · ${doc.source.filename}`;
  b.innerHTML = '';

  // 한국어·공백이 포함된 경로를 각 세그먼트별로 인코딩
  const encodedPath = doc.source.url.split('/').map(encodeURIComponent).join('/');
  const url = `${import.meta.env.BASE_URL}${encodedPath}`;
  const fmt = doc.source.format;

  if (fmt === 'pdf') {
    const iframe = document.createElement('iframe');
    iframe.src = url + '#view=FitH';
    b.appendChild(iframe);
  } else if (fmt === 'jpg' || fmt === 'jpeg' || fmt === 'png') {
    const img = document.createElement('img');
    img.src = url;
    img.style.cssText = 'max-width:100%;height:auto;display:block;margin:0 auto;';
    b.appendChild(img);
  } else if (fmt === 'html' || fmt === 'htm') {
    const iframe = document.createElement('iframe');
    iframe.src = url;
    b.appendChild(iframe);
  } else {
    // xlsx/hwp 등 — 다운로드 링크
    const wrap = document.createElement('div');
    wrap.style.padding = '16px';
    wrap.innerHTML = `이 형식은 브라우저에서 직접 미리보기를 제공하지 않습니다.<br><br>
      <a href="${url}" download style="color:var(--accent)">${doc.source.filename} 다운로드</a>`;
    b.appendChild(wrap);
  }

  // 공식 페이지 링크 추가 (있는 경우)
  if (doc.source.official_url) {
    const bar = document.createElement('div');
    bar.style.cssText = 'padding:8px 14px;border-top:1px solid var(--line);font-size:13px;';
    bar.innerHTML = `공식 입학처: <a href="${doc.source.official_url}" target="_blank" rel="noopener noreferrer"
      style="color:var(--accent)">${doc.source.official_url}</a>`;
    b.appendChild(bar);
  }

  v.classList.remove('hidden');
  v.setAttribute('aria-hidden', 'false');
}

export function closeViewer() {
  const v = document.getElementById('viewer');
  if (!v) return;
  v.classList.add('hidden');
  v.setAttribute('aria-hidden', 'true');
}
