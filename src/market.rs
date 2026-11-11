//! 行情解析与本地计算。这里不碰宿主 API，可在 native 下直接单元测试。

use encoding_rs::GBK;
use serde::{Deserialize, Serialize};

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

/// 腾讯行情代码：`1.600519` → `sh600519`，`0.000001` → `sz000001`。
pub fn tencent_code(secid: &str) -> Option<String> {
    let (market, code) = secid.split_once('.')?;
    let prefix = match market {
        "1" => "sh",
        "0" => "sz",
        "2" => "bj",
        _ => return None,
    };
    Some(format!("{prefix}{code}"))
}

/// 公历日期距 1970-01-01 的天数。
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// 接口给的是北京时间的 `yyyyMMddHHmmss`，换算成 UTC 秒，与原先时间戳口径一致。
fn beijing_time(raw: &str) -> f64 {
    if raw.len() != 14 || !raw.bytes().all(|b| b.is_ascii_digit()) {
        return 0.0;
    }
    let at = |a: usize, b: usize| raw[a..b].parse::<i64>().unwrap_or(0);
    let (y, mo, d) = (at(0, 4), at(4, 6), at(6, 8));
    let (h, mi, se) = (at(8, 10), at(10, 12), at(12, 14));
    if !(1990..=2200).contains(&y) || !(1..=12).contains(&mo) || !(1..=31).contains(&d) {
        return 0.0;
    }
    (days_from_civil(y, mo, d) * 86_400 + h * 3_600 + mi * 60 + se - 8 * 3_600) as f64
}

fn field(fields: &[&str], at: usize) -> f64 {
    fields
        .get(at)
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0.0)
}

/// 解析 `qt.gtimg.cn/q=` 的响应：GBK 编码，一只一行 `v_sh600519="…";`，字段以 `~` 分隔。
pub fn parse_quotes(body: &[u8]) -> Result<Vec<Quote>, String> {
    let (text, _, _) = GBK.decode(body);
    let mut quotes = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.trim().strip_prefix("v_") else {
            continue;
        };
        let Some((key, payload)) = rest.split_once('=') else {
            continue;
        };
        let payload = payload.trim().trim_end_matches(';').trim_matches('"');
        let fields: Vec<&str> = payload.split('~').collect();
        // 停牌或代码无效时接口回空串，字段数也不够。
        if fields.len() < 35 || fields[1].is_empty() {
            continue;
        }
        let secid = if let Some(code) = key.strip_prefix("sh") {
            format!("1.{code}")
        } else if let Some(code) = key.strip_prefix("sz") {
            format!("0.{code}")
        } else if let Some(code) = key.strip_prefix("bj") {
            format!("2.{code}")
        } else {
            continue;
        };
        quotes.push(Quote {
            secid,
            code: fields[2].to_string(),
            name: fields[1].to_string(),
            price: field(&fields, 3),
            change: field(&fields, 31),
            percent: field(&fields, 32),
            open: field(&fields, 5),
            high: field(&fields, 33),
            low: field(&fields, 34),
            prev_close: field(&fields, 4),
            volume: field(&fields, 6),
            // 第 37 项是成交额，单位万元。
            amount: field(&fields, 37) * 10_000.0,
            timestamp: beijing_time(fields[30]),
        });
    }
    if quotes.is_empty() {
        return Err("行情响应里没有可用的数据".to_string());
    }
    Ok(quotes)
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

    /// 腾讯接口是 GBK，测试里按同样的编码构造，顺便验证解码这一步。
    fn sample() -> Vec<u8> {
        let text = concat!(
            "v_sh600519=\"1~贵州茅台~600519~1424.37~1439.00~1445.00~10387~4824~5563~1423.79~3~",
            "1422.72~1~1422.55~2~1422.54~1~1422.53~2~1424.34~3~1424.35~3~1424.36~4~1424.37~2~",
            "1424.42~1~~20261111100809~-14.63~-1.02~1445.62~1417.55~1424.37/10387/1482548364~",
            "10387~148255~0.08~21.67~~1445.62~1417.55~1.95~17805.79~17805.79~6.57~1582.90~1295.10\";",
            "\n",
            "v_sz000001=\"51~平安银行~000001~11.83~11.86~11.83~191889~78229~113582~11.82~938~",
            "11.81~1306~11.80~1957~11.79~4622~11.78~7310~11.83~390~11.84~4407~11.85~4251~",
            "11.86~5462~11.87~3669~~20261111100809~-0.03~-0.25~11.86~11.78~11.83/191889/226800488~",
            "191889~22680~0.10~5.28~~11.86~11.78~0.67~2295.69~2295.72~0.48~13.05~10.67~0.94~-2046\";",
            "\n",
        );
        GBK.encode(text).0.into_owned()
    }

    #[test]
    fn parses_tencent_payload() {
        let quotes = parse_quotes(&sample()).unwrap();
        assert_eq!(quotes.len(), 2);
        let q = &quotes[0];
        assert_eq!(q.secid, "1.600519");
        assert_eq!(q.code, "600519");
        assert_eq!(q.name, "贵州茅台");
        assert!((q.price - 1424.37).abs() < 1e-6);
        assert!((q.change - -14.63).abs() < 1e-6);
        assert!((q.percent - -1.02).abs() < 1e-6);
        assert!((q.prev_close - 1439.00).abs() < 1e-6);
        assert!((q.open - 1445.00).abs() < 1e-6);
        assert!((q.high - 1445.62).abs() < 1e-6);
        assert!((q.low - 1417.55).abs() < 1e-6);
        assert_eq!(q.volume, 10387.0);
        assert!((q.amount - 1_482_550_000.0).abs() < 1.0);
        assert!(q.timestamp > 1_700_000_000.0);
        assert_eq!(quotes[1].secid, "0.000001");
        assert_eq!(quotes[1].name, "平安银行");
    }

    #[test]
    fn tolerates_suspended_stock_fields() {
        let body = GBK.encode("v_sh600001=\"\";\n").0.into_owned();
        // 停牌拿到空串时，整批视为无有效数据，而不是 panic。
        assert!(parse_quotes(&body).is_err());
    }

    #[test]
    fn rejects_broken_payload() {
        assert!(parse_quotes(b"").is_err());
        assert!(parse_quotes(b"<html>502</html>").is_err());
    }

    #[test]
    fn maps_tencent_codes() {
        assert_eq!(tencent_code("1.600519").as_deref(), Some("sh600519"));
        assert_eq!(tencent_code("0.000001").as_deref(), Some("sz000001"));
        assert_eq!(tencent_code("2.830799").as_deref(), Some("bj830799"));
        assert_eq!(tencent_code("9.1"), None);
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
