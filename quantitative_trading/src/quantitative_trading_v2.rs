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
    _short_window: usize,
    _long_window: usize,
}

// 交易信号枚举
#[derive(Debug, PartialEq)]
pub enum TradeSignal {
    Buy,
    Sell,
    Hold,
}

enum TradeSignalWithRisk {
    Buy {
        entry_price: f64,
        stop_loss: f64,
        take_profit: f64,
        quantity: f64,
    },
    Sell {
        entry_price: f64,
        stop_loss: f64,
        take_profit: f64,
        quantity: f64,
    },
    Hold,
}

struct RiskManager {
    total_capital: f64,
    risk_per_trade: f64,
    take_profit_pct: f64,
    atr_period: usize,
}

impl RiskManager {
    fn new(total_capital: f64) -> Self {
        RiskManager {
            total_capital,
            risk_per_trade: 0.01,
            take_profit_pct: 0.03,
            atr_period: 14,
        }
    }

    fn calculate_position_size(&self, entry_price: f64, atr: f64) -> f64 {
        let risk_amount = self.total_capital * self.risk_per_trade;
        let units = risk_amount / (atr * entry_price);
        // Round down to the nearest whole number of units
        units.floor()
    }

    fn dynamic_stop_loss(&self, entry_price: f64, atr: f64, is_long: bool) -> f64 {
        if is_long {
            entry_price - 2.0 * atr
        } else {
            entry_price + 2.0 * atr
        }
    }
}

#[tokio::main]
// 异步主函数，返回一个Result类型，其中Ok为空元组，Err为Box<dyn Error>动态错误类型
async fn main() -> Result<(), Box<dyn Error>> {
    // 创建一个策略配置实例，包含API密钥、股票符号、短期窗口和长期窗口
    let config = StrategyConfig {
        api_key: "XTUOEZ3P3FCS956P".to_string(), // API密钥，用于访问市场数据
        symbol: "MSFT".to_string(),              // 股票符号，这里为微软公司
        _short_window: 20,                       // 短期窗口大小，用于计算短期均线
        _long_window: 50,                        // 长期窗口大小，用于计算长期均线
    };

    let risk_manager = RiskManager::new(100000.0);

    // 获取市场数据，使用await等待异步操作完成，?操作符用于错误处理
    let price_data = fetch_market_data_v2(&config).await?;

    let atr = calculate_atr(&price_data, risk_manager.atr_period);

    // 生成交易信号，传入价格数据、短期窗口和长期窗口
    let signal = execute_trading_strategy(&price_data);
    let signal_with_risk_manager =
        calulate_signal_with_risk_manager(&signal, &risk_manager, atr, &price_data);

    match signal_with_risk_manager {
        TradeSignalWithRisk::Buy {
            entry_price,
            stop_loss,
            take_profit,
            quantity,
        } => {
            println!(
                "🟢 BUY: Price={:.2} Qty={} SL={:.2} TP={:.2}",
                entry_price, quantity, stop_loss, take_profit
            );
        }
        TradeSignalWithRisk::Sell {
            entry_price,
            stop_loss,
            take_profit,
            quantity,
        } => {
            println!(
                "🔴 SELL: Price={:.2} Qty={} SL={:.2} TP={:.2}",
                entry_price, quantity, stop_loss, take_profit
            );
        }
        TradeSignalWithRisk::Hold => println!("🟡 HOLD"),
    }

    // 执行交易逻辑
    // match signal {
    //     TradeSignal::Buy => {
    //         let entry_price = ohlc_data.last().unwrap().close;
    //         let current_atr = atr.last().unwrap_or(&0.0);

    //         let stop_loss = risk_manager.dynamic_stop_loss(entry_price, *current_atr, true);
    //         let quantity = risk_manager.calculate_position_size(entry_price, *current_atr);

    //         let take_profit = entry_price * (1.0 + risk_manager.take_profit_pct);

    //         return TradeSignal::Buy {
    //             entry_price,
    //             stop_loss,
    //             take_profit,
    //             quantity,
    //         };
    //     }
    //     TradeSignal::Sell => println!("🔴 SELL SIGNAL"),
    //     TradeSignal::Hold => println!("🟡 HOLD"),
    // }

    Ok(())
}

