//! 用户偏好。
//!
//! 两个入口，一份配置：IDE 的 Settings › Plugins（settings.schema.json）提供默认值，
//! Web view 面板里的改动写进 `host::storage` 覆盖它。面板改过的字段以 storage 为准。

use openidea_plugin::host;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const KEY: &str = "stockwatch:prefs";

/// 卡片上可直接显示的列（现价与涨跌幅常驻，不在此列表里）。
pub const COLUMNS: [&str; 10] = [
    "change",
    "prevClose",
    "open",
    "high",
    "low",
    "volume",
    "cost",
    "qty",
    "profit",
    "profitPercent",
];

pub fn preset_columns(preset: &str) -> Vec<String> {
    let keys: &[&str] = match preset {
        "compact" => &["change", "prevClose"],
        "full" => &COLUMNS,
        _ => &["change", "prevClose", "volume"],
    };
    keys.iter().map(|s| s.to_string()).collect()
}

/// 表格列的自定义顺序：默认与 [`COLUMNS`] 一致，用户可以拖动表头调整。
pub fn default_order() -> Vec<String> {
    COLUMNS.iter().map(|s| s.to_string()).collect()
}

fn one_of(value: &mut String, allowed: &[&str], fallback: &str) {
    if !allowed.contains(&value.as_str()) {
        *value = fallback.to_string();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Prefs {
    /// cn 红涨绿跌 / us 绿涨红跌 / mono 无颜色
    pub color_scheme: String,
    /// 隐匿模式：去色、仅箭头、压缩信息、隐藏名称与价格、停通知
    pub stealth: bool,
    pub stealth_hide_names: bool,
    pub stealth_mask_price: bool,
    /// index / profit / hidden
    pub status_bar_mode: String,
    /// compact / comfortable
    pub density: String,
    /// small / normal / large
    pub font_size: String,
    /// minimal / normal / full
    pub detail: String,
    pub show_indices: bool,
    /// tabs 自选/持仓分页 / all 合并列表 / positions 仅持仓
    pub list_mode: String,
    /// card 卡片（列网格 + 可展开全部）/ table 表格（所有列直接铺成表头）
    pub view_style: String,
    /// 卡片上直接铺开的列，不再需要点「明细」
    pub columns: Vec<String>,
    /// 表格可选列的显示顺序（可拖动表头调整，只含 [`COLUMNS`] 里的键）
    pub column_order: Vec<String>,
    /// compact / normal / full / custom
    pub column_preset: String,
    pub show_header_labels: bool,
    /// 卡片上是否显示「日内区间」条
    pub show_range_bar: bool,
    pub interval_sec: u32,
    /// custom / percent / code
    pub sort_by: String,
    /// amount / percent / hidden
    pub profit_display: String,
    pub alert_percent: f64,
    pub quiet_hours: bool,
    pub decimals: u32,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            color_scheme: "cn".into(),
            stealth: false,
            stealth_hide_names: true,
            stealth_mask_price: true,
            status_bar_mode: "index".into(),
            density: "compact".into(),
            font_size: "normal".into(),
            detail: "normal".into(),
            show_indices: true,
            list_mode: "tabs".into(),
            view_style: "card".into(),
            columns: preset_columns("normal"),
            column_order: default_order(),
            column_preset: "normal".into(),
            show_header_labels: true,
            show_range_bar: true,
            interval_sec: 10,
            sort_by: "custom".into(),
            profit_display: "amount".into(),
            alert_percent: 0.0,
            quiet_hours: true,
            decimals: 2,
        }
    }
}

impl Prefs {
    pub fn load() -> Self {
        if let Some(s) = host::storage::get(KEY).ok().flatten() {
            if let Ok(mut p) = serde_json::from_str::<Prefs>(&s) {
                p.normalize();
                return p;
            }
        }
        let mut p = Self::from_settings();
        p.normalize();
        p
    }

