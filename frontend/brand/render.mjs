// Renders the logo options to PNG with headless Chrome (plan 75 Section 7.6). Used while the logo is
// chosen and again to export the chosen one.
//
//   node frontend/brand/render.mjs options [name]     renders every options/*.svg (or the ones whose name
//                                                      contains `name`): 1024, 128 and 32 px, on a light
//                                                      and a dark dock background, into renders/, and
//                                                      one sheet of all the options side by side
//   node frontend/brand/render.mjs export <svg> <png>  the 1024 by 1024 PNG with a transparent ground
//                                                      (the Mac rounded square with its margin)
//   node frontend/brand/render.mjs small <svg>         16, 32 and 64 px on a light and a dark ground, and one
//                                                      sheet that shows each at its own size and enlarged
//
// It needs Google Chrome (the path can be set in CHROME). Nothing here ships in the app.

import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readdirSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const CHROME =
  process.env.CHROME ?? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const SIZES = [1024, 128, 32];
const GROUNDS = { light: '#e8e8ed', dark: '#1d1d1f' };

function shoot(html, out, width, height, transparent = false) {
  const dir = mkdtempSync(path.join(tmpdir(), 'brand-'));
  const page = path.join(dir, 'page.html');
  writeFileSync(page, html);
  const args = [
    '--headless=new',
    '--disable-gpu',
    '--hide-scrollbars',
    '--force-device-scale-factor=1',
    `--window-size=${width},${height}`,
    ...(transparent ? ['--default-background-color=00000000'] : []),
    `--screenshot=${out}`,
    pathToFileURL(page).href,
  ];
  const run = spawnSync(CHROME, args, { encoding: 'utf8' });
  if (run.status !== 0) throw new Error(`Chrome failed for ${out}: ${run.stderr}`);
}

function wrap(svgFile, size, ground) {
  const background = ground === null ? 'transparent' : ground;
  return `<!doctype html><meta charset="utf-8"><style>html,body{margin:0;padding:0;background:${background}}
img{display:block;width:${size}px;height:${size}px}</style><img src="${pathToFileURL(svgFile).href}">`;
}

const [command, a, b] = process.argv.slice(2);
const only = command === 'options' ? a : undefined;

if (command === 'options') {
  const out = path.join(HERE, 'renders');
  mkdirSync(out, { recursive: true });
  const options = readdirSync(path.join(HERE, 'options')).filter((f) => f.endsWith('.svg'));
  for (const file of options.filter((f) => only === undefined || f.includes(only))) {
    const name = file.replace(/\.svg$/, '');
    for (const size of SIZES) {
      for (const [groundName, ground] of Object.entries(GROUNDS)) {
        shoot(
          wrap(path.join(HERE, 'options', file), size, ground),
          path.join(out, `${name}-${size}-${groundName}.png`),
          size,
          size,
        );
      }
    }
  }
  // One sheet with every option, so they can be compared side by side.
  const cell = (name, groundName) => `<div class="cell ${groundName}">
      <img class="big" src="${pathToFileURL(path.join(out, `${name}-1024-${groundName}.png`)).href}">
      <div class="row">
        <img src="${pathToFileURL(path.join(out, `${name}-128-${groundName}.png`)).href}">
        <img src="${pathToFileURL(path.join(out, `${name}-32-${groundName}.png`)).href}">
        <img class="zoom" src="${pathToFileURL(path.join(out, `${name}-32-${groundName}.png`)).href}">
      </div></div>`;
  const names = options.map((f) => f.replace(/\.svg$/, ''));
  const sheet = `<!doctype html><meta charset="utf-8"><style>
    body{margin:0;font:14px/1.3 -apple-system,sans-serif;background:#fff}
    .grid{display:grid;grid-template-columns:repeat(${names.length},1fr);gap:0}
    .cell{padding:20px;display:flex;flex-direction:column;align-items:center;gap:16px}
    .light{background:#e8e8ed;color:#222}.dark{background:#1d1d1f;color:#eee}
    .big{width:420px;height:420px}.row{display:flex;align-items:flex-end;gap:24px}
    .row img{width:auto}.zoom{width:128px!important;height:128px!important;image-rendering:pixelated}
    h2{margin:0;padding:10px 20px;font-size:16px}
  </style>
  <div class="grid">${names.map((n) => `<h2>${n}</h2>`).join('')}</div>
  <div class="grid">${names.map((n) => cell(n, 'light')).join('')}</div>
  <div class="grid">${names.map((n) => cell(n, 'dark')).join('')}</div>`;
  shoot(sheet, path.join(out, 'sheet.png'), 460 * names.length, 1250);
  console.log(`rendered ${options.length} options into ${out}`);
} else if (command === 'small' && a !== undefined) {
  const out = path.join(HERE, 'renders');
  mkdirSync(out, { recursive: true });
  const name = path.basename(a, '.svg');
  const file = path.resolve(a);
  const small = [16, 32, 64];
  for (const size of small) {
    for (const [groundName, ground] of Object.entries(GROUNDS)) {
      shoot(wrap(file, size, ground), path.join(out, `${name}-${size}-${groundName}.png`), size, size);
    }
  }
  const row = (groundName) => `<div class="row ${groundName}">${small
    .map((size) => {
      const src = pathToFileURL(path.join(out, `${name}-${size}-${groundName}.png`)).href;
      return `<div><img src="${src}"><img class="zoom" style="width:${size * 12}px;height:${size * 12}px" src="${src}"></div>`;
    })
    .join('')}</div>`;
  const sheet = `<!doctype html><meta charset="utf-8"><style>
    body{margin:0}.row{display:flex;gap:32px;align-items:flex-end;padding:24px}
    .light{background:#e8e8ed}.dark{background:#1d1d1f}
    div>img{display:block}.zoom{image-rendering:pixelated;margin-top:12px}</style>
    ${row('light')}${row('dark')}`;
  shoot(sheet, path.join(out, `${name}-small-check.png`), 1260, 1000);
  console.log(`rendered ${small.join(', ')} px of ${name}`);
} else if (command === 'export' && a !== undefined && b !== undefined) {
  shoot(wrap(path.resolve(a), 1024, null), path.resolve(b), 1024, 1024, true);
  console.log(`wrote ${path.resolve(b)}`);
} else {
  console.error('usage: node render.mjs options | node render.mjs export <svg> <png>');
  process.exit(1);
}
