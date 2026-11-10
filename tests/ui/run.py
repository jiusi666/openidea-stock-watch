#!/usr/bin/env python3
"""在沙箱页面上跑界面用例。

每个用例是一个自包含的脚本（tests/ui/cases/*.js），注入 web/index.html 之后
在无头 Chrome 里执行，把断言结果写进 `document.title`。这个运行器只负责：
注入、跑、把标题里的结果读回来、按失败标记汇总。

用法：
    python tests/ui/run.py                 # 全部用例
    python tests/ui/run.py hscroll sticky  # 名字里含这些片段用例

退出码 0 表示全部通过。需要 Chrome（用 CHROME 环境变量可指定路径）。
"""

from __future__ import annotations

import argparse
import html
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
PAGE = ROOT / "web" / "index.html"
CASES = Path(__file__).resolve().parent / "cases"

# 面板是侧边工具窗口，用例都在这个宽度下跑；个别用例需要更宽的行来拖动。
SIZE = "650,800"

CHROME_CANDIDATES = [
    os.environ.get("CHROME", ""),
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "google-chrome",
    "chromium",
    "chromium-browser",
]

FAIL_MARKS = ("✗", "FAIL", "fail:")


def find_chrome() -> str:
    for candidate in CHROME_CANDIDATES:
        if not candidate:
            continue
        found = shutil.which(candidate) or (candidate if Path(candidate).is_file() else None)
        if found:
            return found
    sys.exit("找不到 Chrome；用 CHROME=/path/to/chrome 指定。")


def run_case(chrome: str, name: str, script: str, workdir: Path) -> tuple[bool, str]:
    page = PAGE.read_text(encoding="utf-8")
    if "</body>" not in page:
        sys.exit(f"{PAGE} 里没有 </body>，无法注入用例。")
    html_path = workdir / f"{name}.html"
    html_path.write_text(
        page.replace("</body>", f"<script>\n{script}\n</script>\n</body>"),
        encoding="utf-8",
    )
    proc = subprocess.run(
        [
            chrome,
            "--headless=new",
            "--disable-gpu",
            "--no-first-run",
            "--disable-extensions",
            f"--window-size={SIZE}",
            "--virtual-time-budget=6000",
            "--dump-dom",
            html_path.as_uri(),
        ],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=120,
    )
    match = re.search(r"<title>(.*?)</title>", proc.stdout, re.S)
    title = html.unescape(match.group(1)).strip() if match else ""
    if not title:
        return False, "用例没有输出结果（脚本异常或超时）"
    failed = any(mark in title for mark in FAIL_MARKS)
    return not failed, title


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("filter", nargs="*", help="只跑名字里含这些片段的用例")
    args = parser.parse_args()

    chrome = find_chrome()
    cases = sorted(CASES.glob("*.js"))
    if args.filter:
        cases = [c for c in cases if any(f in c.stem for f in args.filter)]
    if not cases:
        sys.exit("没有匹配的用例。")

    print(f"Chrome: {chrome}")
    print(f"面板页: {PAGE.relative_to(ROOT)}\n")

    failures = 0
    with tempfile.TemporaryDirectory(prefix="stockwatch-ui-") as tmp:
        workdir = Path(tmp)
        for case in cases:
            ok, detail = run_case(chrome, case.stem, case.read_text(encoding="utf-8"), workdir)
            mark = "PASS" if ok else "FAIL"
            print(f"[{mark}] {case.stem}")
            print(f"       {detail[:400]}")
            if not ok:
                failures += 1

    print(f"\n{len(cases) - failures}/{len(cases)} 通过")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
