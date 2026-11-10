//! Stock Watch — OpenIDEA 自选股看盘插件。
//!
//! 行情来自东方财富公开接口；自选列表、成本价、持仓数量与界面偏好只写在本机
//! `host::storage`，不上传、不接交易、不提供投资建议。
//! Web view 负责渲染与设置面板，本组件负责取数、解析、偏好与本地计算。

mod market;
mod prefs;

use std::cmp::Ordering;
use std::collections::HashMap;

use openidea_plugin::{
    Error, HttpRequest, Invocation, NotificationLevel, Plugin, StatusItem, host, register_plugin,
    views::{View, ViewEvent, Viewport, Views},
};
use prefs::Prefs;
use serde_json::{Value, json};

const VIEW: &str = "watchlist";
const KEY_LIST: &str = "stockwatch:list";
const KEY_LAST: &str = "stockwatch:last";
const API_HOST: &str = "push2.eastmoney.com";
const STATUS_ID: &str = "stockwatch.index";
const INDICES: [(&str, &str); 3] = [
    ("1.000001", "上证指数"),
    ("0.399001", "深证成指"),
    ("0.399006", "创业板指"),
];

#[derive(Default)]
pub struct StockWatch {
    items: Vec<market::Item>,
    last: Vec<market::Quote>,
    prefs: Prefs,
    /// 已提醒过的 secid，避免每次刷新重复弹通知。
    alerted: Vec<String>,
    /// 是否已经从本机 storage 载入过，避免视图消息早于 init 时操作空列表。
    loaded: bool,
}

fn default_items() -> Vec<market::Item> {
    ["1.600519", "0.000001", "0.300750"]
        .iter()
        .map(|id| market::Item::new(id))
        .collect()
}

/// 接口给的是 UTC 秒，按北京时间显示时分秒。
fn clock(timestamp: f64) -> String {
    if timestamp <= 0.0 {
        return "—".to_string();
    }
    let secs = ((timestamp as i64 % 86_400) + 8 * 3_600) % 86_400;
    format!(
        "{:02}:{:02}:{:02}",
        secs / 3_600,
        (secs % 3_600) / 60,
        secs % 60
    )
}

