#!/usr/bin/env python3
"""Render the derived roadmap page (issue #26).

Derived, never hand-edited: every fact on the page is computed from sources
the gates already validate — the REQ registry, the RFC index, the doc-drift
report, the route table, and the route-coverage burn-down list. The page has
no status field of its own, and it is built to be able to look bad: drifted
documents, unrouted files, and requirements not started are first-class
content, because a roadmap that only shows progress is a witness that
cannot fail.

This is a dev/ tool, not a gate: it renders and cannot meaningfully fail the
tree (RFC-18 §3.1). It runs in CI on pushes to staging (the trunk event
issue #22 created) and publishes to GitHub Pages.

Usage: ``uv run python dev/render-roadmap.py [--out <dir>]``
"""

import html
import json
import subprocess
import sys
import tomllib
from pathlib import Path


def _run(args, cwd):
    proc = subprocess.run(args, capture_output=True, text=True, cwd=cwd)
    return proc.returncode, proc.stdout, proc.stderr


def repo_root():
    rc, out, err = _run(["git", "rev-parse", "--show-toplevel"], None)
    if rc != 0:
        sys.exit(f"render-roadmap: not in a git repository: {err.strip()}")
    return Path(out.strip())


def head_stamp(root):
    rc, out, _ = _run(
        ["git", "log", "-1", "--format=%h %cI", "HEAD"], root
    )
    if rc != 0:
        return "unknown", "unknown"
    sha, _, when = out.strip().partition(" ")
    return sha, when


def repository_web_url(root):
    rc, out, _ = _run(["git", "remote", "get-url", "origin"], root)
    if rc != 0:
        return None
    url = out.strip()
    if url.startswith("git@github.com:"):
        url = "https://github.com/" + url.removeprefix("git@github.com:")
    elif url.startswith("ssh://git@github.com/"):
        url = "https://github.com/" + url.removeprefix("ssh://git@github.com/")
    if url.startswith("https://github.com/"):
        return url.removesuffix(".git").rstrip("/")
    return None


def load_requirements(root):
    with open(root / ".design/reqs/registry.toml", "rb") as stream:
        data = tomllib.load(stream)
    return data.get("requirement", []), len(data.get("view", []))


def load_rfc_index(root):
    rc, out, err = _run(
        [sys.executable, str(root / "gates/rfc-check.py"), "--index",
         "--json"],
        root,
    )
    if rc != 0:
        return None, f"rfc-check --index exited {rc}: {err.strip()[:200]}"
    try:
        value = json.loads(out)
        if not isinstance(value, list):
            return None, "rfc-check --index returned a non-list JSON value"
        return value, None
    except json.JSONDecodeError as exc:
        return None, f"rfc-check --index output unparseable: {exc}"


def load_doc_drift(root):
    """(current, drifted, problem) — problem is an honest inconclusive note."""
    rc, out, err = _run(
        [sys.executable, str(root / "gates/doc-drift.py")], root
    )
    if rc not in (0, 1):
        detail = (err.strip() or out.strip() or "no diagnostic")[:200]
        return [], [], f"doc-drift INCONCLUSIVE (exit {rc}): {detail}"
    current, drifted = [], []
    for line in out.splitlines():
        if line.startswith("CURRENT"):
            current.append(line.split()[1])
        elif line.startswith(("DRIFT", "MISSING-PIN", "INVALID-PIN")):
            parts = line.split()
            drifted.append((parts[0], parts[1]))
    return current, drifted, None


def load_routes(root):
    with open(root / "gates/routes.toml", "rb") as stream:
        data = tomllib.load(stream)
    routes = data.get("route", [])
    docs = {r["design"] for r in routes}
    return len(routes), len(docs)


def load_burndown(root):
    p = root / "gates/route-coverage-burndown.txt"
    if not p.is_file():
        return None
    return [
        line.split("#", 1)[0].strip()
        for line in p.read_text(encoding="utf-8").splitlines()
        if line.split("#", 1)[0].strip()
    ]


def e(s):
    return html.escape(str(s), quote=True)


