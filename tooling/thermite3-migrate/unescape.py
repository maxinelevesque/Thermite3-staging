"""Rust string-literal unescaping that keeps an offset map.

The rewriter needs the literal's *value* to parse it, and the literal's *source
offsets* to splice into. So unescaping records, for each character of the value,
the offset in the source it came from — which lets an edit computed on the value
be applied to the source with every other escape left exactly as written.
"""

SIMPLE = {"n": "\n", "t": "\t", "r": "\r", "0": "\0", "\\": "\\", '"': '"', "'": "'"}


def unescape(src: str, raw: bool):
    """Return (value, map) where map[i] is the offset in `src` of value[i].
    map has one extra entry at the end: the offset just past the last character."""
    if raw:
        return src, list(range(len(src) + 1))
    out, omap = [], []
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if c != "\\":
            omap.append(i)
            out.append(c)
            i += 1
            continue
        if i + 1 >= n:
            omap.append(i); out.append(c); i += 1; continue
        e = src[i + 1]
        if e == "\n":
            # a line continuation: the backslash, the newline and the leading
            # whitespace of the next line all vanish
            i += 2
            while i < n and src[i] in " \t":
                i += 1
            continue
        if e in SIMPLE:
            omap.append(i); out.append(SIMPLE[e]); i += 2; continue
        if e == "x" and i + 3 < n:
            try:
                omap.append(i); out.append(chr(int(src[i + 2:i + 4], 16))); i += 4; continue
            except ValueError:
                pass
        if e == "u" and i + 2 < n and src[i + 2] == "{":
            j = src.find("}", i)
            if j > 0:
                try:
                    omap.append(i); out.append(chr(int(src[i + 3:j], 16))); i = j + 1; continue
                except ValueError:
                    pass
        omap.append(i); out.append(e); i += 2
    omap.append(n)
    return "".join(out), omap


def splice_back(src: str, omap, edits):
    """Apply edits — (value_start, value_len, text) — to the escaped source.

    Inserted text is plain ASCII words and punctuation, so it needs no escaping;
    everything outside an edit keeps its original spelling byte for byte.
    """
    out, pos = [], 0
    for vs, vlen, text in sorted(edits, key=lambda e: e[0]):
        s = omap[vs]
        e = omap[vs + vlen]
        if s < pos:
            raise ValueError(f"overlapping edit at {s} (already at {pos})")
        out.append(src[pos:s])
        out.append(text)
        pos = e
    out.append(src[pos:])
    return "".join(out)
