"""Page builder for the step-by-step test book (docs/demo/how-to-test-the-app.pdf).

Draws simple bilingual screens (Urdu as the app shows it, English under it) with red numbers on the
exact buttons and fields. `python3 build.py` writes guide.html and the PDF. Written by tech-lead.
"""
import html
import re

# ---------------------------------------------------------------- text markup

def txt(s):
    """[[ur|en]] -> Urdu word with English in brackets; {{x}} -> value to type; **x** -> bold."""
    s = html.escape(s, quote=False)
    s = re.sub(r"\[\[(.+?)\|(.+?)\]\]", r'<span class="u">\1</span> <span class="en">(\2)</span>', s)
    s = re.sub(r"\[\[(.+?)\]\]", r'<span class="u">\1</span>', s)
    s = re.sub(r"\{\{(.+?)\}\}", r'<span class="typed">\1</span>', s)
    s = re.sub(r"\*\*(.+?)\*\*", r"<b>\1</b>", s)
    return s


def bd(n):
    return f'<i class="bd">{n}</i>' if n else ""


# ---------------------------------------------------------------- screen parts

def lab(ur, en):
    return f'<div class="lab"><span class="u">{ur}</span><span class="e">{en}</span></div>'


def fld(ur, en, val="", n=None, grow=1, kind="in", ph="", w=None):
    style = f"flex:{grow}" + (f";width:{w}px;flex:none" if w else "")
    chev = '<span class="chev">&#9662;</span>' if kind == "sel" else ""
    chk = ""
    if kind == "chk":
        return (f'<div class="fld chk" style="{style}"><div class="cb{" on" if val else ""}">{"&#10003;" if val else ""}{bd(n)}</div>'
                f'<div class="lab"><span class="u">{ur}</span><span class="e">{en}</span></div></div>')
    shown = f'<span class="val">{html.escape(val)}</span>' if val else f'<span class="ph">{html.escape(ph)}</span>'
    return (f'<div class="fld" style="{style}">{lab(ur, en)}<div class="box">{shown}{chev}{bd(n)}</div></div>')


def btn(ur, en, key="", pri=False, n=None, danger=False, small=False):
    cls = "btn" + (" pri" if pri else "") + (" dng" if danger else "") + (" sm" if small else "")
    k = f'<span class="key">{key}</span>' if key else ""
    return f'<div class="{cls}">{k}<span class="u">{ur}</span><span class="e">{en}</span>{bd(n)}</div>'


def row(*items, gap=10):
    return f'<div class="row" style="gap:{gap}px">{"".join(items)}</div>'


def card(ur, en, *kids, n=None):
    head = f'<div class="ch">{lab(ur, en)}</div>' if ur else ""
    return f'<div class="card">{head}{"".join(kids)}{bd(n)}</div>'


def note(kind, text, n=None):
    return f'<div class="note {kind}">{txt(text)}{bd(n)}</div>'


def tabs(items, active=0, n=None):
    out = []
    for i, (ur, en) in enumerate(items):
        out.append(f'<div class="tab{" on" if i == active else ""}"><span class="u">{ur}</span><span class="e">{en}</span></div>')
    return f'<div class="tabs">{"".join(out)}{bd(n)}</div>'


def tbl(heads, rows, n=None, hl=None, total=None, widths=None):
    """heads: [(ur, en)], rows: [[cell,...]]; hl: set of (r, c) cells to colour green."""
    hl = hl or set()
    th = "".join(f'<th>{lab(u, e)}</th>' for u, e in heads)
    body = ""
    for r, cells in enumerate(rows):
        tds = ""
        for c, v in enumerate(cells):
            cls = ' class="hl"' if (r, c) in hl else ""
            tds += f"<td{cls}>{v}</td>"
        body += f"<tr>{tds}</tr>"
    if total:
        body += "<tr class=\"tot\">" + "".join(f"<td>{v}</td>" for v in total) + "</tr>"
    return f'<div class="tw"><table>{"<tr>" + th + "</tr>"}{body}</table>{bd(n)}</div>'


def dialog(ur, en, text, buttons):
    b = "".join(buttons)
    return f'<div class="dlg"><div class="dh">{lab(ur, en)}</div><div class="dt">{txt(text)}</div><div class="db">{b}</div></div>'


def tree(items, n=None):
    """items: [(indent, code, ur, en, amount)]"""
    out = ""
    for ind, code, ur, en, amt in items:
        out += (f'<div class="tr" style="padding-inline-start:{ind * 22}px"><span class="code">{code}</span>'
                f'<span class="u">{ur}</span><span class="e">{en}</span><span class="amt">{amt}</span></div>')
    return f'<div class="treebox">{out}{bd(n)}</div>'


