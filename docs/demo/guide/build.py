"""Builds guide.html and the PDF. Run: python3 build.py   (needs Google Chrome for the PDF)."""
import importlib
import subprocess
import sys
from pathlib import Path

import ui

HERE = Path(__file__).resolve().parent
OUT_PDF = HERE.parent / "how-to-test-the-app.pdf"
MODULES = ["p_start", "p_index", "p_sales", "p_cash", "p_rest"]


def main(only=None):
    pages = []
    for name in MODULES:
        if not (HERE / f"{name}.py").exists():
            continue
        sys.path.insert(0, str(HERE))
        mod = importlib.import_module(name)
        pages.extend(mod.pages())
    out = []
    for i, p in enumerate(pages, 1):
        out.append(p.replace("@@", str(i)))
    html = ui.document(out)
    (HERE / "guide.html").write_text(html, encoding="utf-8")
    chrome = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
    subprocess.run([chrome, "--headless=new", "--disable-gpu", "--no-pdf-header-footer",
                    f"--print-to-pdf={OUT_PDF}", (HERE / "guide.html").as_uri()],
                   check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=180)
    print(f"{len(out)} pages -> {OUT_PDF}")


if __name__ == "__main__":
    main()