impl StockWatch {
    fn load(&mut self) {
        self.items = host::storage::get(KEY_LIST)
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str::<Vec<market::Item>>(&s).ok())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(default_items);
        self.last = host::storage::get(KEY_LAST)
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str::<Vec<market::Quote>>(&s).ok())
            .unwrap_or_default();
        self.prefs = Prefs::load();
        self.loaded = true;
    }

    /// 视图可能在 init 之前就发来消息（例如插件实例被回收后重新激活）。
    fn ensure_loaded(&mut self) {
        if !self.loaded {
            self.load();
        }
    }

    fn save_items(&self) {
        if let Ok(s) = serde_json::to_string(&self.items) {
            let _ = host::storage::set(KEY_LIST, &s);
        }
    }

    fn save_last(&self) {
        if let Ok(s) = serde_json::to_string(&self.last) {
            let _ = host::storage::set(KEY_LAST, &s);
        }
    }

    /// 按视图传来的 secid 顺序重排自选。没在列表里出现的条目保持原相对顺序补在末尾，
    /// 这样即使前端只看到「持仓」分页，也不会因为一次拖动丢掉别的自选。返回顺序是否真的变了。
    fn apply_order(&mut self, order: &[String]) -> bool {
        let before: Vec<String> = self.items.iter().map(|i| i.secid.clone()).collect();
        let mut rest = std::mem::take(&mut self.items);
        let mut next = Vec::with_capacity(rest.len());
        for secid in order {
            if let Some(pos) = rest.iter().position(|i| &i.secid == secid) {
                next.push(rest.remove(pos));
            }
        }
        next.append(&mut rest);
        let changed = before != next.iter().map(|i| i.secid.clone()).collect::<Vec<_>>();
        self.items = next;
        changed
    }

    fn by_id(&self) -> HashMap<&str, &market::Quote> {
        self.last.iter().map(|q| (q.secid.as_str(), q)).collect()
    }

    fn total_profit(&self) -> f64 {
        let by_id = self.by_id();
        self.items
            .iter()
            .filter_map(|item| {
                by_id
                    .get(item.secid.as_str())
                    .copied()
                    .and_then(|q| market::profit(item, q))
            })
            .sum()
    }

    fn snapshot(&self, message: &str) -> Value {
        let by_id = self.by_id();
        let mut rows: Vec<Value> = self
            .items
            .iter()
            .map(|item| {
                let q = by_id.get(item.secid.as_str()).copied();
                let profit = q.and_then(|q| market::profit(item, q));
                let profit_percent = match (q, item.cost) {
                    (Some(q), cost) if cost > 0.0 => Some((q.price - cost) / cost * 100.0),
                    _ => None,
                };
                json!({
                    "secid": item.secid,
                    "code": item.code,
                    "name": q.map(|q| q.name.as_str()).unwrap_or("-"),
                    "price": q.map(|q| q.price).unwrap_or(0.0),
                    "change": q.map(|q| q.change).unwrap_or(0.0),
                    "percent": q.map(|q| q.percent).unwrap_or(0.0),
                    "open": q.map(|q| q.open).unwrap_or(0.0),
                    "high": q.map(|q| q.high).unwrap_or(0.0),
                    "low": q.map(|q| q.low).unwrap_or(0.0),
                    "prevClose": q.map(|q| q.prev_close).unwrap_or(0.0),
                    "volume": q.map(|q| q.volume).unwrap_or(0.0),
                    "cost": item.cost,
                    "qty": item.qty,
                    "profit": profit,
                    "profitPercent": profit_percent,
                })
            })
            .collect();

        match self.prefs.sort_by.as_str() {
            "percent" => rows.sort_by(|a, b| {
                b["percent"]
                    .as_f64()
                    .unwrap_or(0.0)
                    .partial_cmp(&a["percent"].as_f64().unwrap_or(0.0))
                    .unwrap_or(Ordering::Equal)
            }),
            "code" => rows.sort_by(|a, b| {
                a["code"]
                    .as_str()
                    .unwrap_or("")
                    .cmp(b["code"].as_str().unwrap_or(""))
            }),
            _ => {}
        }

        let indices: Vec<Value> = INDICES
            .iter()
            .map(|(secid, label)| {
                let q = by_id.get(*secid).copied();
                json!({
                    "name": label,
                    "price": q.map(|q| q.price).unwrap_or(0.0),
                    "percent": q.map(|q| q.percent).unwrap_or(0.0),
                })
            })
            .collect();

        let updated_at = self
            .last
            .first()
            .map(|q| clock(q.timestamp))
            .unwrap_or_else(|| "—".to_string());

        json!({
            "type": "snapshot",
            "rows": rows,
            "indices": indices,
            "updatedAt": updated_at,
            "source": "东方财富（公开接口，轮询）",
            "intervalSec": self.prefs.interval_sec,
            "prefs": self.prefs,
            "message": message,
        })
    }

    fn push(&self, message: &str) {
        let _ = host::post_view_message(VIEW, &self.snapshot(message).to_string());
    }

    fn update_status(&self) {
        let command = Some("refresh".to_string());
        let tooltip = Some("Stock Watch：点击刷新行情".to_string());
        match self.prefs.status_bar_mode.as_str() {
            "hidden" => host::remove_status_item(STATUS_ID),
            "profit" => {
                let total = self.total_profit();
                host::set_status_item(
                    STATUS_ID,
                    &StatusItem {
                        text: format!("持仓盈亏 {:+.0}", total),
                        tooltip,
                        command,
                    },
                );
            }
            _ => {
                // 隐匿模式下状态栏也不该泄露行情：中性占位符，仍然可以点击刷新。
                let item = if self.prefs.stealth {
                    StatusItem {
                        text: "··".into(),
                        tooltip: Some("Stock Watch：隐匿模式已开启，点击刷新".into()),
                        command,
                    }
                } else if let Some(q) = self.last.iter().find(|q| q.secid == INDICES[0].0) {
                    StatusItem {
                        text: format!("上证 {:.2} {:+.2}%", q.price, q.percent),
                        tooltip,
                        command,
                    }
                } else {
                    return;
                };
                host::set_status_item(STATUS_ID, &item);
            }
        }
    }

    fn check_alerts(&mut self) {
        // 隐匿模式的意义就是不被注意到：通知一律不发。
        if self.prefs.stealth || self.prefs.alert_percent <= 0.0 {
            return;
        }
        let threshold = self.prefs.alert_percent;
        for q in &self.last {
            if INDICES.iter().any(|(id, _)| *id == q.secid) {
                continue;
            }
            if q.percent.abs() >= threshold {
                if !self.alerted.contains(&q.secid) {
                    self.alerted.push(q.secid.clone());
                    host::notify(
                        NotificationLevel::Info,
                        "Stock Watch 涨跌提醒",
                        &format!("{} {:.2}（{:+.2}%）", q.name, q.price, q.percent),
                    );
                }
            } else {
                self.alerted.retain(|id| id != &q.secid);
            }
        }
    }

    fn refresh(&mut self) {
        self.ensure_loaded();
        let mut ids: Vec<String> = INDICES.iter().map(|(id, _)| id.to_string()).collect();
        for item in &self.items {
            if !ids.contains(&item.secid) {
                ids.push(item.secid.clone());
            }
        }
        let url = format!(
            "https://{API_HOST}/api/qt/ulist.np/get?secids={}&fields=f2,f3,f4,f5,f6,f12,f13,f14,f15,f16,f17,f18,f124",
            ids.join(",")
        );
        let request = HttpRequest::get(&url)
            .header("User-Agent", "OpenIDEA-StockWatch/0.2")
            .header("Referer", "https://quote.eastmoney.com/");

        match host::fetch(&request) {
            Ok(response) if response.status == 200 => match market::parse_quotes(&response.body) {
                Ok(quotes) => {
                    self.last = quotes;
                    self.save_last();
                    self.update_status();
                    self.check_alerts();
                    self.push("");
                }
                Err(e) => self.push(&e),
            },
            Ok(response) => self.push(&format!("行情接口返回 HTTP {}", response.status)),
            Err(Error::Denied(_)) => {
                self.push("未授予 net:push2.eastmoney.com 权限，请在 Settings › Plugins 中开启")
            }
            Err(e) => self.push(&format!("行情请求失败：{e}")),
        }
    }

    fn set_stealth(&mut self, on: bool) {
        self.prefs.stealth = on;
        self.prefs.save();
        if !on {
            self.alerted.clear();
        }
        self.update_status();
        self.push(if on {
            "隐匿模式已开启"
        } else {
            "隐匿模式已关闭"
        });
    }

    fn on_message(&mut self, text: &str) {
        let msg: Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(_) => return,
        };
        let kind = msg.get("type").and_then(Value::as_str).unwrap_or("");
        if kind != "init" {
            self.ensure_loaded();
        }
        match kind {
            "init" => {
                self.load();
                self.update_status();
                self.push("");
                self.refresh();
            }
            "refresh" => self.refresh(),
            "prefs" => {
                if let Some(patch) = msg.get("patch") {
                    self.prefs.merge_patch(patch);
                    self.prefs.save();
                    self.update_status();
                    self.push("设置已保存");
                }
            }
            "toggle-stealth" => self.set_stealth(!self.prefs.stealth),
            "add" => {
                let raw = msg.get("code").and_then(Value::as_str).unwrap_or("");
                match market::normalize(raw) {
                    Some(secid) => {
                        if !self.items.iter().any(|i| i.secid == secid) {
                            self.items.push(market::Item::new(&secid));
                            self.save_items();
                        }
                        self.refresh();
                    }
                    None => self.push("代码格式不对：请输入 6 位数字，或 sh600519 / 1.600519"),
                }
            }
            "reorder" => {
                let order: Vec<String> = msg
                    .get("order")
                    .and_then(Value::as_array)
                    .map(|a| {
                        a.iter()
                            .filter_map(Value::as_str)
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default();
                if !order.is_empty() && self.apply_order(&order) {
                    self.save_items();
                }
                self.push("");
            }
            "remove" => {
                let secid = msg.get("secid").and_then(Value::as_str).unwrap_or("");
                let before = self.items.len();
                self.items.retain(|i| i.secid != secid);
                if self.items.len() != before {
                    self.save_items();
                }
                self.push("");
            }
            "position" => {
                let secid = msg.get("secid").and_then(Value::as_str).unwrap_or("");
                let code = msg.get("code").and_then(Value::as_str).unwrap_or("");
                let cost = msg.get("cost").and_then(Value::as_f64).unwrap_or(0.0);
                let qty = msg.get("qty").and_then(Value::as_f64).unwrap_or(0.0);
                // 前端同时带 secid 与 code，任一命中即可，兼容旧版缓存里缺失 secid 的条目。
                match self
                    .items
                    .iter_mut()
                    .find(|i| i.secid == secid || (!code.is_empty() && i.code == code))
                {
                    Some(item) => {
                        item.cost = cost.max(0.0);
                        item.qty = qty.max(0.0);
                        self.save_items();
                        self.update_status();
                        self.push("持仓已保存到本机");
                    }
                    None => self.push("请先把这只股票加入自选"),
                }
            }
            _ => {}
        }
    }
}