    pub fn save(&self) {
        if let Ok(s) = serde_json::to_string(self) {
            let _ = host::storage::set(KEY, &s);
        }
    }

    fn from_settings() -> Self {
        let d = Self::default();
        let mut p = Self {
            color_scheme: host::setting_string("colorScheme")
                .unwrap_or_else(|| d.color_scheme.clone()),
            stealth: host::setting_bool("stealth").unwrap_or(d.stealth),
            stealth_hide_names: host::setting_bool("stealthHideNames")
                .unwrap_or(d.stealth_hide_names),
            stealth_mask_price: host::setting_bool("stealthMaskPrice")
                .unwrap_or(d.stealth_mask_price),
            status_bar_mode: host::setting_string("statusBarMode")
                .unwrap_or_else(|| d.status_bar_mode.clone()),
            density: host::setting_string("density").unwrap_or_else(|| d.density.clone()),
            font_size: host::setting_string("fontSize").unwrap_or_else(|| d.font_size.clone()),
            detail: host::setting_string("detail").unwrap_or_else(|| d.detail.clone()),
            show_indices: host::setting_bool("showIndices").unwrap_or(d.show_indices),
            list_mode: host::setting_string("listMode").unwrap_or_else(|| d.list_mode.clone()),
            view_style: host::setting_string("viewStyle").unwrap_or_else(|| d.view_style.clone()),
            columns: d.columns.clone(),
            column_order: d.column_order.clone(),
            column_preset: host::setting_string("columnPreset")
                .unwrap_or_else(|| d.column_preset.clone()),
            show_header_labels: host::setting_bool("showHeaderLabels")
                .unwrap_or(d.show_header_labels),
            show_range_bar: host::setting_bool("showRangeBar").unwrap_or(d.show_range_bar),
            interval_sec: host::setting_number("intervalSec")
                .unwrap_or(d.interval_sec as f64)
                .clamp(3.0, 300.0) as u32,
            sort_by: host::setting_string("sortBy").unwrap_or_else(|| d.sort_by.clone()),
            profit_display: host::setting_string("profitDisplay")
                .unwrap_or_else(|| d.profit_display.clone()),
            alert_percent: host::setting_number("alertPercent")
                .unwrap_or(d.alert_percent)
                .abs(),
            quiet_hours: host::setting_bool("quietHours").unwrap_or(d.quiet_hours),
            decimals: host::setting_number("decimals")
                .unwrap_or(d.decimals as f64)
                .clamp(0.0, 4.0) as u32,
        };
        if p.column_preset != "custom" {
            p.columns = preset_columns(&p.column_preset);
        }
        p
    }

    /// 面板发来的补丁：逐字段覆盖，类型不对的字段保持原值，避免整包反序列化失败
    /// 把用户刚才的改动全部丢掉。
    pub fn merge_patch(&mut self, patch: &Value) {
        let Some(fields) = patch.as_object() else {
            return;
        };
        macro_rules! apply {
            ($key:literal, $field:ident, $kind:ident) => {
                if let Some(v) = fields.get($key) {
                    apply!(@set $field, v, $kind);
                }
            };
            (@set $field:ident, $v:expr, str) => {
                if let Some(s) = $v.as_str() { self.$field = s.to_string(); }
            };
            (@set $field:ident, $v:expr, bool) => {
                if let Some(b) = $v.as_bool() { self.$field = b; }
            };
            (@set $field:ident, $v:expr, num) => {
                if let Some(n) = $v.as_f64() { self.$field = n as _; }
            };
        }
        apply!("colorScheme", color_scheme, str);
        apply!("stealth", stealth, bool);
        apply!("stealthHideNames", stealth_hide_names, bool);
        apply!("stealthMaskPrice", stealth_mask_price, bool);
        apply!("statusBarMode", status_bar_mode, str);
        apply!("density", density, str);
        apply!("fontSize", font_size, str);
        apply!("detail", detail, str);
        apply!("showIndices", show_indices, bool);
        apply!("listMode", list_mode, str);
        apply!("viewStyle", view_style, str);
        apply!("columnPreset", column_preset, str);
        apply!("showHeaderLabels", show_header_labels, bool);
        apply!("showRangeBar", show_range_bar, bool);
        apply!("intervalSec", interval_sec, num);
        apply!("sortBy", sort_by, str);
        apply!("profitDisplay", profit_display, str);
        apply!("alertPercent", alert_percent, num);
        apply!("quietHours", quiet_hours, bool);
        apply!("decimals", decimals, num);

        let preset_only = !fields.contains_key("columns")
            && fields
                .get("columnPreset")
                .and_then(Value::as_str)
                .is_some_and(|s| s != "custom");
        if let Some(list) = fields.get("columns").and_then(Value::as_array) {
            self.columns = list
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect();
        } else if preset_only {
            self.columns = preset_columns(&self.column_preset);
        }
        if let Some(list) = fields.get("columnOrder").and_then(Value::as_array) {
            self.column_order = list
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect();
        }
        self.normalize();
    }

