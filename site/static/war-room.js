// The war room's page script: local search over Zola's static index, its `/` shortcut, and copy
// buttons on code blocks. The imported docs.js only runs the docs shell's drawers and contents.
// All navigation works without either.
(() => {
  const script = document.currentScript;
  const search = document.getElementById('search');
  if (!search || !script?.dataset.index) return;
  const results = document.getElementById('search-results');
  const status = results.querySelector('.search-results__header');
  const list = results.querySelector('.search-results__items');
  let index;
  let request = 0;

  search.addEventListener('input', async () => {
    const current = ++request;
    const query = search.value.trim().toLocaleLowerCase();
    list.replaceChildren();
    results.hidden = !query;
    if (!query) return;
    status.textContent = 'Searching…';
    try {
      index ??= fetch(script.dataset.index).then(response => {
        if (!response.ok) throw new Error('Search index unavailable');
        return response.json();
      }).catch(error => { index = undefined; throw error; });
      const records = await index;
      if (current !== request || search.value.trim().toLocaleLowerCase() !== query) return;
      const root = search.dataset.searchRoot;
      const words = query.split(/\s+/);
      const matches = records.filter(record => record.url.startsWith(root) &&
        words.every(word => `${record.title} ${record.body} ${record.description ?? ''}`.toLocaleLowerCase().includes(word)))
        .sort((a, b) => Number(b.title.toLocaleLowerCase().includes(query)) - Number(a.title.toLocaleLowerCase().includes(query)))
        .slice(0, 15);
      status.textContent = matches.length ? `${matches.length} results (up to 15 shown)` : 'No matching pages. Try a project or workflow name.';
      for (const record of matches) {
        const item = document.createElement('li');
        const link = document.createElement('a');
        link.href = record.url;
        link.textContent = record.title;
        item.append(link);
        list.append(item);
      }
    } catch {
      status.textContent = 'Search unavailable. Use the project directory or navigation.';
    }
  });
  search.addEventListener('keydown', event => {
    if (event.key === 'ArrowDown') { event.preventDefault(); list.querySelector('a')?.focus(); }
  });
  document.addEventListener('click', event => {
    if (!event.target.closest('.search-container')) results.hidden = true;
  });
  // `/` jumps to the search from anywhere but a text field; Escape closes the results.
  document.addEventListener('keydown', event => {
    const active = document.activeElement;
    const typing = active?.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(active?.tagName);
    if (event.key === '/' && !typing && !event.ctrlKey && !event.metaKey && !event.altKey) {
      event.preventDefault();
      search.focus();
    } else if (event.key === 'Escape' && (active === search || results.contains(active))) {
      results.hidden = true;
      search.blur();
    }
  });
})();

(() => {
  const copyCode = async code => {
    try {
      await navigator.clipboard.writeText(code.textContent);
      return true;
    } catch {
      try {
        const selection = window.getSelection();
        if (!selection) return false;
        const range = document.createRange();
        range.selectNodeContents(code);
        selection.removeAllRanges();
        selection.addRange(range);
        const copied = document.execCommand('copy');
        selection.removeAllRanges();
        return copied;
      } catch {
        return false;
      }
    }
  };

  for (const block of document.querySelectorAll('.docs-article pre')) {
    const code = block.querySelector('code');
    if (!code || block.parentElement.classList.contains('docs-code-block')) continue;
    const frame = document.createElement('div');
    frame.className = 'docs-code-block';
    block.before(frame);
    frame.append(block);
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'docs-code-copy';
    button.textContent = 'Copy';
    button.setAttribute('aria-label', 'Copy code to clipboard');
    frame.append(button);
    button.addEventListener('click', async () => {
      const copied = await copyCode(code);
      button.textContent = copied ? 'Copied' : 'Copy failed';
      button.setAttribute('aria-label', copied ? 'Code copied to clipboard' : 'Copying code failed');
      window.setTimeout(() => {
        button.textContent = 'Copy';
        button.setAttribute('aria-label', 'Copy code to clipboard');
      }, 1500);
    });
  }
})();
