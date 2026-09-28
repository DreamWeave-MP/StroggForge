"""Validate built local links, assets and fragments, including project Pages prefixes."""
from html.parser import HTMLParser
from pathlib import Path
import tomllib
from urllib.parse import unquote, urljoin, urlsplit

ROOT = Path(__file__).resolve().parents[2]
PUBLIC = ROOT / "site/public"
CONFIG = tomllib.loads((ROOT / "site/config.toml").read_text())
BASE = CONFIG["base_url"].rstrip("/") + "/"
BASE_URL = urlsplit(BASE)


class Document(HTMLParser):
    def __init__(self, text):
        super().__init__(convert_charrefs=True)
        self.ids = set()
        self.links = []
        self.feed(text)

    def handle_starttag(self, tag, attributes):
        attributes = dict(attributes)
        if "id" in attributes:
            self.ids.add(attributes["id"])
        if tag in ("a", "link") and "href" in attributes:
            self.links.append(attributes["href"])
        if tag in ("img", "script", "source") and "src" in attributes:
            self.links.append(attributes["src"])


documents = {path: Document(path.read_text()) for path in sorted(PUBLIC.rglob("*.html"))}
if not documents:
    raise SystemExit("No built HTML; run zola --root site build first")
errors = []
checked = 0
for path, document in documents.items():
    relative = str(path.relative_to(PUBLIC))
    current = urljoin(BASE, relative.removesuffix("index.html"))
    for link in document.links:
        target = urlsplit(urljoin(current, link))
        if target.scheme not in ("http", "https") or target.netloc != BASE_URL.netloc:
            continue
        if not target.path.startswith(BASE_URL.path):
            # Absolute sibling project sites are deliberate external documentation links.
            if not urlsplit(link).netloc:
                errors.append(f"{relative}: relative link escapes site base: {link}")
            continue
        local = PUBLIC / unquote(target.path[len(BASE_URL.path):])
        if local.is_dir():
            local /= "index.html"
        checked += 1
        if not local.is_file():
            errors.append(f"{relative}: missing target {link}")
        elif target.fragment and local.suffix == ".html":
            if unquote(target.fragment) not in documents[local].ids:
                errors.append(f"{relative}: missing anchor {link}")
if errors:
    raise SystemExit("\n".join(errors))
print(f"Checked {checked} local links/assets/fragments across {len(documents)} HTML pages")
