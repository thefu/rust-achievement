use reqwest;
use serde::Deserialize;
use serde_json::Value;
use signal_aggregator::{execute_trading_strategy, PriceData};
use std::error::Error;
use ta::indicators::SimpleMovingAverage;
use ta::Next;

pub mod signal_aggregator;

// Alpha Vantage数据结构
#[derive(Debug, Deserialize)]
struct AlphaVantageResponse {
    #[serde(rename = "Time Series (5min)")]
    time_series: Option<Value>,
}

// 策略配置
struct StrategyConfig {
    api_key: String,
    symbol: String,
    short_window: usize,
    long_window: usize,
}

// 交易信号枚举
#[derive(Debug, PartialEq)]
pub enum TradeSignal {
    Buy,
    Sell,
    Hold,
}

#[tokio::main]
// 异步主函数，返回一个Result类型，其中Ok为空元组，Err为Box<dyn Error>动态错误类型
async fn main() -> Result<(), Box<dyn Error>> {
    // 创建一个策略配置实例，包含API密钥、股票符号、短期窗口和长期窗口
    let config = StrategyConfig {
        api_key: "XTUOEZ3P3FCS956P".to_string(), // API密钥，用于访问市场数据
        symbol: "MSFT".to_string(),              // 股票符号，这里为微软公司
        short_window: 20,                        // 短期窗口大小，用于计算短期均线
        long_window: 50,                         // 长期窗口大小，用于计算长期均线
    };

    // 获取市场数据，使用await等待异步操作完成，?操作符用于错误处理
    let price_data = fetch_market_data_v2(&config).await?;

    // 生成交易信号，传入价格数据、短期窗口和长期窗口
    let signal = execute_trading_strategy(&price_data);

    // 执行交易逻辑
    match signal {
        TradeSignal::Buy => println!("🟢 BUY SIGNAL"),
        TradeSignal::Sell => println!("🔴 SELL SIGNAL"),
        TradeSignal::Hold => println!("🟡 HOLD"),
    }

    Ok(())
}

// 定义一个异步函数fetch_market_data，用于获取市场数据
// 参数config是一个StrategyConfig的引用，返回一个Result类型，其中包含一个f64类型的向量或者一个动态错误
async fn fetch_market_data(config: &StrategyConfig) -> Result<Vec<f64>, Box<dyn Error>> {
    // 构建请求URL，使用format!宏插入symbol和api_key
    let url = format!(
        "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY&symbol={}&interval=5min&apikey={}",
        config.symbol, config.api_key
    );

    // 发送HTTP GET请求，并等待响应
    // 使用?操作符处理可能的错误
    // 将响应解析为AlphaVantageResponse类型的JSON
    let response = reqwest::get(&url)
        .await?
        .json::<AlphaVantageResponse>()
        .await?;

    // 初始化一个空的f64类型的向量，用于存储收盘价
    let mut closes = Vec::new();
    // 检查响应中是否包含时间序列数据
    if let Some(time_series) = response.time_series {
        // 遍历时间序列数据
        for (_, v) in time_series.as_object().unwrap() {
            // 从时间序列数据中提取收盘价，并解析为f64类型
            let close = v["4. close"].as_str().unwrap().parse::<f64>()?;
            // 将收盘价添加到向量中
            closes.push(close);
        }
    }

    closes.reverse(); // 确保数据按时间升序排列
    Ok(closes)
}

