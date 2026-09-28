// Optional local search over Zola's static index. All navigation works without it.
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
})();