def load_rfc_guides(root, rfcs):
    """Load the editorial guide and validate it against the canonical index."""
    path = root / "dev/roadmap-rfcs.json"
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        return None, f"RFC guide unavailable: {exc}"
    guides = data.get("guides")
    first, last = data.get("first_rfc"), data.get("last_rfc")
    if not isinstance(guides, list) or not isinstance(first, int) or not isinstance(last, int):
        return None, "RFC guide must declare integer bounds and a guides list"
    expected = set(range(first, last + 1))
    indexed = {int(r["rfc"]): r for r in rfcs}
    seen = set()
    for guide in guides:
        number = guide.get("rfc")
        if not isinstance(number, int) or number in seen:
            return None, f"RFC guide has an invalid or duplicate RFC number: {number!r}"
        seen.add(number)
        if number not in indexed:
            return None, f"RFC-{number} is guided but absent from the canonical RFC index"
        for field in ("summary", "example_label", "example"):
            if not isinstance(guide.get(field), str) or not guide[field].strip():
                return None, f"RFC-{number} guide is missing {field}"
        features = guide.get("features")
        if not isinstance(features, list) or not features or not all(
            isinstance(feature, str) and feature.strip() for feature in features
        ):
            return None, f"RFC-{number} guide needs a non-empty feature list"
        guide["index"] = indexed[number]
    if seen != expected:
        missing = sorted(expected - seen)
        extra = sorted(seen - expected)
        return None, f"RFC guide coverage mismatch (missing={missing}, extra={extra})"
    return sorted(guides, key=lambda guide: guide["rfc"]), None


def requirements_for_rfc(requirements, rfc_file):
    return [r for r in requirements if rfc_file in r.get("contributors", [])]


def meter(label, done, total):
    pct = 0 if total == 0 else max(0, min(100, round(100 * done / total)))
    return f"""
      <div class="meter-row">
        <div class="meter-label">{e(label)}</div>
        <div class="meter-track" role="img"
             aria-label="{e(label)}: {done} of {total} shipped ({pct}%)"
             title="{e(label)}: {done}/{total} shipped">
          <div class="meter-fill" style="width:{pct}%"></div>
        </div>
        <div class="meter-value">{done}/{total} <span class="pct">({pct}%)</span></div>
      </div>"""


def badge(kind, text):
    icon = {"good": "✓", "critical": "✗", "warning": "△"}[kind]
    return (f'<span class="badge badge-{kind}">'
            f'<span aria-hidden="true">{icon}</span> {e(text)}</span>')


def render_rfc_guides(guides, requirements, source_base):
    cards = []
    for guide in guides:
        index = guide["index"]
        rfc_file = f".design/rfcs/{index['file']}"
        linked = requirements_for_rfc(requirements, rfc_file)
        shipped = sum(1 for req in linked if req.get("status") in {"shipped", "retired"})
        progress = (
            f"{shipped}/{len(linked)} registered requirements shipped"
            if linked else "No registry requirements linked"
        )
        features = "".join(f"<li>{e(feature)}</li>" for feature in guide["features"])
        source_href = f"{source_base}/{e(rfc_file)}" if source_base else "#rfc-index"
        cards.append(f"""
      <article class="rfc-card" id="rfc-{guide['rfc']}">
        <div class="rfc-card-head">
          <div><a class="rfc-number" href="{source_href}">RFC-{guide['rfc']}</a>
            <h3>{e(index['title'])}</h3></div>
          <div class="rfc-state">{badge('good' if shipped == len(linked) and linked else 'warning', index['status'])}
            <span>{e(progress)}</span></div>
        </div>
        <p>{e(guide['summary'])}</p>
        <div class="rfc-columns">
          <div><h4>Enables</h4><ul>{features}</ul></div>
          <div><h4>{e(guide['example_label'])}</h4><pre><code>{e(guide['example'])}</code></pre></div>
        </div>
      </article>""")
    return "".join(cards)


