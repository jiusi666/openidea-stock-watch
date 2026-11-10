# 上架到 OpenIDEA 插件市场

市场（`Settings › Plugins › Available`）的索引来自仓库 [l0ng-ai/openidea-plugins](https://github.com/l0ng-ai/openidea-plugins)。它只记录"谁发布了什么"：**插件代码留在作者自己的仓库**，索引里每个版本钉住一个 `.oiplugin` 包的地址与 SHA-256，包由发布者的密钥签名。

```
authors repo ──build──▶ .oiplugin（签名）──▶ GitHub Release
                                                  │
                            openidea-plugins PR ──┴─▶ CI：下载包、校验 sha256、
                                                        跑 openidea-plugin check、
                                                        验签 ──▶ index.json ──▶ 市场
```

整条链路有四个约束，决定了下面每一步：

1. 插件 id 必须是 `<publisher>.<name>`，publisher 由 `publishers/<id>.toml` 定义并持有公钥；
2. 包必须用该发布者的私钥签名，签名覆盖 id、版本与内容摘要；
3. 索引条目钉住包文件的 SHA-256，CI 会重新下载核对；
4. 仓库必须公开、开源，并且包能从这份源码构建出来。

## 当前进度

| 步骤 | 状态 |
| --- | --- |
| 清单、能力、README 达到市场要求 | 已完成 |
| 打包 + 签名工具 | 已完成：`scripts/oiplugin.py`（格式已用官方发布的包反证） |
| 发布密钥 | 已生成：`.keys/fg.key`（公钥 `Z7EAivBTVudC8qvolLk+zloEBpwZtreRlU6iCCmt9dI=`） |
| 签名包 | 已生成：`dist/fg.stock-watch-0.9.5.oiplugin`，sha256 `bd098685107381c24b2cc4ab8e3ba9e75eeab8740e0d27f87eb52843b6171a3f` |
| 公开仓库 + Release | **待办**：需要你的 GitHub 账号 |
| registry PR | **待办**：模板已备在 `publish/` |

## 1. 先定 publisher 与 id

- publisher id：`fg`（只允许小写字母、数字、`-`、`_`，最长 32 位）。
- 插件 id：`fg.stock-watch`，与 `plugin.toml` 里的 `id` 一致。
- `publishers/fg.toml` 里的 `owners` 是**可以改这个 publisher 条目的 GitHub 账号**，写你自己的登录名。

publisher id 不必等于 GitHub 用户名，但它一旦出现在市场里就不能改（换 id 等于换一个插件，用户要重装）。想改还来得及：改 `plugin.toml` 的 `id`、重新 `keygen`、重新 `pack`，并同步 `publish/` 下的两个模板。

## 2. 生成发布密钥（每个发布者只做一次）

官方方式：

```sh
openidea-plugin keygen fg.key          # 打印公钥
```

本机方式（官方 CLI 需要在目标机器上编译 IDE 源码，见文末）：

```sh
python scripts/oiplugin.py keygen .keys/fg.key
```

私钥文件里是 32 字节种子的 base64，**不是** PEM/JSON。它等于这个插件的发布权：

- 备份到密码管理器或离线介质，不要提交到仓库（`.gitignore` 已忽略 `.keys/` 与 `*.key`）；
- 丢了就不能再更新自己的插件（只能请维护者撤销旧密钥、换新密钥，用户会被要求确认签名者变更）；
- 泄露了就立刻请维护者撤销，见文末「密钥轮换与撤销」。

`fg` 的公钥是 `Z7EAivBTVudC8qvolLk+zloEBpwZtreRlU6iCCmt9dI=`，`publish/publishers/fg.toml` 里已经填好。

## 3. 构建、检查、打包

```sh
# 1) 构建 wasm（Windows 上用 GNU 工具链，见 README）
cargo build --release --target wasm32-wasip2
cp target/wasm32-wasip2/release/openidea_plugin_stock_watch.wasm dist/plugin.wasm

# 2) 单元测试与界面用例
cargo test
python tests/ui/run.py

# 3) 按安装时的规则检查清单
python scripts/oiplugin.py check .

# 4) 打包并签名
python scripts/oiplugin.py pack . --key .keys/fg.key --publisher fg \
  --out dist/fg.stock-watch-0.9.5.oiplugin

# 5) 自检：解包、重算摘要、验签
python scripts/oiplugin.py verify dist/fg.stock-watch-0.9.5.oiplugin
```

`pack` 会打印两个值，都要留好：

- `digest`：插件内容摘要（BLAKE3），签名覆盖它；
- `sha256`：**包文件**的 SHA-256，填进 registry 条目。

有官方 CLI 时等价命令是 `openidea-plugin build` + `openidea-plugin pack . --key fg.key --publisher fg`；`pack` 之前会先跑一遍 `check`。

## 4. 发 GitHub Release

把包挂到一个 release 上，标签用 `v<version>`：

```sh
git tag v0.9.5 && git push origin v0.9.5
gh release create v0.9.5 dist/fg.stock-watch-0.9.5.oiplugin \
  --title "0.9.5" --notes "列表拖动排序、提示改为浮层、横向浏览重做。"
```

没有 `gh` 就在网页上建 release，把 `dist/fg.stock-watch-0.9.5.oiplugin` 拖进去当附件。

拿到可直接下载的地址，形如：

```
https://github.com/<你的账号>/openidea-stock-watch/releases/download/v0.9.5/fg.stock-watch-0.9.5.oiplugin
```

核对一下这个地址真的能下到同一个文件（CI 会下载它并与 sha256 比对）：

```sh
shasum -a 256 fg.stock-watch-0.9.5.oiplugin
```

## 5. 提 PR 到 openidea-plugins

Fork 仓库，加两个文件（模板在 `publish/`，替换 `<你的 GitHub 账号>` 与仓库地址即可）：

`publishers/fg.toml`

```toml
name = "九思"
owners = ["<你的 GitHub 账号>"]
keys = ["Z7EAivBTVudC8qvolLk+zloEBpwZtreRlU6iCCmt9dI="]
revoked-keys = []
```

`plugins/fg.stock-watch.toml`

```toml
repository = "https://github.com/<你的账号>/openidea-stock-watch"

[[versions]]
version = "0.9.5"
package = "https://github.com/<你的账号>/openidea-stock-watch/releases/download/v0.9.5/fg.stock-watch-0.9.5.oiplugin"
sha256 = "bd098685107381c24b2cc4ab8e3ba9e75eeab8740e0d27f87eb52843b6171a3f"
```

PR 标题写清楚是新增插件，描述里带上：

- 插件做什么、给谁用；
- 每一项能力为什么需要（直接引 README 里那两段）；
- 怎么验证过（`cargo test`、界面用例、`verify` 的输出）。

CI 会做这些事，本地能提前对上的部分都已在第 3 步覆盖：

| CI 检查 | 本地对应 | 说明 |
| --- | --- | --- |
| 下载包、核对 `sha256` | `shasum -a 256` | 地址必须是公开可下载的 https |
| 解包并重算摘要、验签 | `scripts/oiplugin.py verify` | 摘要算法见下节 |
| `openidea-plugin check`（对着最新 release） | `scripts/oiplugin.py check` | 官方那次还会编译组件、核对 API 版本 |
| 条目格式（id、semver、文件名） | 本文件与 `publish/` 模板 | 文件名就是插件 id |

首次提交的 publisher 与新插件由维护者人工审阅：源码、申请的能力与理由、是否和已有插件重复（市场里目前没有行情/股票类插件）。

## 包格式

CI 与 IDE 安装走的是同一套规则，逐条如下（与官方实现一致，可用官方发布的包复核：`scripts/oiplugin.py verify` 对一个 `openidea.checkstyle-0.1.0.oiplugin` 会算出与官方 `index.json` 相同的摘要）。

**容器**：zip，Deflate。条目名用 `/` 分隔，包含 `plugin.toml`、清单引用到的文件，可选地最后跟一个 `SIGNATURE.json`。允许包内容在唯一一个顶层目录里（Finder 打包的 `__MACOSX/`、`.DS_Store`、`._*` 会被忽略）。

**清单引用的文件**：`wasm`、`contributes.views[].web`、`contributes.settings.schema`、`icon` / `icon-dark`。本插件是：

```
dist/plugin.wasm
web/index.html
settings.schema.json
```

**摘要**：BLAKE3，对排序去重后的每个文件依次喂入

```
<相对路径（/ 分隔）> NUL <文件内容> NUL
```

并把 `plugin.toml` 也作为其中一个文件。目录名、时间戳、压缩级别都不进入摘要，所以重新打包不会改变摘要——只有内容变了才会。

**签名**：Ed25519，消息是

```
openidea-plugin/1
<插件 id>
<版本>
<摘要>
```

**SIGNATURE.json**：

```json
{
  "publisher": "fg",
  "key": "Z7EAivBTVudC8qvolLk+zloEBpwZtreRlU6iCCmt9dI=",
  "signature": "…base64…"
}
```

`publisher` 必须出现在索引的 `publishers` 里，`key` 必须是它列出的公钥，且插件 id 以 `<publisher>.` 开头，否则安装时报 "may only publish plugins named fg.*"。

## 审核看什么

- 公开仓库 + 开源许可（本仓库 Apache-2.0），包能从这份源码构建；
- README 说明插件做什么，以及**每一项** `net:` / `process:` 能力为什么需要（README 的「隐私与权限」就是照这个写的）；
- 不含无法从源码构建的混淆或压缩代码（界面是手写单文件，未压缩、未打包）；
- 不和已有插件重复（市场目前无股票/行情类插件）。

## 后续发版

1. 改版本号：`plugin.toml`、`Cargo.toml`、`change-notes`，并在 `CHANGELOG.md` 记一条；
2. 重新构建、跑测试、`pack`、`verify`；
3. 发新的 GitHub Release，拿到新的 `sha256`；
4. 在 `plugins/fg.stock-watch.toml` **新增**一个 `[[versions]]`，**保留旧的**：每个版本记录了它需要的插件 API（`engines.openidea` 与组件实际构建的 API 取交集），旧 IDE 会装它能加载的最新版本；索引对每个 API 要求只留最新一个版本，所以不再需要的旧表可以删；
5. 提 PR，CI 通过后由维护者合并，一小时内进入市场。

已安装用户的更新：签名密钥不变就静默更新；新增能力会重新征求同意。

## 密钥轮换与撤销

换密钥时先让旧密钥给新密钥背书，老用户才不会看到"签名者变更"：

```sh
openidea-plugin endorse "<新公钥>" --key fg.key --publisher fg
```

在 `publishers/fg.toml` 里把新密钥放在 `keys` 最前、保留旧的，并把命令打印的签名写进 `[endorsements]`。丢失或泄露的密钥只能请维护者写进 `revoked-keys`。

## 本机没有官方 CLI 时

`openidea-plugin` 由 `crates/ide-host` 编译产出，需要完整 Rust 工具链；Windows 上要么有 MSVC 链接器，要么有 MinGW 的 `as`/`dlltool`。本机两者都不全，所以仓库里带了一个等价实现：

```sh
pip install blake3 cryptography
python scripts/oiplugin.py --help
```

它做的是同样四件事（keygen / check / pack / verify），不替代官方那次 `check`（官方还会编译组件、核对它实际构建所用的 API 版本）。如果之后能装上官方 CLI，两者产出的摘要应当一致——可以用 `openidea-plugin pack` 与 `scripts/oiplugin.py pack` 对同一目录各打一次包来交叉验证。
