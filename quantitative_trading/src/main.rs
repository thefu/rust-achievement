use reqwest::Error;
use serde::Deserialize;
use std::collections::HashMap;
use ta::indicators::{ExponentialMovingAverage, RelativeStrengthIndex};
use ta::Next;

// 数据结构定义
#[derive(Debug, Deserialize)]
struct TimeSeriesData {
    #[serde(rename = "1. open")]
    _open: String,
    #[serde(rename = "2. high")]
    _high: String,
    #[serde(rename = "3. low")]
    _low: String,
    #[serde(rename = "4. close")]
    close: String,
    #[serde(rename = "5. volume")]
    _volume: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(rename = "Time Series (Daily)")]
    time_series: HashMap<String, TimeSeriesData>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 获取股票数据
    let api_key = "XTUOEZ3P3FCS956P"; // API密钥，用于访问股票数据API
    let symbol = "600016"; // 股票代码
    let url = format!(
        "https://www.alphavantage.co/query?function=TIME_SERIES_DAILY&symbol={}&apikey={}",
        symbol, api_key
    ); // 构建API请求URL

    let response = reqwest::get(&url).await?.json::<ApiResponse>().await?; // 发送HTTP请求并解析JSON响应

    // 初始化技术指标
    let mut ema = ExponentialMovingAverage::new(3).unwrap(); // 初始化指数移动平均线（EMA），周期为3
                                                             // EMA（Exponential Moving Average）是指数移动平均线，用于平滑价格数据，赋予最近的数据点更大的权重
    let mut rsi = RelativeStrengthIndex::new(14).unwrap(); // 初始化相对强弱指数（RSI），周期为14
                                                           // RSI（Relative Strength Index）是相对强弱指数，用于衡量价格的超买或超卖状态

    // 处理时间序列数据
    for (date, data) in response.time_series {
        let close_price: f64 = data.close.parse().unwrap(); // 将收盘价字符串解析为浮点数
        let ema_value = ema.next(close_price); // 计算EMA值
        let rsi_value = rsi.next(close_price); // 计算RSI值

        println!(
            "Date: {}, Close Price: {}, EMA: {}, RSI: {}",
            date, close_price, ema_value, rsi_value
        ); // 打印日期、收盘价、EMA值和RSI值

        // 判断是否下单
        if should_place_order(ema_value, rsi_value) {
            println!("Order placed on {}", date); // 如果满足下单条件，打印下单信息
        } else {
            println!("No order on {}", date); // 如果不满足下单条件，打印不下单信息
        }
    }

    Ok(())
}

// 下单逻辑
fn should_place_order(ema_value: f64, rsi_value: f64) -> bool {
    let ema_threshold = 150.0; // 设定的EMA阈值
    let rsi_threshold = 70.0; // 设定的RSI阈值

    if ema_value > ema_threshold && rsi_value > rsi_threshold {
        true
    } else {
        false
    }
}