def build(root, out_dir):
    sha, when = head_stamp(root)
    repository = repository_web_url(root)
    source_base = f"{repository}/blob/{sha}" if repository and sha != "unknown" else None
    reqs, view_count = load_requirements(root)
    rfcs, rfc_problem = load_rfc_index(root)
    current, drifted, drift_problem = load_doc_drift(root)
    route_count, routed_docs = load_routes(root)
    burndown = load_burndown(root)

    done_status = {"shipped", "retired"}
    live = [r for r in reqs if r.get("status") != "retired"]
    shipped = sum(1 for r in live if r.get("status") in done_status)
    partial = sum(1 for r in live if r.get("status") == "partial")
    scopes = {}
    for r in live:
        s = scopes.setdefault(r.get("scope", "unscoped"), [0, 0])
        s[1] += 1
        if r.get("status") in done_status:
            s[0] += 1
    scope_rows = "".join(
        meter(scope, d, t)
        for scope, (d, t) in sorted(
            scopes.items(), key=lambda kv: -kv[1][1]
        )
    )

    if rfc_problem:
        rfc_section = f"<p>{badge('warning', 'not derived')} {e(rfc_problem)}</p>"
        rfc_count = "?"
    else:
        rfc_count = len(rfcs)
        rows = "".join(
            f"<tr><td><a href=\"{source_base + '/.design/rfcs/' + e(r['file']) if source_base else '#rfc-' + e(r['rfc'])}\">RFC-{e(r['rfc'])}</a></td><td>{e(r['title'])}</td>"
            f"<td>{e(r['status'])}</td><td>r{e(r['revision'])}</td></tr>"
            for r in sorted(rfcs, key=lambda r: int(r["rfc"]))
        )
        rfc_section = (
            "<div class=\"scroll\"><table><thead><tr><th>RFC</th><th>Title</th>"
            "<th>Status</th><th>Rev</th></tr></thead>"
            f"<tbody>{rows}</tbody></table></div>"
        )

    guides, guide_problem = (None, rfc_problem) if rfc_problem else load_rfc_guides(root, rfcs)
    if guide_problem:
        guide_section = f"<p>{badge('warning', 'guide unavailable')} {e(guide_problem)}</p>"
    else:
        guide_section = render_rfc_guides(guides, live, source_base)

    if drift_problem:
        drift_section = (
            f"<p>{badge('warning', 'inconclusive')} {e(drift_problem)}</p>"
        )
        drift_headline = badge("warning", "inconclusive")
    elif drifted:
        items = "".join(
            f"<li>{badge('critical', kind)} <code>{e(doc)}</code></li>"
            for kind, doc in drifted
        )
        drift_section = (
            f"<p>{len(drifted)} of {len(drifted) + len(current)} routed "
            f"documents are not current:</p><ul class=\"plain\">{items}</ul>"
        )
        drift_headline = badge("critical", f"{len(drifted)} drifted")
    else:
        drift_section = (
            f"<p>{badge('good', 'all current')} every routed document "
            f"({len(current)}) matches its pin.</p>"
        )
        drift_headline = badge("good", "all current")

    if burndown is None:
        burn_section = (
            f"<p>{badge('warning', 'not derived')} "
            "gates/route-coverage-burndown.txt is not in this tree yet "
            "(it lands with the route-coverage gate).</p>"
        )
    elif burndown:
        items = "".join(f"<li><code>{e(f)}</code></li>" for f in burndown)
        burn_section = (
            f"<p>{badge('critical', str(len(burndown)) + ' unrouted')} "
            "gated source files with no route — each is un-editable until "
            f"routed, and this list can only shrink:</p>"
            f"<ul class=\"plain cols\">{items}</ul>"
        )
    else:
        burn_section = (
            f"<p>{badge('good', 'empty')} every gated file is routed.</p>"
        )

    not_derived = """
      <ul>
        <li>Residual-trust statements — written in the stage documents and
            RFC §6 sections; extracting them is future work, and until then
            they are absent here, not absent from the project.</li>
        <li>Blocked-on-others items — recorded in the session handoffs
            (machine-local by design), not derivable from the tree.</li>
        <li>Upstream filing state — lives on the upstream tracker.</li>
      </ul>"""

    total = len(live)
    pct = 0 if total == 0 else round(100 * shipped / total)

    page = f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Thermite 3 — derived roadmap</title>
