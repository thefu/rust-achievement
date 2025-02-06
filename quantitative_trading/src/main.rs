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
    let api_key = "YOUR_API_KEY";
    let symbol = "AAPL";
    let url = format!(
        "https://www.alphavantage.co/query?function=TIME_SERIES_DAILY&symbol={}&apikey={}",
        symbol, api_key
    );

    let response = reqwest::get(&url).await?.json::<ApiResponse>().await?;

    // 初始化技术指标
    let mut ema = ExponentialMovingAverage::new(3).unwrap();
    let mut rsi = RelativeStrengthIndex::new(14).unwrap();

    // 处理时间序列数据
    for (date, data) in response.time_series {
        let close_price: f64 = data.close.parse().unwrap();
        let ema_value = ema.next(close_price);
        let rsi_value = rsi.next(close_price);

        println!(
            "Date: {}, Close Price: {}, EMA: {}, RSI: {}",
            date, close_price, ema_value, rsi_value
        );

        // 判断是否下单
        if should_place_order(ema_value, rsi_value) {
            println!("Order placed on {}", date);
        } else {
            println!("No order on {}", date);
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
