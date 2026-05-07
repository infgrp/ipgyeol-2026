import { defineConfig } from 'vite';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
// 로컬 개발 시 ../data/raw/ 를 /data/raw/ 로 서빙 (원본 PDF 미리보기용).
// 프로덕션(GitHub Pages) 빌드에는 원본 파일이 포함되지 않는다.
const DATA_RAW = path.resolve(__dirname, '../data/raw');

export default defineConfig({
  // GitHub Pages 사용 시 SITE_BASE 환경변수로 저장소명을 전달한다.
  // 예: SITE_BASE=/ipgyeol-2026/ npm run build
  base: process.env.SITE_BASE ?? '/',
  build: {
    outDir: 'dist',
    sourcemap: false,
    target: 'es2020',
  },
  server: {
    port: 5173,
    open: false,
  },
  plugins: [
    {
      name: 'serve-data-raw',
      configureServer(server) {
        server.middlewares.use('/data/raw', (req, res, next) => {
          const decoded = decodeURIComponent(req.url ?? '/');
          const abs = path.join(DATA_RAW, decoded);
          // path traversal 방지
          if (!abs.startsWith(DATA_RAW + path.sep) && abs !== DATA_RAW) {
            next(); return;
          }
          if (fs.existsSync(abs) && fs.statSync(abs).isFile()) {
            const ext = path.extname(abs).toLowerCase();
            const mime = {
              '.pdf': 'application/pdf',
              '.xlsx': 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
              '.xls': 'application/vnd.ms-excel',
              '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg',
              '.png': 'image/png',
              '.html': 'text/html', '.htm': 'text/html',
            }[ext] ?? 'application/octet-stream';
            res.setHeader('Content-Type', mime);
            res.setHeader('Cache-Control', 'no-store');
            fs.createReadStream(abs).pipe(res);
          } else {
            next();
          }
        });
      },
    },
  ],
});