<style>
  :root {{
    color-scheme: light;
    --page: #f9f9f7; --surface: #fcfcfb;
    --ink: #0b0b0b; --ink-2: #52514e; --muted: #898781;
    --grid: #e1e0d9; --border: rgba(11,11,11,0.10);
    --fill: #2a78d6;
    --good: #0ca30c; --warning: #fab219; --critical: #d03b3b;
    --good-text: #006300;
  }}
  @media (prefers-color-scheme: dark) {{
    :root:not([data-theme="light"]) {{
      color-scheme: dark;
      --page: #0d0d0d; --surface: #1a1a19;
      --ink: #ffffff; --ink-2: #c3c2b7; --muted: #898781;
      --grid: #2c2c2a; --border: rgba(255,255,255,0.10);
      --fill: #3987e5;
      --good-text: #0ca30c;
    }}
  }}
  :root[data-theme="dark"] {{
    color-scheme: dark;
    --page: #0d0d0d; --surface: #1a1a19;
    --ink: #ffffff; --ink-2: #c3c2b7; --muted: #898781;
    --grid: #2c2c2a; --border: rgba(255,255,255,0.10);
    --fill: #3987e5;
    --good-text: #0ca30c;
  }}
  * {{ box-sizing: border-box; }}
  body {{
    margin: 0; background: var(--page); color: var(--ink);
    font: 15px/1.55 system-ui, -apple-system, "Segoe UI", sans-serif;
  }}
  main {{ max-width: 880px; margin: 0 auto; padding: 32px 20px 64px; }}
  h1 {{ font-size: 1.5rem; margin: 0 0 4px; }}
  h2 {{ font-size: 1.05rem; margin: 36px 0 12px; }}
  h3 {{ font-size: 1rem; margin: 3px 0 0; }}
  h4 {{ font-size: 0.85rem; letter-spacing: 0.04em; text-transform: uppercase;
        color: var(--ink-2); margin: 0 0 8px; }}
  .sub {{ color: var(--ink-2); margin: 0 0 24px; }}
  .tiles {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 12px; }}
  .tile {{ background: var(--surface); border: 1px solid var(--border);
           border-radius: 8px; padding: 14px 16px; }}
  .tile .n {{ font-size: 1.7rem; font-weight: 650; }}
  .tile .k {{ color: var(--ink-2); font-size: 0.85rem; }}
  .meter-row {{ display: grid; grid-template-columns: 7.5em 1fr 8.5em;
                gap: 10px; align-items: center; margin: 6px 0; }}
  .meter-label {{ color: var(--ink-2); text-align: right; }}
  .meter-track {{ background: var(--grid); border-radius: 4px; height: 10px; }}
  .meter-fill {{ background: var(--fill); border-radius: 4px; height: 10px; }}
  .meter-value {{ font-variant-numeric: tabular-nums; }}
  .pct {{ color: var(--muted); }}
  table {{ border-collapse: collapse; width: 100%; background: var(--surface);
           font-variant-numeric: tabular-nums; }}
  th, td {{ text-align: left; padding: 6px 10px;
            border-bottom: 1px solid var(--grid); }}
  th {{ color: var(--ink-2); font-weight: 600; }}
  .scroll {{ overflow-x: auto; border: 1px solid var(--border);
             border-radius: 8px; }}
  a {{ color: var(--fill); text-underline-offset: 0.15em; }}
  .rfc-guide {{ display: grid; gap: 14px; }}
  .rfc-card {{ background: var(--surface); border: 1px solid var(--border);
               border-radius: 10px; padding: 18px; scroll-margin-top: 16px; }}
  .rfc-card-head {{ display: flex; justify-content: space-between; gap: 18px;
                    align-items: flex-start; }}
  .rfc-number {{ font-weight: 700; font-variant-numeric: tabular-nums; }}
  .rfc-state {{ display: flex; flex-direction: column; align-items: flex-end;
                gap: 5px; color: var(--muted); font-size: 0.8rem; text-align: right; }}
  .rfc-columns {{ display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
                  gap: 22px; margin-top: 16px; }}
  .rfc-columns ul {{ margin: 0; padding-left: 20px; }}
  pre {{ margin: 0; padding: 12px; overflow-x: auto; border-radius: 7px;
         background: var(--page); border: 1px solid var(--grid); line-height: 1.4; }}
  .badge {{ display: inline-block; border-radius: 999px; padding: 1px 10px;
            font-size: 0.85rem; border: 1px solid var(--border); }}
  .badge-good {{ color: var(--good-text); }}
  .badge-warning {{ color: var(--ink); }}
  .badge-critical {{ color: var(--critical); }}
  ul.plain {{ list-style: none; padding-left: 0; }}
  ul.plain li {{ margin: 3px 0; }}
  ul.cols {{ columns: 2; }}
  code {{ font-size: 0.9em; }}
  footer {{ margin-top: 48px; color: var(--muted); font-size: 0.85rem;
            border-top: 1px solid var(--grid); padding-top: 12px; }}
  @media (max-width: 640px) {{ ul.cols {{ columns: 1; }}
    .meter-row {{ grid-template-columns: 5.5em 1fr 7em; }}
    .rfc-card-head {{ display: block; }}
    .rfc-state {{ align-items: flex-start; text-align: left; margin-top: 8px; }}
    .rfc-columns {{ grid-template-columns: 1fr; }} }}
