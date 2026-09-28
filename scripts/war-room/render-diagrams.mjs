import { readdir, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import puppeteer from 'puppeteer';

// One browser for all diagrams. Mermaid is build-time only; the site ships SVG.
const directory = resolve('site/static/generated/maps');
const mermaid = resolve('node_modules/mermaid/dist/mermaid.min.js');
const browser = await puppeteer.launch({ headless: true, args: process.env.CI ? ['--no-sandbox'] : [] });
try {
  const page = await browser.newPage();
  await page.setViewport({ width: 1600, height: 1000 });
  await page.addScriptTag({ path: mermaid });
  await page.evaluate(() => window.mermaid.initialize({
    startOnLoad: false,
    htmlLabels: false,
    securityLevel: 'strict',
    theme: 'base',
    deterministicIds: true,
    deterministicIDSeed: 'stroggforge',
    fontFamily: 'monospace',
    themeVariables: {
      background: '#1b1425', primaryColor: '#30203f', primaryTextColor: '#eee5f6',
      primaryBorderColor: '#b28bd8', lineColor: '#bc9edf', secondaryColor: '#253932',
      tertiaryColor: '#20182b', clusterBkg: '#20182b', clusterBorder: '#6e4e8d',
      edgeLabelBackground: '#251c32', titleColor: '#eee5f6'
    },
    flowchart: { htmlLabels: false, useMaxWidth: true, curve: 'linear', wrappingWidth: 360 }
  }));
  const files = (await readdir(directory)).filter(file => file.endsWith('.mmd')).sort();
  for (const [index, file] of files.entries()) {
    const source = await readFile(resolve(directory, file), 'utf8');
    const svg = await page.evaluate(async ({ source, index }) => {
      await window.mermaid.parse(source);
      const { svg } = await window.mermaid.render(`diagram-${index}`, source);
      // Browsers load <img src="*.svg"> as strict XML. Mermaid can emit HTML-only markup such
      // as <br>, so round-trip through the DOM and require a clean XML parse.
      const host = document.createElement('div');
      host.innerHTML = svg;
      // Give the image its natural size so CSS can shrink wide maps without inflating small ones.
      const root = host.firstElementChild;
      const [, , width, height] = root.getAttribute('viewBox').split(/\s+/).map(Number);
      root.setAttribute('width', String(Math.ceil(width)));
      root.setAttribute('height', String(Math.ceil(height)));
      root.style.removeProperty('max-width');
      const xml = new XMLSerializer().serializeToString(root);
      const parsed = new DOMParser().parseFromString(xml, 'image/svg+xml');
      if (parsed.getElementsByTagName('parsererror').length) throw new Error('SVG is not well-formed XML');
      return xml;
    }, { source, index });
    await writeFile(resolve(directory, file.replace(/\.mmd$/, '.svg')), svg);
    console.log(`Rendered ${file}`);
  }
  if (!files.length) throw new Error('No generated diagrams; run the Rust generator first');
} finally {
  await browser.close();
}
