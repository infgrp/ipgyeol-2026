// 인라인 뷰어.
// 개발 서버(DEV): ../data/raw/ 미들웨어로 파일 직접 서빙.
// 프로덕션(GitHub Pages): 원본 파일 미포함 → official_url 연결 또는 안내 메시지.

const IS_DEV = import.meta.env.DEV;

export function openViewer(doc) {
  const v = document.getElementById('viewer');
  const t = document.getElementById('viewer-title');
  const b = document.getElementById('viewer-body');
  if (!v || !t || !b) return;

  t.textContent = `${doc.univ} · ${doc.source.filename}`;
  b.innerHTML = '';

  const url = `${import.meta.env.BASE_URL}${doc.source.url}`;
  const fmt = doc.source.format;

  if (IS_DEV) {
    // 로컬 개발: 파일 직접 표시
    _renderFile(b, fmt, url, doc);
  } else {
    // 프로덕션: 원본 파일 미포함 → 안내 또는 공식 페이지 이동
    if (doc.source.official_url) {
      window.open(doc.source.official_url, '_blank', 'noopener noreferrer');
      return; // 뷰어 패널 열지 않음
    }
    _renderUnavailable(b, doc);
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

function _renderFile(b, fmt, url, doc) {
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
    const p = document.createElement('p');
    p.style.padding = '16px';
    p.innerHTML = `이 형식은 직접 미리보기를 제공하지 않습니다. <a href="${url}" download>${doc.source.filename} 다운로드</a>`;
    b.appendChild(p);
  }
}

function _renderUnavailable(b, doc) {
  const wrap = document.createElement('div');
  wrap.style.cssText = 'padding:28px 24px;line-height:1.8;';

  wrap.innerHTML = `
    <p style="font-size:15px;font-weight:600;margin:0 0 8px;">원본 파일은 이 서버에서 제공되지 않습니다.</p>
    <p style="color:var(--muted);margin:0 0 16px;font-size:13px;">
      저작권 보호를 위해 원본 PDF/XLSX는 GitHub Pages에 포함되어 있지 않습니다.<br>
      각 대학 입학처 공식 홈페이지에서 직접 확인해 주세요.
    </p>
    ${doc.source.official_url
      ? `<a href="${doc.source.official_url}" target="_blank" rel="noopener noreferrer"
            style="display:inline-block;padding:8px 14px;border:1px solid var(--accent);
                   border-radius:6px;color:var(--accent);text-decoration:none;font-size:13px;">
           공식 입학처 페이지 열기 →
         </a>`
      : `<p style="color:var(--muted);font-size:12px;">
           공식 페이지 URL이 아직 등록되어 있지 않습니다.<br>
           <code>data/meta.yaml</code>의 <code>official_url</code> 필드에 추가하면 여기에 링크가 표시됩니다.
         </p>`
    }
  `;
  b.appendChild(wrap);
}
