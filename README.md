# Stock Watch 股票盯盘

A 股自选与持仓的轻量盯盘工具，作为 OpenIDEA 的侧边工具窗口运行。行情来自东方财富公开接口，自选、成本价、持仓数量与界面偏好只保存在本机。

| | |
| --- | --- |
| 插件 id | `fg.stock-watch` |
| 作者 | 九思 |
| 许可 | Apache-2.0 |
| 插件 API | `^1.4` |
| 分类 | tools |

## 它能做什么

| 能力 | 说明 |
| --- | --- |
| 自选股 | 输入 6 位代码或 `sh600519` / `1.600519` 添加，自动归一为 `市场.代码` |
| 双视图 | **卡片**：现价 + 涨跌幅徽章 + 日内区间条，点「全部」展开明细；**表格**：所有已选列直接铺开，不需要点开任何东西 |
| 排序 | 自选与持仓列表都能按住行上下拖动调整顺序（卡片 / 表格通用，带插入指示线），顺序保存在本机；「设置 › 排序」选「按涨跌幅 / 按代码」时改由插件排序，拖动自动停用 |
| 表格列 | 10 个可选列自由勾选，按住表头左右拖动即可调整顺序（带插入指示线，拖动中不怕刷新打断）；列多时底部有常显的横向滚动条（可拖动、可点轨道跳转，滚轮在表格上也能直接横推）；「代码 / 名称」固定在左侧、「持仓 / 删除」固定在右侧 |
| 持仓 | 点「持仓」弹出小窗填写成本价与数量，实时预览市值 / 浮动盈亏 / 盈亏比例，保存即关闭 |
| 大盘 | 上证、深证成指、创业板指三张指数卡，可关闭 |
| 状态栏 | 常驻显示上证指数或持仓浮盈，也可隐藏 |
| 涨跌配色 | `cn` 红涨绿跌（默认）、`us` 绿涨红跌、`mono` 去色仅箭头 |
| 隐匿模式 | 一键（`Alt+Shift+H`）去色、只留箭头与涨跌幅、遮住价格与名称、暂停提醒 |
| 提醒 | 涨跌幅超过阈值弹一次通知，回落自动复位 |

**明确不做**：不下单、不接券商、不给投资建议、不上传持仓、不做 K 线图。

## 安装

市场里还没上架时，有两种装法：

- **从包安装**：`Settings › Plugins › Install from Disk`，选 `fg.stock-watch-0.9.5.oiplugin`。
- **开发期链接**：`openidea-plugin link`（或把本目录写进 `%APPDATA%\ai.l0ng.openidea\plugins\links.json` 的 `paths`），改界面不用重编。

上架流程见 [docs/PUBLISHING.md](docs/PUBLISHING.md)。

## 隐私与权限

插件只申请两项能力，安装时会逐项列出：

| 能力 | 类型 | 用途 |
| --- | --- | --- |
| `storage` | 必需 | 保存自选列表、成本价、持仓数量与界面偏好。数据留在本机，卸载即删除 |
| `net:push2.eastmoney.com` | 可选 | 唯一的网络用途：向东方财富公开行情接口 `https://push2.eastmoney.com/api/qt/ulist.np/get` 轮询报价 |

关于 `net:push2.eastmoney.com` 的几点说明——市场审阅时看的就是这几条：

- 作用仅限于读取公开的股票与指数报价（现价、涨跌、开高低收、成交量、时间戳），不发送任何本地数据，不带账号信息。
- 接口域名是静态白名单，不接受运行时传入的任意地址；用户也可以在 `Settings › Plugins` 里单独撤销这项授权，撤销后插件会提示原因而不是崩溃。
- 请求会带 `Referer: https://quote.eastmoney.com/` 以匹配该接口的常规调用方式，`User-Agent` 标明 `OpenIDEA-StockWatch`。
- 这是第三方免费公开接口，**没有 SLA，可能限频或变更**。插件默认 10 秒轮询、休市时降频；接口不可用时界面会显示错误而不是假装有数据。

**这不是实时行情**。它是定时轮询，延迟取决于刷新间隔与接口本身，不适合任何对时效敏感的场景。

除上述两项外，插件不读项目文件、不执行任何程序、不写项目目录、没有 `process:` 或 `vcs.*` 能力。界面跑在沙箱 iframe 里，本身没有网络权限，所有请求都由 WASM 侧发出并受上面的白名单约束。

## 使用