impl Plugin for StockWatch {
    fn run_command(
        &mut self,
        id: &str,
        _ctx: &Invocation,
        _arg: Option<&str>,
    ) -> Result<Option<String>, Error> {
        self.ensure_loaded();
        match id {
            "refresh" => {
                self.refresh();
                Ok(None)
            }
            "clear" => {
                self.items.clear();
                self.save_items();
                self.push("自选已清空");
                Ok(None)
            }
            "toggle-stealth" => {
                self.set_stealth(!self.prefs.stealth);
                Ok(None)
            }
            _ => Err(Error::NotFound(format!("command {id}"))),
        }
    }
}

impl Views for StockWatch {
    fn render(
        &mut self,
        _view: &str,
        _viewport: Option<Viewport>,
        _ctx: &Invocation,
    ) -> Result<View, Error> {
        // 声明了 web view，实际界面由 web/index.html 绘制。
        Ok(View::message("正在加载行情…"))
    }

    fn handle(&mut self, _view: &str, event: ViewEvent, _ctx: &Invocation) -> Result<(), Error> {
        if let ViewEvent::Message(text) = event {
            self.on_message(&text);
        }
        Ok(())
    }
}

register_plugin!(StockWatch, views);

#[cfg(test)]
mod tests {
    use super::*;

