# 更新 PR：jiusi.stock-watch → 0.9.6

> 提给 [l0ng-ai/openidea-plugins](https://github.com/l0ng-ai/openidea-plugins) 的更新 PR 正文。
> 结构固定为「## 标题 / ## 正文」，`openidea-plugin-update` 的 `update_pr.py` 会直接读这两段。

## 标题

```
Update jiusi.stock-watch to 0.9.6
```

## 正文

```markdown
### jiusi.stock-watch → 0.9.6

Adds `0.9.6` to the version list; `0.9.5` stays published for anyone on an older
`engines.openidea`. Release:
https://github.com/jiusi666/openidea-stock-watch/releases/tag/v0.9.6

#### What changed since 0.9.5

**Quote source moved from Eastmoney to Tencent.** The Eastmoney endpoint answers
with a 302 to a second host, so every poll cost two round trips — measured 4–7 s
on a normal connection. The host gives a single plugin call a 5 s budget, so the
plugin was repeatedly killed and restarted for timing out. `https://qt.gtimg.cn/q=`
returns every quote in one request in 0.2–0.3 s with no redirect. The response is
GBK and is decoded inside the WASM module with `encoding_rs` — no extra service,
no plaintext proxy, nothing leaves the machine.

**Fix: silent install failure.** The package icon SVGs carried `role="img"`.
Plugin icons are validated against a strict element/attribute allowlist that does
not include `role`, so the whole package check failed *before* the consent dialog:
clicking "install from package" did nothing at all. The attribute is removed and
both SVGs pass the allowlist check.

**New: icons.** `icon` / `icon-dark` for the plugin list, plus a separate
16px-optimised `icons/view.svg` used as the side-panel view title icon.

**Smaller:** number inputs no longer show the WebView spinner arrows; the request
carries a 4 s fallback timeout so a slow endpoint fails with a readable message
instead of being killed by the host; the panel footer now names the source it is
actually using.

#### Capabilities

Unchanged in kind — `storage` (required) plus one network host, now
`net:qt.gtimg.cn` instead of `net:push2.eastmoney.com`. The old host is removed
from the manifest, so only the new one is requested. Static allowlist, no local
data sent, no account, no cookies, revocable in Settings › Plugins.

Not real-time: it polls (default 10 s, slower when the market is closed). The
README says so.

#### Verification

- `cargo test` — 20 unit tests (quote parsing including a GBK fixture, code
  normalisation, preference merging, list reordering)
- `python scripts/oiplugin.py verify dist/jiusi.stock-watch-0.9.6.oiplugin` —
  digest and Ed25519 signature
- `python publish/check_release.py` — downloads the asset from the Release and
  compares its sha256 with the registry entry

#### Package

- `jiusi.stock-watch-0.9.6.oiplugin`
- sha256 `f68e10fffc94739dffd6b7b1a115cdb7117f01a6501e65938b1f7d558837b45f`
- Source: https://github.com/jiusi666/openidea-stock-watch (Apache-2.0)
```
