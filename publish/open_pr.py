#!/usr/bin/env python3
"""把 Stock Watch 提成 openidea-plugins 的 PR。

用法：
    python publish/open_pr.py --dry-run    # 只打印计划，不动远程
    python publish/open_pr.py              # 真做：fork、推分支、开 PR

流程：
    1. 确保 jiusi666/openidea-plugins 这个 fork 存在（没有就建）；
    2. 把 fork 克隆到 .tmp/registry，基于 upstream/main 开头分支；
    3. 拷进 publishers/jiusi.toml 与 plugins/jiusi.stock-watch.toml，提交并推到 fork；
    4. 调 API 向 l0ng-ai/openidea-plugins 开 PR（标题与正文取自 publish/PR.md）。

凭据取自 git credential（首次推送时存的那份），不写进任何参数或文件。
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

for _stream in (sys.stdout, sys.stderr):
    try:
        _stream.reconfigure(encoding="utf-8", errors="replace")
    except (AttributeError, ValueError):
        pass

ROOT = Path(__file__).resolve().parent.parent
UPSTREAM = "l0ng-ai/openidea-plugins"
OWNER = "jiusi666"
BRANCH = "add-jiusi-stock-watch"
WORK = ROOT / ".tmp" / "registry"
FILES = {
    "publishers/jiusi.toml": ROOT / "publish" / "publishers" / "jiusi.toml",
    "plugins/jiusi.stock-watch.toml": ROOT / "publish" / "plugins" / "jiusi.stock-watch.toml",
}
PR_DOC = ROOT / "publish" / "PR.md"


def token() -> str:
    proc = subprocess.run(
        ["git", "credential", "fill"],
        input="protocol=https\nhost=github.com\n\n",
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        sys.exit("取不到 GitHub 凭据，先在终端跑一次 bash publish/push.sh 登录")
    for line in proc.stdout.splitlines():
        if line.startswith("password="):
            return line.split("=", 1)[1]
    sys.exit("凭据里没有 token")


def api(tok: str, path: str, method: str = "GET", body: dict | None = None):
    req = urllib.request.Request(
        "https://api.github.com" + path,
        method=method,
        headers={
            "Authorization": f"Bearer {tok}",
            "Accept": "application/vnd.github+json",
            "User-Agent": "stock-watch-publish",
        },
    )
    if body is not None:
        req.add_header("Content-Type", "application/json")
        req.data = json.dumps(body).encode()
    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            raw = resp.read()
            return resp.status, json.loads(raw) if raw else {}
    except urllib.error.HTTPError as exc:
        raw = exc.read()
        try:
            return exc.code, json.loads(raw) if raw else {}
        except json.JSONDecodeError:
            return exc.code, {"raw": raw.decode("utf-8", "replace")}


def block(text: str, heading: str, lang: str = "") -> str:
    """取出 `## heading` 之后第一个代码块的内容。"""
    idx = text.find(heading)
    if idx < 0:
        sys.exit(f"PR.md 里找不到「{heading}」")
    m = re.search(r"```" + re.escape(lang) + r"\n(.*?)```", text[idx:], re.S)
    if not m:
        sys.exit(f"「{heading}」后面的代码块没找到")
    return m.group(1).strip()


def run(cmd: list[str], cwd: Path | None = None) -> subprocess.CompletedProcess:
    env = {"GIT_TERMINAL_PROMPT": "0"}
    import os

    proc = subprocess.run(
        cmd,
        cwd=str(cwd) if cwd else None,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        env={**os.environ, **env},
    )
    if proc.returncode != 0:
        print(f"  命令失败：{' '.join(cmd)}")
        print((proc.stdout + proc.stderr).strip()[:1200])
        sys.exit(1)
    return proc


def main() -> None:
    ap = argparse.ArgumentParser(description="开 PR 到 openidea-plugins")
    ap.add_argument("--dry-run", action="store_true", help="只打印计划")
    args = ap.parse_args()

    for rel, src in FILES.items():
        if not src.is_file():
            sys.exit(f"缺少 {src}")
        print(f"  本地 {rel}  ({src.stat().st_size} 字节)")

    doc = PR_DOC.read_text(encoding="utf-8")
    title = block(doc, "## 标题", "")
    body = block(doc, "## 正文", "markdown")
    print(f"  标题 {title}")
    print(f"  正文 {len(body)} 字符")

    if args.dry_run:
        print("\n[dry-run] 将执行：")
        print(f"  1. 确保 fork {OWNER}/{UPSTREAM.split('/')[1]} 存在")
        print(f"  2. 克隆到 {WORK}，基于 upstream/main 建分支 {BRANCH}")
        print("  3. 提交 " + " + ".join(FILES))
        print(f"  4. 向 {UPSTREAM}:main 开 PR")
        return

    tok = token()

    # 1. fork
    st, repo = api(tok, f"/repos/{OWNER}/{UPSTREAM.split('/')[1]}")
    if st == 404:
        print("创建 fork …")
        st, info = api(tok, f"/repos/{UPSTREAM}/forks", "POST", {"default_branch_only": True})
        if st not in (200, 201, 202):
            sys.exit(f"创建 fork 失败：{st} {info}")
        for i in range(40):
            time.sleep(3)
            st, repo = api(tok, f"/repos/{OWNER}/{UPSTREAM.split('/')[1]}")
            if st == 200:
                print(f"  fork 就绪（等了 {(i + 1) * 3} 秒）")
                break
        else:
            sys.exit("fork 迟迟没就绪，稍后重跑本脚本即可")
    else:
        print(f"  fork 已存在：{repo.get('full_name')}")

    # 2. 克隆
    if WORK.exists():
        shutil.rmtree(WORK)
    WORK.parent.mkdir(parents=True, exist_ok=True)
    fork_url = f"https://github.com/{OWNER}/{UPSTREAM.split('/')[1]}.git"
    print(f"克隆 {fork_url} …")
    for attempt in range(3):
        try:
            run(["git", "clone", "--quiet", fork_url, str(WORK)])
            break
        except SystemExit:
            if attempt == 2:
                raise
            print("  clone 失败，5 秒后重试")
            time.sleep(5)

    run(["git", "remote", "add", "upstream", f"https://github.com/{UPSTREAM}.git"], cwd=WORK)
    run(["git", "fetch", "--quiet", "upstream", "main"], cwd=WORK)
    run(["git", "checkout", "--quiet", "-B", BRANCH, "upstream/main"], cwd=WORK)

    # 3. 放文件
    for rel, src in FILES.items():
        dst = WORK / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(src, dst)
        print(f"  写入 {rel}")

    run(["git", "add"] + list(FILES), cwd=WORK)
    staged = run(["git", "status", "--porcelain"], cwd=WORK).stdout.strip()
    if not staged:
        sys.exit("没有差异：fork 里已经是这份内容了")

    msg = (
        "Add jiusi.stock-watch\n\n"
        "Publisher jiusi (owner jiusi666) and the 0.9.5 entry for\n"
        "jiusi.stock-watch, published at\n"
        "https://github.com/jiusi666/openidea-stock-watch/releases/tag/v0.9.5"
    )
    run(
        [
            "git",
            "-c",
            f"user.name={OWNER}",
            "-c",
            f"user.email={OWNER}@users.noreply.github.com",
            "commit",
            "--quiet",
            "-m",
            msg,
        ],
        cwd=WORK,
    )
    print("推送分支 …")
    run(["git", "push", "--quiet", "--force", "origin", BRANCH], cwd=WORK)

    # 4. PR
    st, pr = api(
        tok,
        f"/repos/{UPSTREAM}/pulls",
        "POST",
        {
            "title": title,
            "head": f"{OWNER}:{BRANCH}",
            "base": "main",
            "body": body,
        },
    )
    if st in (200, 201):
        print(f"\nPR 已创建：{pr.get('html_url')}")
        return
    if st == 422:
        print("这个分支的 PR 可能已存在，查一下：")
        st2, pulls = api(tok, f"/repos/{UPSTREAM}/pulls?head={OWNER}:{BRANCH}&state=all")
        if isinstance(pulls, list) and pulls:
            for p in pulls:
                print(f"  #{p['number']} {p['state']} {p['html_url']}")
            return
        sys.exit(f"422 {pr}")
    sys.exit(f"开 PR 失败：{st} {pr}")


if __name__ == "__main__":
    main()
