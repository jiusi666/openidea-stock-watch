#!/usr/bin/env python3
"""发布后自检：确认 Release 附件与 registry 条目一致。

用法：
    python publish/check_release.py            # 下载远端附件核对
    python publish/check_release.py --local    # 只核对本地 dist/ 下的包

检查项：
    1. package 地址能下载到文件（占位或 404 会直接报错）；
    2. 下载文件的 sha256 与 registry 条目里的 sha256 相同；
    3. 包本身能通过摘要重算与签名验证（oiplugin.py verify）；
    4. 本地 dist/ 与远端是同一份字节。

任何一项不通过都会以非零状态退出，方便在提 PR 前跑一遍。
"""

from __future__ import annotations

import argparse
import hashlib
import subprocess
import sys
import tempfile
import tomllib
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ENTRY = ROOT / "publish" / "plugins" / "jiusi.stock-watch.toml"
TOOL = ROOT / "scripts" / "oiplugin.py"


def fail(msg: str) -> None:
    print(f"  ✗ {msg}")
    sys.exit(1)


def sha256_of(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def fetch(url: str) -> bytes:
    req = urllib.request.Request(url, headers={"User-Agent": "stock-watch-check"})
    with urllib.request.urlopen(req, timeout=60) as resp:
        return resp.read()


def main() -> None:
    parser = argparse.ArgumentParser(description="核对发布包与 registry 条目")
    parser.add_argument(
        "--local",
        action="store_true",
        help="不联网，只核对本地 dist/ 下的包",
    )
    args = parser.parse_args()

    if not ENTRY.is_file():
        fail(f"找不到条目文件 {ENTRY}")
    entry = tomllib.loads(ENTRY.read_text(encoding="utf-8"))
    versions = entry.get("versions") or []
    if not versions:
        fail("条目里没有 [[versions]]")

    ok = True
    for ver in versions:
        version = ver.get("version", "?")
        url = ver.get("package", "")
        want = ver.get("sha256", "").lower()
        local = ROOT / "dist" / f"jiusi.stock-watch-{version}.oiplugin"

        print(f"版本 {version}")
        print(f"  地址 {url}")

        if args.local:
            if not local.is_file():
                fail(f"本地缺少 {local}")
            blob = local.read_bytes()
            print("  来源 本地 dist/")
        else:
            try:
                blob = fetch(url)
            except urllib.error.HTTPError as exc:
                fail(f"下载失败：HTTP {exc.code}（Release 还没发或文件名不一致）")
            except Exception as exc:  # noqa: BLE001
                fail(f"下载失败：{exc}")
            print(f"  来源 远端（{len(blob)} 字节）")

        got = sha256_of(blob)
        if got != want:
            ok = False
            print(f"  ✗ sha256 不一致\n      条目 {want}\n      实际 {got}")
        else:
            print(f"  ✓ sha256 {got}")

        with tempfile.TemporaryDirectory() as tmp:
            pkg = Path(tmp) / "pkg.oiplugin"
            pkg.write_bytes(blob)
            proc = subprocess.run(
                [sys.executable, str(TOOL), "verify", str(pkg)],
                capture_output=True,
                text=True,
            )
            if proc.returncode != 0:
                ok = False
                print("  ✗ 包校验失败")
                print((proc.stdout + proc.stderr).strip())
            else:
                lines = [ln.strip() for ln in (proc.stdout or "").splitlines() if ln.strip()]
                head = next((ln for ln in lines if ln.startswith("ok")), "验签通过")
                digest = next((ln for ln in lines if ln.startswith("digest")), "")
                print(f"  ✓ {head}")
                if digest:
                    print(f"  ✓ {digest}")

        if not args.local and local.is_file():
            if local.read_bytes() != blob:
                ok = False
                print("  ✗ 远端附件与本地 dist/ 不是同一份字节")
            else:
                print("  ✓ 与本地 dist/ 一致")

    print()
    print("全部通过，可以提 PR。" if ok else "存在问题，先别提 PR。")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
