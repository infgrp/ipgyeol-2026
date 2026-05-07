// 인라인 뷰어. PDF/이미지/HTML/XLSX를 분기.
// PDF는 브라우저 내장 PDF 뷰어를 우선 사용(가장 가볍고 안정).

export function openViewer(doc) {
  const v = document.getElementById('viewer');
  const t = document.getElementById('viewer-title');
  const b = document.getElementById('viewer-body');
  if (!v || !t || !b) return;
  t.textContent = `${doc.univ} · ${doc.source.filename}`;
  b.innerHTML = '';

  const url = `${import.meta.env.BASE_URL}${doc.source.url}`;
  const fmt = doc.source.format;

  if (fmt === 'pdf') {
    const iframe = document.createElement('iframe');
    iframe.src = url + '#view=FitH';
    b.appendChild(iframe);
  } else if (fmt === 'jpg' || fmt === 'jpeg' || fmt === 'png') {
    const img = document.createElement('img');
    img.src = url;
    img.style.maxWidth = '100%';
    img.style.height = 'auto';
    img.style.display = 'block';
    img.style.margin = '0 auto';
    b.appendChild(img);
  } else if (fmt === 'html' || fmt === 'htm') {
    const iframe = document.createElement('iframe');
    iframe.src = url;
    b.appendChild(iframe);
  } else {
    // xlsx/hwp 등 — 다운로드 링크
    const p = document.createElement('p');
    p.style.padding = '16px';
    p.innerHTML = `이 형식은 브라우저에서 직접 미리보기를 제공하지 않습니다. <a href="${url}" download>${doc.source.filename} 다운로드</a>`;
    b.appendChild(p);
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