fn calculate_atr(price_data: &PriceData, period: usize) -> Vec<f64> {
    let mut atr_values = Vec::new();
    let mut true_ranges = Vec::new();

    for i in 1..price_data.prices.len() {
        let tr1 = price_data.highs[i] - price_data.lows[i];
        let tr2 = (price_data.highs[i] - price_data.closes[i - 1]).abs();
        let tr3 = (price_data.lows[i] - price_data.closes[i - 1]).abs();

        true_ranges.push(tr1.max(tr2).max(tr3));
    }

    let mut atr = SimpleMovingAverage::new(period).unwrap();
    for tr in true_ranges {
        atr_values.push(atr.next(tr));
    }

    atr_values
}

fn calulate_signal_with_risk_manager(
    signal: &TradeSignal,
    risk_manager: &RiskManager,
    atr: Vec<f64>,
    price_data: &PriceData,
) -> TradeSignalWithRisk {
    match signal {
        TradeSignal::Buy => {
            let entry_price = price_data.closes.last().unwrap();
            let current_atr = atr.last().unwrap_or(&0.0);

            let stop_loss = risk_manager.dynamic_stop_loss(*entry_price, *current_atr, true);
            let quantity = risk_manager.calculate_position_size(*entry_price, *current_atr);

            let take_profit = entry_price * (1.0 + risk_manager.take_profit_pct);

            return TradeSignalWithRisk::Buy {
                entry_price: *entry_price,
                stop_loss,
                take_profit,
                quantity,
            };
        }
        TradeSignal::Sell => {
            let entry_price = price_data.closes.last().unwrap();
            let current_atr = atr.last().unwrap_or(&0.0);

            let stop_loss = risk_manager.dynamic_stop_loss(*entry_price, *current_atr, false);
            let quantity = risk_manager.calculate_position_size(*entry_price, *current_atr);
            let take_profit = entry_price * (1.0 - risk_manager.take_profit_pct);

            return TradeSignalWithRisk::Sell {
                entry_price: *entry_price,
                stop_loss,
                take_profit,
                quantity,
            };
        }
        TradeSignal::Hold => {
            return TradeSignalWithRisk::Hold;
        }
    }
}

// 定义一个异步函数fetch_market_data，用于获取市场数据
// 参数config是一个StrategyConfig的引用，返回一个Result类型，其中包含一个f64类型的向量或者一个动态错误
async fn _fetch_market_data(config: &StrategyConfig) -> Result<Vec<f64>, Box<dyn Error>> {
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
fn _generate_signal(prices: &[f64], short_window: usize, long_window: usize) -> TradeSignal {
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
        let result = _generate_signal(&prices, short_window, long_window);
        assert_eq!(result, TradeSignal::Hold);
    }

    #[test]
    fn test_generate_signal_short_window_less_than_long_window() {
        let prices = vec![10.0, 20.0, 15.0, 30.0, 25.0];
        let short_window = 2;
        let long_window = 3;
        let result = _generate_signal(&prices, short_window, long_window);
        assert_eq!(result, TradeSignal::Hold);
    }

    #[test]
    fn test_generate_signal_golden_cross() {
        let prices = vec![10.0, 20.0, 15.0, 30.0, 25.0];
        let short_window = 2;
        let long_window = 3;
        let result = _generate_signal(&prices, short_window, long_window);
        assert_eq!(result, TradeSignal::Buy);
    }

    #[test]
    fn test_generate_signal_death_cross() {
        let prices = vec![30.0, 20.0, 15.0, 10.0, 25.0];
        let short_window = 3;
        let long_window = 2;
        let result = _generate_signal(&prices, short_window, long_window);
        assert_eq!(result, TradeSignal::Sell);
    }

    #[test]
    fn test_generate_signal_no_cross() {
        let prices = vec![10.0, 20.0, 15.0, 20.0, 25.0];
        let short_window = 3;
        let long_window = 2;
        let result = _generate_signal(&prices, short_window, long_window);
        assert_eq!(result, TradeSignal::Hold);
    }
}