    fn watch(ids: &[&str]) -> StockWatch {
        StockWatch {
            items: ids.iter().map(|id| market::Item::new(id)).collect(),
            loaded: true,
            ..Default::default()
        }
    }

    fn ids(p: &StockWatch) -> Vec<&str> {
        p.items.iter().map(|i| i.secid.as_str()).collect()
    }

    #[test]
    fn reorder_follows_the_view_order() {
        let mut p = watch(&["1.600519", "0.000001", "0.300750"]);
        assert!(p.apply_order(&["0.300750".into(), "1.600519".into(), "0.000001".into()]));
        assert_eq!(ids(&p), ["0.300750", "1.600519", "0.000001"]);
    }

    #[test]
    fn reorder_keeps_items_the_view_did_not_mention() {
        // 视图只看得到「持仓」分页时，也不该把别的自选弄丢或打乱。
        let mut p = watch(&["1.600519", "0.000001", "0.300750"]);
        assert!(p.apply_order(&["0.300750".into()]));
        assert_eq!(ids(&p), ["0.300750", "1.600519", "0.000001"]);
    }

    #[test]
    fn reorder_ignores_unknown_ids_and_no_ops() {
        let mut p = watch(&["1.600519", "0.000001"]);
        assert!(!p.apply_order(&["1.600519".into(), "0.000001".into()]));
        assert!(!p.apply_order(&["9.999999".into()]));
        assert_eq!(ids(&p), ["1.600519", "0.000001"]);
    }
}