    pub fn normalize(&mut self) {
        one_of(&mut self.color_scheme, &["cn", "us", "mono"], "cn");
        one_of(
            &mut self.status_bar_mode,
            &["index", "profit", "hidden"],
            "index",
        );
        one_of(&mut self.density, &["compact", "comfortable"], "compact");
        one_of(&mut self.font_size, &["small", "normal", "large"], "normal");
        one_of(&mut self.detail, &["minimal", "normal", "full"], "normal");
        one_of(&mut self.list_mode, &["tabs", "all", "positions"], "tabs");
        one_of(&mut self.view_style, &["card", "table"], "card");
        one_of(
            &mut self.column_preset,
            &["compact", "normal", "full", "custom"],
            "normal",
        );
        one_of(&mut self.sort_by, &["custom", "percent", "code"], "custom");
        one_of(
            &mut self.profit_display,
            &["amount", "percent", "hidden"],
            "amount",
        );
        self.columns.retain(|c| COLUMNS.contains(&c.as_str()));
        if self.columns.is_empty() && self.column_preset != "custom" {
            self.columns = preset_columns(&self.column_preset);
        }
        // 列顺序：去掉非法键与重复项，再把没提到的列按默认顺序补到末尾，
        // 这样即使以后新增列，老用户的顺序也能自动兼容。
        let mut order: Vec<String> = Vec::with_capacity(COLUMNS.len());
        for key in &self.column_order {
            if COLUMNS.contains(&key.as_str()) && !order.contains(key) {
                order.push(key.clone());
            }
        }
        for key in COLUMNS.iter() {
            if !order.iter().any(|o| o == key) {
                order.push((*key).to_string());
            }
        }
        self.column_order = order;
        self.interval_sec = self.interval_sec.clamp(3, 300);
        self.decimals = self.decimals.min(4);
        self.alert_percent = self.alert_percent.abs().min(100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn defaults_are_domestic_and_non_stealth() {
        let p = Prefs::default();
        assert_eq!(p.color_scheme, "cn");
        assert!(!p.stealth);
        assert_eq!(p.interval_sec, 10);
        assert_eq!(p.decimals, 2);
        assert_eq!(p.list_mode, "tabs");
        assert_eq!(p.view_style, "card");
        assert_eq!(p.columns, preset_columns("normal"));
        assert!(p.show_header_labels);
        assert_eq!(p.column_order, default_order());
        assert!(p.show_range_bar);
    }

    #[test]
    fn column_order_accepts_drag_result_and_repairs_itself() {
        let mut p = Prefs::default();
        p.merge_patch(&json!({ "columnOrder": ["profit", "high", "profit", "bogus"] }));
        // 非法键被丢弃、重复项去重，其余列按默认顺序补到末尾。
        assert_eq!(&p.column_order[..2], &["profit", "high"]);
        assert_eq!(p.column_order.len(), COLUMNS.len());
        assert!(p.column_order.iter().any(|k| k == "change"));
        // 拖动结果可以往返 storage。
        let back: Prefs = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_eq!(back.column_order, p.column_order);
    }

    #[test]
    fn range_bar_can_be_switched_off() {
        let mut p = Prefs::default();
        p.merge_patch(&json!({ "showRangeBar": false }));
        assert!(!p.show_range_bar);
        p.merge_patch(&json!({ "showRangeBar": "no" }));
        assert!(!p.show_range_bar); // 类型不对时保持原值
    }

    #[test]
    fn patch_only_touches_its_own_fields() {
        let mut p = Prefs::default();
        p.merge_patch(&json!({ "colorScheme": "us", "stealth": true }));
        assert_eq!(p.color_scheme, "us");
        assert!(p.stealth);
        assert_eq!(p.interval_sec, 10);
        assert_eq!(p.detail, "normal");
    }

    #[test]
    fn patch_round_trips_through_storage_json() {
        let mut p = Prefs::default();
        p.merge_patch(
            &json!({ "fontSize": "small", "sortBy": "percent", "decimals": 3, "listMode": "all" }),
        );
        let s = serde_json::to_string(&p).unwrap();
        let back: Prefs = serde_json::from_str(&s).unwrap();
        assert_eq!(back, p);
        assert_eq!(serde_json::to_value(&p).unwrap()["colorScheme"], "cn");
        assert_eq!(back.list_mode, "all");
    }

    #[test]
    fn a_badly_typed_field_no_longer_discards_the_whole_patch() {
        let mut p = Prefs::default();
        p.merge_patch(&json!({
            "colorScheme": "us",
            "stealth": true,
            "decimals": "2",
            "intervalSec": 20
        }));
        assert_eq!(p.color_scheme, "us");
        assert!(p.stealth);
        assert_eq!(p.interval_sec, 20);
        assert_eq!(p.decimals, 2);
    }

    #[test]
    fn column_presets_and_custom_columns() {
        let mut p = Prefs::default();
        p.merge_patch(&json!({ "columnPreset": "compact" }));
        assert_eq!(p.columns, vec!["change", "prevClose"]);

        p.merge_patch(&json!({ "columnPreset": "custom", "columns": ["high", "low", "unknown"] }));
        assert_eq!(p.columns, vec!["high", "low"]);

        p.merge_patch(&json!({ "columnPreset": "full" }));
        assert_eq!(
            p.columns,
            COLUMNS.iter().map(|s| s.to_string()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn empty_custom_columns_are_allowed() {
        let mut p = Prefs::default();
        p.merge_patch(&json!({ "columnPreset": "custom", "columns": [] }));
        assert!(p.columns.is_empty());
    }

    #[test]
    fn view_style_switches_between_card_and_table() {
        let mut p = Prefs::default();
        p.merge_patch(&json!({ "viewStyle": "table" }));
        assert_eq!(p.view_style, "table");
        p.merge_patch(&json!({ "viewStyle": "unknown" }));
        assert_eq!(p.view_style, "card");
    }

    #[test]
    fn normalize_rejects_bad_values() {
        let mut p = Prefs::default();
        p.merge_patch(&json!({
            "colorScheme": "rainbow",
            "listMode": "unknown",
            "columnPreset": "unknown",
            "intervalSec": 9999,
            "decimals": 99,
            "alertPercent": -12
        }));
        assert_eq!(p.color_scheme, "cn");
        assert_eq!(p.list_mode, "tabs");
        assert_eq!(p.column_preset, "normal");
        assert_eq!(p.interval_sec, 300);
        assert_eq!(p.decimals, 4);
        assert_eq!(p.alert_percent, 12.0);
    }
}