1. 安装后在右侧工具窗口打开「股票盯盘」。
2. 顶部输入代码添加自选，首次打开会给 3 只演示标的。
3. 点某行「持仓」填成本价与数量，浮盈浮亏由本机计算。
4. 右上角三个按钮：**隐匿**（切换隐匿模式）、**设置**（20 项偏好）、**刷新**（立即拉一次行情）。
5. 删除自选需要点两次：第一次按钮变为「确认删除」，3 秒内再点一次才真正删除。
6. 想调整顺序：按住任意一行上下拖动即可（左侧 `⋮⋮` 是抓手），松手就生效并记住；「设置 › 排序」选的是「自选顺序」时才可拖。
7. 提示都是浮层气泡，出现在面板底部，不占布局、不挡按钮，2 秒后自动消失；刷新、改设置、切视图这类操作不再弹提示，只在页脚状态里更新。

快捷键 `Alt+Shift+H`（macOS `Meta+Shift+H`）切换隐匿模式。命令面板里还有 `Stock Watch: 刷新行情` 与 `Stock Watch: 清空自选`。

设置有两个入口，互不冲突：`Settings › Plugins › Stock Watch 股票盯盘` 提供**默认值**（适合随配置同步到新设备），面板里改过的项写入本机并覆盖默认值。面板里还有「恢复默认」。

## 开发

需要 Rust 与 `wasm32-wasip2` 目标：

```sh
rustup target add wasm32-wasip2
cargo test                                   # 19 个单元测试：行情解析、代码归一、偏好合并、列表重排
cargo build --release --target wasm32-wasip2
cp target/wasm32-wasip2/release/openidea_plugin_stock_watch.wasm dist/plugin.wasm
```

Windows 上如果 MSVC 链接器不可用（例如 Git Bash 的 `link` 覆盖了 `link.exe`），改用 GNU 工具链：`cargo +stable-x86_64-pc-windows-gnu build --target wasm32-wasip2`。

### 界面用例

界面是单文件 `web/index.html`，不依赖构建。`tests/ui/cases/*.js` 是自包含的用例脚本：注入页面后在无头 Chrome 里跑真实鼠标事件，把断言写进 `document.title`；`tests/ui/run.py` 负责注入、执行与汇总。

```sh
python tests/ui/run.py            # 7 组用例：拖动排序、横向滚动、固定列、提示、整体回归
python tests/ui/run.py hscroll    # 只跑名字含 hscroll 的
```

覆盖的场景：表头拖拽（含拖动中刷新、Esc 取消、拖到表格外松手）、列表拖动排序（卡片与表格）、自绘横向滚动条（拖拇指、点轨道、重建后仍可用）、左右固定列对齐、浮层提示不占布局、删除二次确认、持仓弹窗、隐匿模式、按涨跌幅排序时禁用拖动。

### 打包与签名

官方命令行 `openidea-plugin` 需要在目标机器上从 IDE 源码编译，本仓库因此附了一个等价实现（包格式已用官方发布的包反证）：

```sh
pip install blake3 cryptography
python scripts/oiplugin.py check                       # 按安装时的规则检查清单
python scripts/oiplugin.py pack . --key .keys/fg.key --publisher fg
python scripts/oiplugin.py verify dist/fg.stock-watch-0.9.5.oiplugin
```

`pack` 会打印插件摘要与包文件的 SHA-256——后者要填进插件市场的索引。包格式、摘要算法与签名细节见 [docs/PUBLISHING.md](docs/PUBLISHING.md#包格式)。

## 结构

```
plugin.toml            清单：能力、命令、快捷键、视图、状态栏、设置
settings.schema.json   20 项偏好的默认值与枚举
src/lib.rs             插件入口：取数、快照、命令、视图消息、状态栏、提醒
src/market.rs          东方财富响应解析、代码归一、盈亏计算
src/prefs.rs           偏好合并（逐字段）、列顺序规范化
web/index.html         沙箱内界面：无外部资源，靠 postMessage 与 WASM 通信
tests/ui/              界面用例与运行器
scripts/oiplugin.py    打包、签名、校验（官方 CLI 的等价实现）
docs/PUBLISHING.md     上架流程与包格式
publish/               提给 openidea-plugins 的两个条目模板与 PR 草稿
CHANGELOG.md           每个版本改了什么
vendor/                SDK v0.1.40 源码快照（保证离线可构建）
dist/plugin.wasm       构建产物
```

界面在沙箱 iframe 里运行，**没有网络权限**，所有请求都由 WASM 侧发出并受能力约束；页面只负责渲染与交互。

## 数据与免责

行情数据版权归数据提供方。本插件仅用于个人查看公开行情，不构成任何投资建议；据此操作的风险由使用者自行承担。