def totals(items, n=None):
    """Summary strip: items [(ur, en, value)]"""
    cells = "".join(f'<div class="sc">{lab(u, e)}<div class="sv">{v}</div></div>' for u, e, v in items)
    return f'<div class="sum">{cells}{bd(n)}</div>'


def stat(ur, en, value, n=None, tone=""):
    return f'<div class="stat {tone}">{lab(ur, en)}<div class="sv">{value}</div>{bd(n)}</div>'


# ---------------------------------------------------------------- the app frame

MENU = [
    ("dashboard", "ڈیش بورڈ", "Dashboard"), ("index", "انڈکس", "Index"), ("bikri", "بکری", "Sales"),
    ("rokar", "روکڑ", "Cash"), ("gardens", "باغات", "Gardens"), ("trips", "مارکیٹ سفر", "Trips"),
    ("partners", "شراکت دار", "Partners"), ("analytics", "تجزیات", "Analytics"), ("reports", "رپورٹ", "Reports"),
    ("backup", "بیک اپ", "Backup"), ("settings", "ترتیبات", "Settings"),
]
SUBS = {
    "bikri": [("book", "بکری بک", "Sale Book"), ("receipt", "وصولی / اگراہی", "Receipt"), ("payment", "ادائیگی", "Payment"),
              ("advance", "پیشگی کھاتہ", "Advance"), ("pukhta", "پختہ کھاتہ", "Pukhta Khata"), ("today", "آج کی بکری", "Today's sales"),
              ("stock", "باقی مال", "Unsold stock")],
    "rokar": [("book", "روکڑ", "Cash book"), ("expense", "خرچہ", "Expense"), ("lent", "ادھار رقم", "Money lent"),
              ("khata", "الگ کھاتہ", "Separate khata"), ("general", "جنرل", "Journal"), ("voucher", "وصولی / ادائیگی / پیشگی", "Vouchers")],
}


def frame(main, sub=None, title=("", ""), actions=(), body="", nmain=None, nsub=None, dirn="rtl", minh=0):
    side = "".join(
        f'<div class="mi{" on" if k == main else ""}"><span class="u">{u}</span><span class="e">{e}</span>{bd(nmain) if k == main else ""}</div>'
        for k, u, e in MENU)
    subhtml = ""
    if main in SUBS:
        subhtml = '<div class="sub">' + "".join(
            f'<div class="si{" on" if k == sub else ""}"><span class="u">{u}</span><span class="e">{e}</span>{bd(nsub) if k == sub else ""}</div>'
            for k, u, e in SUBS[main]) + "</div>"
    top = ('<div class="top"><div class="srch">Ctrl+K &nbsp; <span class="u">تلاش</span></div>'
           '<div class="lang"><span>English</span><span class="on">اردو</span></div><div class="bell">&#128276;</div></div>')
    ttl = ""
    if title[0] or actions:
        ttl = (f'<div class="tt"><div class="ttl"><span class="u big">{title[0]}</span><span class="e">{title[1]}</span></div>'
               f'<div class="acts">{"".join(actions)}</div></div>')
    status = '<div class="stat-bar"><span>Owner</span><span>10-10-2026</span><span class="ok">&#10003; <span class="u">دن کھلا</span></span></div>'
    mh = f"min-height:{minh}px" if minh else ""
    return (f'<div class="mock {dirn}"><div class="appin"><div class="menu"><div class="logo u">منڈی سافٹ ویئر</div>{side}</div>'
            f'{subhtml}<div class="main"><div class="cont" style="{mh}">{ttl}{body}</div></div></div></div>')


def plain(body, dirn="rtl", minh=0):
    """A screen without the menus (sign-in, dialogs)."""
    mh = f"min-height:{minh}px" if minh else ""
    return f'<div class="mock {dirn}"><div class="cont solo" style="{mh}">{body}</div></div>'


# ---------------------------------------------------------------- one page

PART_COLOURS = {
    "Start": "#475569", "Index": "#1d4ed8", "Sales": "#0f766e", "Cash": "#b45309", "Gardens": "#4d7c0f",
    "Trips": "#a16207", "Partners": "#7c3aed", "Analytics": "#0e7490", "Reports": "#be185d", "Backup": "#0369a1",
    "Settings": "#52525b", "Dashboard": "#15803d", "End": "#475569",
}


