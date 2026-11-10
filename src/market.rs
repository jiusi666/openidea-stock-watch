//! 行情解析与本地计算。这里不碰宿主 API，可在 native 下直接单元测试。

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 东方财富 A 股报价按「分」返回（价格、涨跌额、涨跌幅均为实际值 ×100）。
pub const PRICE_SCALE: f64 = 100.0;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Item {
    pub secid: String,
    pub code: String,
    #[serde(default)]
    pub cost: f64,
    #[serde(default)]
    pub qty: f64,
}

impl Item {
    pub fn new(secid: &str) -> Self {
        let code = secid
            .split_once('.')
            .map(|(_, c)| c)
            .unwrap_or(secid)
            .to_string();
        Self {
            secid: secid.to_string(),
            code,
            cost: 0.0,
            qty: 0.0,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Quote {
    pub secid: String,
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub percent: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub prev_close: f64,
    pub volume: f64,
    pub amount: f64,
    pub timestamp: f64,
}

fn number(value: Option<&Value>) -> f64 {
    match value {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0),
        // 停牌或字段缺失时接口会返回字符串 "-"
        Some(Value::String(s)) => s.parse().unwrap_or(0.0),
        _ => 0.0,
    }
}

fn string(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

/// 解析 `ulist.np/get` 的 `data.diff` 数组。
pub fn parse_quotes(body: &[u8]) -> Result<Vec<Quote>, String> {
    let root: Value = serde_json::from_slice(body).map_err(|e| format!("行情解析失败：{e}"))?;
    let rows = root
        .pointer("/data/diff")
        .and_then(Value::as_array)
        .ok_or_else(|| "行情响应缺少 data.diff".to_string())?;

    Ok(rows
        .iter()
        .map(|d| {
            let market = number(d.get("f13")).round() as i64;
            let code = string(d.get("f12"));
            Quote {
                secid: format!("{market}.{code}"),
                code,
                name: string(d.get("f14")),
                price: number(d.get("f2")) / PRICE_SCALE,
                change: number(d.get("f4")) / PRICE_SCALE,
                percent: number(d.get("f3")) / PRICE_SCALE,
                open: number(d.get("f17")) / PRICE_SCALE,
                high: number(d.get("f15")) / PRICE_SCALE,
                low: number(d.get("f16")) / PRICE_SCALE,
                prev_close: number(d.get("f18")) / PRICE_SCALE,
                volume: number(d.get("f5")),
                amount: number(d.get("f6")),
                timestamp: number(d.get("f124")),
            }
        })
        .collect())
}

fn is_code(code: &str) -> bool {
    code.len() == 6 && code.bytes().all(|b| b.is_ascii_digit())
}

/// 把用户输入归一为东财 secid：`600519` → `1.600519`，`sz000001` → `0.000001`。
/// 纯 6 位数字按股票习惯推断：6/9 开头归沪市，其余归深市；指数请用 `1.000001` 形式。
pub fn normalize(input: &str) -> Option<String> {
    let s = input.trim().to_ascii_lowercase().replace(' ', "");
    if s.is_empty() {
        return None;
    }
    if let Some((market, code)) = s.split_once('.') {
        return if (market == "0" || market == "1") && is_code(code) {
            Some(format!("{market}.{code}"))
        } else {
            None
        };
    }
    let (market, code) = if let Some(c) = s.strip_prefix("sh") {
        ("1", c)
    } else if let Some(c) = s.strip_prefix("sz") {
        ("0", c)
    } else {
        ("", s.as_str())
    };
    if !is_code(code) {
        return None;
    }
    let market = if market.is_empty() {
        if code.starts_with('6') || code.starts_with('9') {
            "1"
        } else {
            "0"
        }
    } else {
        market
    };
    Some(format!("{market}.{code}"))
}

/// 浮动盈亏：只有填了成本价和数量才算。
pub fn profit(item: &Item, quote: &Quote) -> Option<f64> {
    if item.qty <= 0.0 || item.cost <= 0.0 || quote.price <= 0.0 {
        return None;
    }
    Some((quote.price - item.cost) * item.qty)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> &'static [u8] {
        r#"{"rc":0,"data":{"total":2,"diff":[
        {"f2":143649,"f3":-59,"f4":-850,"f5":3829,"f6":550730919.0,
         "f12":"600519","f13":1,"f14":"贵州茅台","f15":144300,"f16":143126,
         "f17":143823,"f18":144499,"f124":1794274985},
        {"f2":1181,"f3":-67,"f4":-8,"f5":136640,"f6":161104909.54,
         "f12":"000001","f13":0,"f14":"平安银行","f15":1184,"f16":1173,
         "f17":1183,"f18":1189,"f124":1794274986}]}}"#
            .as_bytes()
    }

    #[test]
    fn parses_eastmoney_payload() {
        let quotes = parse_quotes(sample()).unwrap();
        assert_eq!(quotes.len(), 2);
        let q = &quotes[0];
        assert_eq!(q.secid, "1.600519");
        assert_eq!(q.name, "贵州茅台");
        assert!((q.price - 1436.49).abs() < 1e-6);
        assert!((q.change - -8.50).abs() < 1e-6);
        assert!((q.percent - -0.59).abs() < 1e-6);
        assert!((q.prev_close - 1444.99).abs() < 1e-6);
        assert_eq!(quotes[1].secid, "0.000001");
    }

    #[test]
    fn tolerates_suspended_stock_fields() {
        let body = r#"{"data":{"diff":[{"f2":"-","f3":"-","f12":"600001","f13":1,"f14":"停牌"}]}}"#
            .as_bytes();
        let quotes = parse_quotes(body).unwrap();
        assert_eq!(quotes[0].price, 0.0);
        assert_eq!(quotes[0].secid, "1.600001");
    }

    #[test]
    fn rejects_broken_payload() {
        assert!(parse_quotes(b"{}").is_err());
        assert!(parse_quotes(b"not json").is_err());
    }

    #[test]
    fn normalizes_codes() {
        assert_eq!(normalize("600519").as_deref(), Some("1.600519"));
        assert_eq!(normalize(" 000001 ").as_deref(), Some("0.000001"));
        assert_eq!(normalize("sh600519").as_deref(), Some("1.600519"));
        assert_eq!(normalize("SZ300750").as_deref(), Some("0.300750"));
        assert_eq!(normalize("1.000001").as_deref(), Some("1.000001"));
        assert_eq!(normalize("688981").as_deref(), Some("1.688981"));
        assert_eq!(normalize("abc"), None);
        assert_eq!(normalize("2.600519"), None);
    }

    #[test]
    fn computes_profit() {
        let mut item = Item::new("1.600519");
        let quote = Quote {
            price: 1500.0,
            ..Quote::default()
        };
        assert_eq!(profit(&item, &quote), None);
        item.cost = 1400.0;
        item.qty = 100.0;
        assert_eq!(profit(&item, &quote), Some(10_000.0));
    }

    #[test]
    fn items_round_trip_through_storage() {
        let items = vec![
            Item::new("1.600519"),
            Item {
                code: "000001".into(),
                secid: "0.000001".into(),
                cost: 11.5,
                qty: 1000.0,
            },
        ];
        let json = serde_json::to_string(&items).unwrap();
        let back: Vec<Item> = serde_json::from_str(&json).unwrap();
        assert_eq!(back, items);
    }
}