</style>
</head>
<body>
<main>
  <h1>Thermite 3 — derived roadmap</h1>
  <p class="sub">Every fact on this page is computed from the tree; nothing
  here is hand-maintained. Drift and debt are shown on purpose — a roadmap
  that can only show progress is a witness that cannot fail.</p>

  <div class="tiles">
    <div class="tile"><div class="n">{pct}%</div>
      <div class="k">{shipped} of {total} requirements shipped
      ({partial} partial)</div></div>
    <div class="tile"><div class="n">{rfc_count}</div>
      <div class="k">RFCs on record</div></div>
    <div class="tile"><div class="n">{drift_headline}</div>
      <div class="k">design-doc freshness (doc-drift)</div></div>
    <div class="tile"><div class="n">{route_count}</div>
      <div class="k">routes governing {routed_docs} design docs</div></div>
  </div>

  <h2>Requirements by scope</h2>
  {scope_rows}
  <p class="sub">Counts from <code>.design/reqs/registry.toml</code>
  ({view_count} generated views; retired requirements excluded).</p>

  <h2 id="rfc-index">RFCs</h2>
  {rfc_section}

  <h2>Thermite 3 capability sequence</h2>
  <p class="sub">RFC status and requirement progress are derived from the canonical
  index and registry. Feature summaries and examples are checked editorial guides
  linked to their source RFCs.</p>
  <div class="rfc-guide">{guide_section}</div>

  <h2>Verification and routing health</h2>
  {drift_section}
  {burn_section}

  <h2>Not derived yet — named, not omitted</h2>
  {not_derived}

  <footer>Derived from <code>staging @ {e(sha)}</code> (committed {e(when)})
  by <code>dev/render-roadmap.py</code>. If this page disagrees with the
  tree, the tree wins and the renderer has a bug.</footer>
</main>
</body>
</html>
"""
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "index.html").write_text(page, encoding="utf-8")
    (out_dir / ".nojekyll").write_text("", encoding="utf-8")
    print(f"render-roadmap: wrote {out_dir / 'index.html'} "
          f"({shipped}/{total} shipped, {len(drifted)} drifted, "
          f"{'no burn-down list' if burndown is None else str(len(burndown)) + ' unrouted'})")


def main(argv=None):
    args = list(sys.argv[1:] if argv is None else argv)
    out = None
    while args:
        a = args.pop(0)
        if a == "--out":
            if not args:
                sys.exit("render-roadmap: --out requires a value")
            out = Path(args.pop(0))
        elif a in ("-h", "--help"):
            print(__doc__)
            return
        else:
            sys.exit(f"render-roadmap: unknown argument {a!r}")
    root = repo_root()
    build(root, out if out is not None else root / ".build/roadmap")


if __name__ == "__main__":
    main()
