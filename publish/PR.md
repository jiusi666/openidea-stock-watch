# PR：新增插件 jiusi.stock-watch

> 这份是提给 [l0ng-ai/openidea-plugins](https://github.com/l0ng-ai/openidea-plugins) 的 PR 正文草稿。
> 提交前把 `<你的账号>` 换成实际 GitHub 账号，并确认两个文件已按 `publish/` 里的模板放进 fork。

## 标题

```
Add jiusi.stock-watch — A-share watchlist and positions in a side panel
```

## 正文

```markdown
### What it is

`jiusi.stock-watch` — a side-panel watchlist for A-shares. Add a 6-digit code, see
price / change / intraday range on a card, or switch to a table with 10 optional
columns; record cost and quantity to get floating P/L computed locally. Stealth
mode hides names and prices and drops colours, leaving only arrows, for when the
panel is visible to someone else.

Source: https://github.com/<你的账号>/openidea-stock-watch (Apache-2.0).

### Capabilities

`storage` (required) — the watchlist, cost basis, position size and the 20 UI
preferences. Local only; removed with the plugin.

`net:push2.eastmoney.com` (optional) — one purpose: polling public quotes from
Eastmoney's documented public endpoint
(`GET https://push2.eastmoney.com/api/qt/ulist.np/get`). Static host allowlist;
no local data is sent, no account, no cookies; the user can revoke it in
Settings › Plugins, and the panel then says why instead of failing silently.

No `process:`, no `vcs.*`, no project or filesystem access. The panel UI runs in
the sandboxed view with no network of its own — every request goes through the
WASM side and the allowlist above.

Not real-time: it polls (default 10 s, slower when the market is closed). The
README says so, and the panel shows an error rather than stale numbers when the
endpoint fails.

### Verification

- `cargo test` — 19 unit tests (quote parsing, code normalisation, preference
  merging, list reordering).
- `python tests/ui/run.py` — 7 headless-Chrome cases driving the real panel:
  column drag, list drag, horizontal scroll, sticky columns, toast placement,
  delete confirmation, position dialog, stealth mode.
- `python scripts/oiplugin.py verify dist/jiusi.stock-watch-0.9.5.oiplugin` —
  unpacks, recomputes the digest and verifies the signature.

### Notes

- The wasm is built from this source with `wasm32-wasip2` + component model
  (`engines.openidea = "^1.4"`), no minified or generated-only code; the UI is a
  hand-written single HTML file.
- Package: `jiusi.stock-watch-0.9.5.oiplugin`,
  sha256 `c231f7f13724f79dce86ec709f2e329d5e6587224bfb67f3251f008880da92b0`.
```

## 检查清单

- [ ] `publish/publishers/jiusi.toml` 与 `publish/plugins/jiusi.stock-watch.toml` 已复制到 fork 的对应目录
- [ ] `owners` 是真实 GitHub 登录名
- [ ] `repository` 与 `package` 里的 `<你的账号>` 已替换，且 Release 已发布
- [ ] `package` 地址在未登录的浏览器里能直接下载到同名文件
- [ ] `sha256` 与下载到的文件一致（`shasum -a 256 jiusi.stock-watch-0.9.5.oiplugin`）
- [ ] README 的「隐私与权限」仍与清单里的能力一致