def page(part, no, title_en, title_ur, path, screen="", steps=(), typed=(), see=(), wrong=(), tip="", differs=(), extra=""):
    col = PART_COLOURS.get(part, "#334155")
    chips = ' <span class="arr">&#10132;</span> '.join(f'<span class="chip"><span class="u">{u}</span> {e}</span>' for u, e in path)
    openbar = f'<div class="open"><b>Open it:</b> {chips}</div>' if path else ""
    st = ""
    for s in steps:
        n, t = s.split("|", 1)
        mark = f'<span class="sn">{n}</span>' if n.strip() else '<span class="sn dot"></span>'
        st += f'<div class="step">{mark}<div class="stx">{txt(t)}</div></div>'
    st = f'<div class="steps">{st}</div>' if st else ""
    ty = ""
    if typed:
        rows = "".join(f'<tr><td>{txt(a)}</td><td>{txt(b)}</td></tr>' for a, b in typed)
        ty = f'<div class="box typebox"><div class="bt">Type this</div><table class="tt2">{rows}</table></div>'
    se = ""
    if see:
        li = "".join(f"<li>{txt(x)}</li>" for x in see)
        se = f'<div class="box seebox"><div class="bt">You must see</div><ul>{li}</ul></div>'
    wr = ""
    if wrong:
        li = "".join(f"<li>{txt(x)}</li>" for x in wrong)
        wr = f'<div class="box wrongbox"><div class="bt">Now try a wrong entry (do not save)</div><ul>{li}</ul></div>'
    df = ""
    if differs:
        li = "".join(f"<li>{txt(x)}</li>" for x in differs)
        df = f'<div class="box diffbox"><div class="bt">Not the same as</div><ul>{li}</ul></div>'
    tp = f'<div class="tip"><b>Tip:</b> {txt(tip)}</div>' if tip else ""
    boxes = ty + se + wr + df
    return (f'<section class="page"><div class="head" style="background:{col}"><span>{part}</span><span class="pn">{no}</span></div>'
            f'<h1><span>{txt(title_en)}</span><span class="u h1u">{title_ur}</span></h1>{openbar}'
            f'<div class="scr">{screen}</div>{st}{extra}<div class="boxes">{boxes}</div>{tp}</section>')


