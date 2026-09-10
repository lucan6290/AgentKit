// screenshot.cjs — 用 Chrome headless 把营销页各区块渲染成 PNG（供 README 引用）
// 运行：先在 site/ 起静态服务器（python -m http.server 7891），再在项目根目录执行 node site/screenshot.cjs
const { execFileSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// TODO: 按需调整 Chrome 路径 / 输出目录 / 端口 / 截图区块
const CHROME = "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe";
const ROOT = path.join(__dirname, '..');                        // 项目根目录
const OUT_DIR = path.join(ROOT, 'docs', 'picture');             // 输出到 docs/picture/
const PORT = 7891;
const BASE = `http://localhost:${PORT}/index.html`;
const DPR = 2;                                                  // 2x 分辨率
const BG = 'f3ece1';                                            // 与页面背景一致的兜底色

if (!fs.existsSync(OUT_DIR)) fs.mkdirSync(OUT_DIR, { recursive: true });

function shot(filename, shotParam, width, height) {
  const url = shotParam ? `${BASE}?shot=${shotParam}` : BASE;
  const outPath = path.join(OUT_DIR, filename);
  try {
    execFileSync(CHROME, [
      '--headless=new',
      '--disable-gpu',
      '--hide-scrollbars',
      `--screenshot=${outPath}`,
      `--window-size=${width},${height}`,
      `--force-device-scale-factor=${DPR}`,
      `--default-background-color=${BG}ff`,
      '--virtual-time-budget=2500',
      url
    ], { stdio: 'pipe', timeout: 30000 });
    const size = fs.statSync(outPath).size;
    console.log(`✓ ${filename}  (${(size / 1024).toFixed(0)} KB)`);
  } catch (e) {
    console.error(`✗ ${filename}:`, e.stderr ? e.stderr.toString().slice(0, 200) : e.message);
  }
}

console.log('=== Marketing screenshots ===\n');

// TODO: 替换为你的截图清单。shot 参数对应 index.html 里 ?shot=xxx 的区块映射。
shot('hero.png', 'hero', 1280, 720);
shot('overview.png', 'full', 1280, 2400);

console.log('\n=== Done. Files in docs/picture/: ===');
fs.readdirSync(OUT_DIR).forEach(f => console.log('  -', f));
