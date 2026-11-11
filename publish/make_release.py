#!/usr/bin/env python3
"""建 GitHub Release 并上传 .oiplugin 包。

用法：
    python publish/make_release.py --tag v0.9.6 --package dist/jiusi.stock-watch-0.9.6.oiplugin
    python publish/make_release.py --tag v0.9.6 --dry-run    # 只打印计划

凭据取自 git credential（首次推送时存的那份），不写进任何参数或文件。
发布说明取自 publish/RELEASE-NOTES.md。
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

for _stream in (sys.stdout, sys.stderr):
    try:
        _stream.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

ROOT = Path(__file__).resolve().parent.parent
OWNER = "jiusi666"
REPO = "openidea-stock-watch"
API = "https://api.github.com"


def token() -> str:
    """从 git credential 取已存的 GitHub 凭据。"""
    proc = subprocess.run(
        ["git", "credential", "fill"],
        input="protocol=https\nhost=github.com\n\n",
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        cwd=str(ROOT),
    )
    for line in proc.stdout.splitlines():
        if line.startswith("password="):
            return line[len("password="):]
    raise SystemExit("没取到 GitHub 凭据，先跑一次 git push 让凭据存进管理器")


def call(method: str, url: str, tok: str, payload=None, raw: bytes | None = None,
         content_type: str = "application/json") -> tuple[int, dict]:
    data = raw if raw is not None else (json.dumps(payload).encode() if payload is not None else None)
    req = urllib.request.Request(url, data=data, method=method)
    req.add_header("Authorization", f"Bearer {tok}")
    req.add_header("Accept", "application/vnd.github+json")
    if data is not None:
        req.add_header("Content-Type", content_type)
    try:
        with urllib.request.urlopen(req, timeout=90) as resp:
            body = resp.read().decode("utf-8", "replace")
            return resp.status, (json.loads(body) if body.strip() else {})
    except urllib.error.HTTPError as e:
        body = e.read().decode("utf-8", "replace")
        try:
            return e.code, json.loads(body)
        except Exception:
            return e.code, {"message": body[:400]}


def retry(method, url, tok, **kwargs):
    last = (0, {})
    for attempt in range(4):
        last = call(method, url, tok, **kwargs)
        if last[0] < 500:
            return last
        time.sleep(1.5 * (attempt + 1))
    return last


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--tag", default="v0.9.6")
    ap.add_argument("--package", default="")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    notes = (ROOT / "publish" / "RELEASE-NOTES.md").read_text(encoding="utf-8")
    title = notes.splitlines()[0].lstrip("# ").strip()
    pkg = Path(args.package) if args.package else None
    if pkg and not pkg.is_absolute():
        pkg = ROOT / pkg

    print(f"tag   {args.tag}")
    print(f"title {title}")
    print(f"body  {len(notes)} 字符")
    if pkg:
        print(f"asset {pkg.name} ({pkg.stat().st_size} 字节)")
    if args.dry_run:
        print("（dry-run，不动远程）")
        return 0

    tok = token()
    status, rel = retry("POST", f"{API}/repos/{OWNER}/{REPO}/releases", tok, payload={
        "tag_name": args.tag,
        "name": title,
        "body": notes,
        "draft": False,
        "prerelease": False,
    })
    if status == 422:  # 已存在
        status, rel = retry("GET", f"{API}/repos/{OWNER}/{REPO}/releases/tags/{args.tag}", tok)
        if status != 200:
            print(f"[x] 取已有 release 失败 HTTP {status}: {rel.get('message')}")
            return 1
        rid = rel["id"]
        status, rel = retry("PATCH", f"{API}/repos/{OWNER}/{REPO}/releases/{rid}", tok,
                            payload={"body": notes, "name": title, "draft": False})
        print(f"    已存在，更新说明 HTTP {status}")
    else:
        print(f"    建 release HTTP {status}")

    if status not in (200, 201):
        print(f"[x] 建 release 失败: {rel.get('message')}")
        return 1
    print(f"    页面 {rel.get('html_url')}")

    if pkg:
        existing = {a["name"] for a in rel.get("assets", [])}
        if pkg.name in existing:
            print(f"    附件 {pkg.name} 已存在，跳过上传")
        else:
            upload = rel["upload_url"].split("{")[0]
            status, asset = retry(
                "POST", f"{upload}?name={urllib.parse.quote(pkg.name)}", tok,
                raw=pkg.read_bytes(), content_type="application/octet-stream",
            )
            if status not in (200, 201):
                print(f"[x] 上传附件失败 HTTP {status}: {asset.get('message')}")
                return 1
            print(f"    已上传 {asset.get('name')} ({asset.get('size')} 字节)")

    status, rel = retry("GET", f"{API}/repos/{OWNER}/{REPO}/releases/tags/{args.tag}", tok)
    for a in rel.get("assets", []):
        print(f"    下载地址 {a.get('browser_download_url')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