CSS = r"""
@font-face{font-family:'Nastaliq';src:url('fonts/NotoNastaliqUrdu-Variable.woff2');font-weight:400 700}
@font-face{font-family:'InterV';src:url('fonts/Inter-Variable.woff2');font-weight:100 900}
@page{size:A4;margin:0}
*{box-sizing:border-box}
body{margin:0;font-family:'InterV',Helvetica,Arial,sans-serif;color:#1f2933;background:#fff;-webkit-print-color-adjust:exact;print-color-adjust:exact}
.u{font-family:'Nastaliq','Noto Nastaliq Urdu',serif;line-height:1.75}
.page{width:210mm;height:296.5mm;padding:0 11mm 5mm;position:relative;page-break-after:always;overflow:hidden}
.head{margin:0 -11mm 4mm;height:8mm;color:#fff;font-size:10.5pt;font-weight:600;display:flex;justify-content:space-between;align-items:center;padding:0 11mm;letter-spacing:.4px;text-transform:uppercase}
h1{font-size:17pt;margin:0 0 1.5mm;display:flex;justify-content:space-between;align-items:baseline;line-height:1.25;gap:8px}
.h1u{font-size:18pt;font-weight:600;color:#14243f}
.open{background:#eef3fb;border:1px solid #cfdcf0;border-radius:6px;padding:1mm 3mm;font-size:10pt;margin-bottom:2.2mm;line-height:1.7}
.chip{display:inline-block;background:#fff;border:1px solid #9db6e0;border-radius:5px;padding:0 7px;font-weight:600;color:#1d4ed8}
.chip .u{font-size:12pt}
.arr{color:#1d4ed8;font-weight:800}
.scr{margin-bottom:2.2mm}
.steps{margin:2mm 0}
.step{display:flex;gap:3mm;align-items:flex-start;margin:0 0 1.1mm;font-size:10.8pt;line-height:1.3}
.sn{flex:none;width:6.4mm;height:6.4mm;border-radius:50%;background:#d92d20;color:#fff;font-weight:800;font-size:11pt;display:flex;align-items:center;justify-content:center;margin-top:.3mm}
.sn.dot{background:#94a3b8;width:3.2mm;height:3.2mm;margin:2mm 2mm 0 2mm}
.stx .u{font-size:12.5pt;color:#14243f;font-weight:600}
.stx .en{color:#52606d;font-size:10pt}
.typed{background:#fff7d6;border:1px solid #f0d36b;border-radius:4px;padding:0 5px;font-family:Menlo,Consolas,monospace;font-size:10.5pt;font-weight:700;color:#7a4b00;white-space:nowrap}
.boxes{display:grid;grid-template-columns:1fr 1fr;gap:3mm;margin-top:2mm}
.box{border-radius:6px;padding:1.6mm 2.6mm;font-size:9.6pt;line-height:1.3;border:1px solid}
.box .bt{font-weight:800;font-size:9.5pt;text-transform:uppercase;letter-spacing:.5px;margin-bottom:1mm}
.box ul{margin:0;padding-left:4mm}.box li{margin:0 0 .6mm}
.box .u{font-size:11.5pt;font-weight:600}.box .en{font-size:9.5pt;color:#52606d}
.typebox{background:#fffbea;border-color:#f0d36b;color:#5c4400}
.seebox{background:#ecf9f2;border-color:#9fdcbc;color:#0b5a3b;grid-column:auto}
.wrongbox{background:#fff1e6;border-color:#f6b98a;color:#8a3b00}
.diffbox{background:#f3effb;border-color:#c9b8ee;color:#4a2a8a}
.tt2{border-collapse:collapse;width:100%}.tt2 td{padding:.6mm 1mm;border-bottom:1px dotted #e8d9a0;vertical-align:top}.tt2 td:first-child{color:#7a6320;width:42%}
.tip{margin-top:2mm;font-size:9.6pt;color:#334155;background:#f1f5f9;border-radius:6px;padding:1.6mm 3mm}
/* --- the drawn screen --- */
.mock{zoom:.9;width:750px;border:2px solid #243b5e;border-radius:8px;overflow:hidden;background:#f4f2ec;position:relative;font-size:12px}
.mock.ltr{direction:ltr}.mock.rtl{direction:rtl}
.appin{display:flex;align-items:stretch;min-height:290px}
.menu{width:116px;background:#14243f;color:#fff;padding:8px 6px;flex:none}
.logo{font-size:15px;text-align:center;padding:4px 0 8px;color:#fff;border-bottom:1px solid #2c4268;margin-bottom:6px;line-height:1.9}
.mi{position:relative;padding:3px 8px;border-radius:6px;margin:1px 0;color:#c9d6ea;display:flex;flex-direction:column;line-height:1.1}
.mi .u{font-size:13px;color:inherit;line-height:1.65}.mi .e{font-size:7.5px;opacity:.7;margin-top:-1px}
.mi.on{background:#2a4f8a;color:#fff}
.sub{width:100px;background:#e9e6dd;border-inline-end:1px solid #d6d1c3;padding:8px 5px;flex:none}
.si{position:relative;padding:2px 6px;border-radius:6px;margin:1px 0;display:flex;flex-direction:column;line-height:1.1;color:#334155}
.si .u{font-size:12.5px;line-height:1.55}.si .e{font-size:7.5px;color:#6b7280;margin-top:-2px}
.si.on{background:#1f3d6e;color:#fff}.si.on .e{color:#cbd5e1}
.main{flex:1;min-width:0;display:flex;flex-direction:column}
.top{display:flex;gap:8px;align-items:center;padding:5px 10px;background:#fff;border-bottom:1px solid #e2ddd0}
.srch{flex:1;border:1px solid #cfc9b8;border-radius:6px;padding:3px 8px;font-size:10px;color:#777;background:#f8f6f1}
.lang{display:flex;border:1px solid #1f3d6e;border-radius:6px;overflow:hidden;font-size:10px}.lang span{padding:2px 8px}.lang .on{background:#1f3d6e;color:#fff}
.bell{font-size:12px}
.cont{padding:8px 12px;flex:1}
.cont.solo{background:#f4f2ec;padding:16px}
.tt{display:flex;justify-content:space-between;align-items:center;margin-bottom:6px;gap:8px}
.ttl{display:flex;flex-direction:row;align-items:baseline;gap:8px}.ttl .big{font-size:22px;color:#14243f;line-height:1.5}.ttl .e{font-size:9.5px;color:#6b7280}
.acts{display:flex;gap:6px;flex-wrap:wrap}
.btn{position:relative;display:inline-flex;align-items:center;gap:5px;border:1px solid #9aa7bd;background:#fff;border-radius:7px;padding:2px 10px;line-height:1.1;white-space:nowrap}
.btn .u{font-size:13px;line-height:1.55}.btn .e{font-size:8px;color:#6b7280}.btn .key{font-size:9px;background:#e8edf5;border-radius:4px;padding:0 4px;font-weight:700;color:#334155}
.btn.pri{background:#1f3d6e;color:#fff;border-color:#1f3d6e}.btn.pri .e{color:#cbd5e1}.btn.pri .key{background:#3b5c91;color:#fff}
.btn.dng{border-color:#d92d20;color:#b42318}.btn.sm{padding:0 7px}.btn.sm .u{font-size:11.5px}
.row{display:flex;flex-wrap:wrap;margin-bottom:8px;align-items:flex-end}
.fld{display:flex;flex-direction:column;min-width:90px}
.lab{display:flex;flex-direction:row;flex-wrap:wrap;align-items:baseline;gap:2px 6px;line-height:1.1}.lab .u{font-size:13px;color:#14243f;line-height:1.6;font-weight:600}.lab .e{font-size:8.5px;color:#6b7280}
.box{position:relative}
.fld .box{border:1px solid #9aa7bd;background:#fff;border-radius:6px;padding:4px 8px;min-height:26px;display:flex;justify-content:space-between;align-items:center;font-size:12px}
.fld .val{direction:ltr;unicode-bidi:plaintext}.fld .ph{color:#aaa}.chev{color:#555;font-size:10px}
.fld.chk{flex-direction:row;align-items:center;gap:6px}.cb{position:relative;width:18px;height:18px;border:1.5px solid #556;border-radius:4px;background:#fff;color:#fff;font-size:12px;display:flex;align-items:center;justify-content:center}.cb.on{background:#1f3d6e}
.card{position:relative;background:#fff;border:1px solid #ddd8ca;border-radius:9px;padding:8px 10px;margin-bottom:8px}
.ch{margin-bottom:4px}
.note{position:relative;border-radius:7px;padding:5px 9px;font-size:11px;margin:4px 0;border:1px solid}
.note .u{font-size:12.5px;font-weight:600}.note .en{font-size:9px}
.note.amber{background:#fff4dc;border-color:#f0c97a;color:#7a4b00}.note.green{background:#e7f6ee;border-color:#9fdcbc;color:#0b5a3b}.note.red{background:#fde8e6;border-color:#f1a7a0;color:#9a1b12}.note.blue{background:#e8f0fc;border-color:#aac3ee;color:#1d3f7a}
.tabs{position:relative;display:flex;gap:4px;margin-bottom:8px;flex-wrap:wrap}
.tab{border:1px solid #b6c0d2;border-radius:7px 7px 0 0;padding:1px 9px;background:#fff;display:flex;flex-direction:column;line-height:1}
.tab{flex-direction:row;gap:5px;align-items:baseline}.tab .u{font-size:12.5px;line-height:1.55}.tab .e{font-size:8px;color:#6b7280}.tab.on{background:#1f3d6e;color:#fff}.tab.on .e{color:#cbd5e1}.tab.on .u{color:#fff}
.tw{position:relative;margin-bottom:8px;border:1px solid #cfc9b8;border-radius:7px;overflow:hidden;background:#fff}
table{border-collapse:collapse;width:100%;font-size:11px}
th{background:#ece8dc;text-align:start;padding:2px 7px;border-bottom:1px solid #cfc9b8;vertical-align:bottom}
td{padding:3px 7px;border-bottom:1px solid #eee9dc;direction:ltr;text-align:start;unicode-bidi:plaintext}
td .u{font-size:12.5px}
td.hl{background:#dff3e8;font-weight:700;color:#0b5a3b}
tr.tot td{background:#ece8dc;font-weight:700}
.dlg{position:relative;width:360px;margin:6px auto;background:#fff;border:2px solid #243b5e;border-radius:10px;padding:10px 14px;box-shadow:0 6px 20px rgba(0,0,0,.25)}
.dh{margin-bottom:4px}.dt{font-size:12px;margin:4px 0 8px;line-height:1.6}.dt .u{font-size:13px}.db{display:flex;gap:8px;justify-content:flex-end}
.treebox{position:relative;background:#fff;border:1px solid #ddd8ca;border-radius:8px;padding:6px 8px;margin-bottom:8px}
.tr{display:flex;gap:10px;align-items:center;padding:1px 0;border-bottom:1px dotted #e7e2d4}
.tr .code{font-family:Menlo,monospace;font-size:10px;color:#556;min-width:78px;direction:ltr;text-align:start}
.tr .u{font-size:13px}.tr .e{font-size:9px;color:#6b7280;flex:1}.tr .amt{direction:ltr;font-weight:700;font-size:11px}
.sum{position:relative;display:flex;gap:0;background:#fff;border:1px solid #ddd8ca;border-radius:8px;margin:4px 0 8px;flex-wrap:wrap}
.sc{flex:1;min-width:88px;padding:3px 9px;border-inline-end:1px solid #eee9dc}
.sv{font-weight:800;font-size:13px;direction:ltr;unicode-bidi:plaintext;text-align:start}
.stat{position:relative;flex:1;min-width:110px;background:#fff;border:1px solid #ddd8ca;border-radius:9px;padding:5px 9px}
.stat .sv{font-size:18px}.stat.green .sv{color:#12805c}.stat.red .sv{color:#c0392b}
.bigcap{font-size:10pt;color:#52606d;margin:1mm 0}
.stat-bar{display:flex;gap:18px;justify-content:flex-end;font-size:10px;padding:3px 10px;background:#e9e6dd;border-top:1px solid #d6d1c3;direction:ltr}.stat-bar .ok{color:#12805c}
.bd{position:absolute;top:-9px;inset-inline-start:-9px;width:21px;height:21px;border-radius:50%;background:#d92d20;color:#fff;font-style:normal;font-weight:800;font-size:12px;display:flex;align-items:center;justify-content:center;border:2px solid #fff;box-shadow:0 1px 3px rgba(0,0,0,.4);z-index:5;font-family:'InterV',sans-serif;line-height:1;direction:ltr}
.mi .bd,.si .bd{inset-inline-start:auto;inset-inline-end:-6px;top:50%;margin-top:-10px}
.btn .bd{top:-10px}
/* --- plain pages (cover, tables) --- */
.cover h2,.plain h2{font-size:12.5pt;margin:3mm 0 1.2mm;color:#14243f}
.plain p,.plain li{font-size:10.4pt;line-height:1.4}
.plain table.gt{border-collapse:collapse;width:100%;margin:1.5mm 0 2.5mm;font-size:9.3pt}
.plain table.gt th{background:#e8edf5;padding:1.1mm 2mm;text-align:left;border:1px solid #c5d0e3;font-size:9.8pt;direction:ltr}
.plain table.gt td{padding:.9mm 2mm;border:1px solid #d6dbe6;vertical-align:top;direction:ltr;text-align:left;line-height:1.35}
.plain table.gt td .u,.plain table.gt th .u{font-size:10.6pt;line-height:1.4}
.legend{display:flex;gap:3mm;margin:3mm 0}
.legend div{flex:1;border-radius:6px;padding:2.2mm 3mm;font-size:10.4pt;border:1px solid}
code.cmd,pre.cmd{background:#0f172a;color:#e2e8f0;border-radius:6px;font-family:Menlo,monospace;font-size:9.4pt}
pre.cmd{padding:3mm;white-space:pre-wrap;margin:2mm 0;line-height:1.45}
.bigtitle{font-size:34pt;font-weight:800;color:#14243f;line-height:1.1;margin:18mm 0 3mm}
.sub2{font-size:15pt;color:#475569;margin-bottom:8mm}
.kbd{display:inline-block;background:#eef2f7;border:1px solid #b7c2d4;border-bottom-width:2px;border-radius:4px;padding:0 5px;font-family:Menlo,monospace;font-size:9.6pt;font-weight:700}
"""


def document(pages):
    return f'<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Mandi Software: test book</title><style>{CSS}</style></head><body>{"".join(pages)}</body></html>'


def plainpage(part, title_en, title_ur, inner):
    col = PART_COLOURS.get(part, "#334155")
    return (f'<section class="page plain"><div class="head" style="background:{col}"><span>{part}</span><span class="pn">@@</span></div>'
            f'<h1><span>{txt(title_en)}</span><span class="u h1u">{title_ur}</span></h1>{inner}</section>')


def gt(heads, rows, widths=None):
    th = "".join(f'<th{(" style=width:" + widths[i]) if widths else ""}>{txt(h)}</th>' for i, h in enumerate(heads))
    body = "".join("<tr>" + "".join(f"<td>{txt(c)}</td>" for c in r) + "</tr>" for r in rows)
    return f'<table class="gt"><tr>{th}</tr>{body}</table>'