// 修改后的fetch_market_data函数，返回PriceData结构体
async fn fetch_market_data_v2(config: &StrategyConfig) -> Result<PriceData, Box<dyn Error>> {
    // 构建请求URL，使用format!宏插入symbol和api_key
    let url = format!(
        "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY&symbol={}&interval=5min&apikey={}",
        config.symbol, config.api_key
    );

    // 发送HTTP GET请求，并等待响应，然后将响应解析为AlphaVantageResponse类型的JSON
    let response = reqwest::get(&url)
        .await?
        .json::<AlphaVantageResponse>()
        .await?;

    // 初始化存储价格相关数据的向量
    let mut prices = Vec::new();
    let mut highs = Vec::new();
    let mut lows = Vec::new();
    let mut closes = Vec::new();

    // 检查响应中是否包含时间序列数据
    if let Some(time_series) = response.time_series {
        // 遍历时间序列数据
        for (_, v) in time_series.as_object().unwrap() {
            // 从每个数据点中提取开盘价、最高价、最低价和收盘价，并解析为f64
            let open = v["1. open"].as_str().unwrap().parse::<f64>()?;
            let high = v["2. high"].as_str().unwrap().parse::<f64>()?;
            let low = v["3. low"].as_str().unwrap().parse::<f64>()?;
            let close = v["4. close"].as_str().unwrap().parse::<f64>()?;

            prices.push(open);
            highs.push(high);
            lows.push(low);
            closes.push(close);
        }
    }

    // 确保数据按时间升序排列（API可能返回降序数据）
    prices.reverse();
    highs.reverse();
    lows.reverse();
    closes.reverse();

    // 将采集到的数据封装到PriceData结构体中返回
    Ok(PriceData {
        prices,
        highs,
        lows,
        closes,
    })
}

// 定义一个函数，用于根据价格数据生成交易信号
fn generate_signal(prices: &[f64], short_window: usize, long_window: usize) -> TradeSignal {
    // 创建短期简单移动平均线（SMA）实例
    let mut short_sma = SimpleMovingAverage::new(short_window).unwrap();
    // 创建长期简单移动平均线（SMA）实例
    let mut long_sma = SimpleMovingAverage::new(long_window).unwrap();

    // 初始化存储短期SMA值的向量
    let mut short_values = Vec::new();
    // 初始化存储长期SMA值的向量
    let mut long_values = Vec::new();

    println!("Prices: {:?}", prices);

    // 遍历价格数据
    for price in prices {
        // 计算当前价格对应的短期SMA值并添加到短期SMA值向量
        short_values.push(short_sma.next(*price));
        // 计算当前价格对应的长期SMA值并添加到长期SMA值向量
        long_values.push(long_sma.next(*price));
    }

    println!("Short SMA: {:?}", short_values);
    println!("Long SMA: {:?}", long_values);

    // 需要足够的数据点生成信号
    if short_values.len() < 2 || long_values.len() < 2 {
        return TradeSignal::Hold;
    }

    let last_short = short_values.last().unwrap();
    let prev_short = short_values[short_values.len() - 2];
    let last_long = long_values.last().unwrap();
    let prev_long = long_values[long_values.len() - 2];

    // 金叉：短期均线上穿长期均线
    if prev_short < prev_long && last_short >= last_long {
        TradeSignal::Buy
    }
    // 死叉：短期均线下穿长期均线
    else if prev_short > prev_long && last_short <= last_long {
        TradeSignal::Sell
    } else {
        TradeSignal::Hold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_signal_short_window_greater_than_long_window() {
        let prices = vec![10.0, 20.0, 15.0, 30.0, 25.0];
        let short_window = 3;
        let long_window = 2;
        let result = generate_signal(&prices, short_window, long_window);
        assert_eq!(result, TradeSignal::Hold);
    }

    #[test]
    fn test_generate_signal_short_window_less_than_long_window() {
        let prices = vec![10.0, 20.0, 15.0, 30.0, 25.0];
        let short_window = 2;
        let long_window = 3;
        let result = generate_signal(&prices, short_window, long_window);
        assert_eq!(result, TradeSignal::Hold);
    }

    #[test]
    fn test_generate_signal_golden_cross() {
        let prices = vec![10.0, 20.0, 15.0, 30.0, 25.0];
        let short_window = 2;
        let long_window = 3;
        let result = generate_signal(&prices, short_window, long_window);
        assert_eq!(result, TradeSignal::Buy);
    }

    #[test]
    fn test_generate_signal_death_cross() {
        let prices = vec![30.0, 20.0, 15.0, 10.0, 25.0];
        let short_window = 3;
        let long_window = 2;
        let result = generate_signal(&prices, short_window, long_window);
        assert_eq!(result, TradeSignal::Sell);
    }

    #[test]
    fn test_generate_signal_no_cross() {
        let prices = vec![10.0, 20.0, 15.0, 20.0, 25.0];
        let short_window = 3;
        let long_window = 2;
        let result = generate_signal(&prices, short_window, long_window);
        assert_eq!(result, TradeSignal::Hold);
    }
}
