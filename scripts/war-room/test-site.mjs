import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import { resolve, extname, sep } from 'node:path';
import puppeteer from 'puppeteer';

const root = resolve('site/public');
const prefix = '/StroggForge/';
const server = createServer(async (request, response) => {
  try {
    let pathname = decodeURIComponent(new URL(request.url, 'http://localhost').pathname);
    assert(pathname.startsWith(prefix));
    pathname = pathname.slice(prefix.length);
    let file = resolve(root, pathname);
    assert(file === root || file.startsWith(root + sep));
    if ((await stat(file)).isDirectory()) file = resolve(file, 'index.html');
    const type = { '.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript', '.json': 'application/json', '.svg': 'image/svg+xml' }[extname(file)] ?? 'text/plain';
    response.writeHead(200, { 'Content-Type': type, 'Access-Control-Allow-Origin': '*' });
    response.end(await readFile(file));
  } catch { response.writeHead(404).end('Not found'); }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const origin = `http://127.0.0.1:${server.address().port}`;
const browser = await puppeteer.launch({ headless: true, args: process.env.CI ? ['--no-sandbox'] : [] });
try {
  const page = await browser.newPage();
  const failures = [];
  page.on('pageerror', error => failures.push(error.message));
  // Exercise production-base URLs while keeping every request local and offline.
  await page.setRequestInterception(true);
  page.on('request', request => {
    const url = request.url();
    if (url.startsWith('https://dreamweave-mp.github.io/StroggForge/')) {
      request.continue({ url: url.replace('https://dreamweave-mp.github.io', origin) });
    } else if (url.startsWith(origin) || url.startsWith('data:')) request.continue();
    else request.abort();
  });
  for (const width of [1440, 390]) {
    await page.setViewport({ width, height: 1000 });
    await page.goto(`${origin}${prefix}`, { waitUntil: 'networkidle0' });
    assert.equal(await page.$eval('h1', element => element.textContent), 'War Room');
    assert(await page.$('nav[aria-label="Engineering navigation"]'));
    assert(await page.$('a[href$="/stroggforge/troubleshooting/"]'));
    await page.$eval('.system-map img', image => image.scrollIntoView());
    await page.waitForFunction(() => document.querySelector('.system-map img').naturalWidth > 0);
    if (process.env.SCREENSHOT_DIRECTORY) {
      await page.screenshot({ path: resolve(process.env.SCREENSHOT_DIRECTORY, `war-room-${width}.png`), fullPage: true });
    }
    assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), `horizontal viewport overflow at ${width}`);
    if (width === 390) {
      assert.equal(await page.$eval('.docs-sidebar__panel', panel => panel.open), false);
      await page.click('.docs-sidebar__panel > summary');
      assert.equal(await page.$eval('.docs-sidebar__panel', panel => panel.open), true);
    }
    await page.keyboard.press('/');
    assert.equal(await page.evaluate(() => document.activeElement.id), 'search');
    await page.type('#search', 'L3i');
    await page.waitForSelector('.search-results__items a');
    await page.keyboard.press('ArrowDown');
    assert(await page.evaluate(() => document.activeElement.closest('.search-results') !== null));
    await page.keyboard.press('Escape');
    assert.equal(await page.$eval('#search-results', element => element.hidden), true);
  }
  // Every generated diagram must actually decode as an image, not merely exist on disk.
  for (const path of ['engineering/supply-chain/', 'archaeology/history/', 'ecosystem/map/', 'ecosystem/foundations/', 'releases/pipelines/']) {
    await page.goto(`${origin}${prefix}${path}`, { waitUntil: 'networkidle0' });
    for (const image of await page.$$('.system-map img')) {
      await image.scrollIntoView();
      await page.waitForFunction(element => element.complete, {}, image);
      assert(await image.evaluate(element => element.naturalWidth > 0), `diagram failed to decode on ${path}`);
    }
  }
  await page.goto(`${origin}${prefix}contributing/maintenance/`, { waitUntil: 'networkidle0' });
  assert(await page.$('.docs-code-copy'));
  assert(await page.$('.docs-sidebar a[aria-current="page"]'));
  await page.setJavaScriptEnabled(false);
  await page.goto(`${origin}${prefix}`, { waitUntil: 'networkidle0' });
  assert(await page.$eval('.docs-article', element => element.textContent.includes('Release pulse')));
  await page.$eval('.system-map img', image => image.scrollIntoView());
  await page.waitForFunction(() => document.querySelector('.system-map img').naturalWidth > 0);
  assert.deepEqual(failures, []);
  console.log('Desktop/mobile, keyboard/search, navigation, copy controls and no-JS reading passed');
} finally {
  await browser.close();
  server.close();
}
