use std::collections::HashMap;

use crate::TradeSignal;

#[derive(Debug, Clone)]
pub struct SignalStrength {
    pub buy_strength: f64,
    pub sell_strength: f64,
}

pub struct SignalAggregator {
    indicators: HashMap<String, f64>, // 指标权重
    threshold: f64,                   // 信号阈值
}

pub struct PriceData {
    pub prices: Vec<f64>, // 价格数据
    pub highs: Vec<f64>,  // 最高价数据
    pub lows: Vec<f64>,   //  最低价数据
    pub closes: Vec<f64>, //    收盘价数据
}

impl SignalAggregator {
    pub fn new(threshold: f64) -> Self {
        let mut indicators = HashMap::new();
        indicators.insert("MACD".to_string(), 0.3);
        indicators.insert("RSI".to_string(), 0.2);
        indicators.insert("BB".to_string(), 0.2);
        indicators.insert("KDJ".to_string(), 0.15);
        indicators.insert("MA_CROSS".to_string(), 0.15);

        Self {
            indicators,
            threshold,
        }
    }

    pub fn generate_composite_signal(
        &self,
        signals: &HashMap<String, SignalStrength>,
    ) -> TradeSignal {
        let mut total_buy = 0.0;
        let mut total_sell = 0.0;

        for (indicator, weight) in &self.indicators {
            if let Some(signal) = signals.get(indicator) {
                total_buy += signal.buy_strength * weight;
                total_sell += signal.sell_strength * weight;
            }
        }

        if total_buy > self.threshold {
            TradeSignal::Buy
        } else if total_sell > self.threshold {
            TradeSignal::Sell
        } else {
            TradeSignal::Hold
        }
    }
}

// 交易信号生成器
pub fn generate_trading_signals(price_data: &PriceData) -> HashMap<String, SignalStrength> {
    let mut signals = HashMap::new();

    // MACD信号
    let macd = calculate_macd_signal(price_data);
    signals.insert("MACD".to_string(), macd);

    // RSI信号
    let rsi = calculate_rsi_signal(price_data);
    signals.insert("RSI".to_string(), rsi);

    // 布林带信号
    let bb = calculate_bollinger_signal(price_data);
    signals.insert("BB".to_string(), bb);

    // KDJ信号
    let kdj = calculate_kdj_signal(price_data);
    signals.insert("KDJ".to_string(), kdj);

    // MA交叉信号
    let ma_cross = calculate_ma_cross_signal(price_data);
    signals.insert("MA_CROSS".to_string(), ma_cross);

    signals
}

// MACD信号计算
// 定义一个函数，用于计算MACD信号强度
// MACD（移动平均收敛散度）的定义和作用
// MACD（Moving Average Convergence Divergence）是一种趋势跟踪指标，用于分析价格的变化趋势和动量，它由两条移动平均线及其差值（MACD线）和一条信号线
// 组成。MACD主要用于识别买入和卖出信号，当MACD线上穿信号线时，产生买入信号；当MACD线下穿信号线时，产生卖出信号。
// 定义一个函数，用于计算MACD信号强度
fn calculate_macd_signal(price_data: &PriceData) -> SignalStrength {
    // 如果价格数据中的价格数量少于26个，则返回一个买入和卖出强度都为0的信号强度
    if price_data.prices.len() < 26 {
        return SignalStrength {
            buy_strength: 0.0,
            sell_strength: 0.0,
        };
    }

    // 定义短期EMA的窗口大小为12
    let short_window = 12;
    // 定义长期EMA的窗口大小为26
    let long_window = 26;
    // 定义信号线的窗口大小为9
    let signal_window = 9;

    // 计算短期EMA
    let short_ema = calculate_ema(&price_data.prices, short_window);
    // 计算长期EMA
    let long_ema = calculate_ema(&price_data.prices, long_window);
    // 计算MACD线，即短期EMA减去长期EMA
    let macd_line = short_ema - long_ema;
    // 计算信号线，即最近signal_window个价格数据的EMA
    let signal_line = calculate_ema(
        &price_data.prices[price_data.prices.len() - signal_window..],
        signal_window,
    );
    // 计算MACD直方图，即MACD线减去信号线
    let macd_histogram = macd_line - signal_line;

    // 如果MACD直方图大于0，则返回买入强度为MACD直方图值，卖出强度为0的信号强度
    if macd_histogram > 0.0 {
        SignalStrength {
            buy_strength: macd_histogram,
            sell_strength: 0.0,
        }
    } else {
        // 否则，返回买入强度为0，卖出强度为MACD直方图绝对值的信号强度
        SignalStrength {
            buy_strength: 0.0,
            sell_strength: macd_histogram.abs(),
        }
    }
}

// 定义一个函数 calculate_ema，用于计算指数移动平均线（EMA）
fn calculate_ema(prices: &[f64], window: usize) -> f64 {
    // 计算平滑因子，公式为 2 / (窗口大小 + 1)
    let multiplier = 2.0 / (window as f64 + 1.0);
    // 初始化EMA为价格数组的第一个元素
    let mut ema = prices[0];
    // 遍历价格数组，从第二个元素开始
    for &price in prices.iter().skip(1) {
        // 根据EMA公式更新EMA值
        // EMA = (当前价格 - 上一个EMA) * 平滑因子 + 上一个EMA
        ema = (price - ema) * multiplier + ema;
    }
    // 返回计算得到的EMA值
    ema
}

// 定义一个函数，用于计算相对强弱指数（RSI）信号
pub fn calculate_rsi_signal(price_data: &PriceData) -> SignalStrength {
    // 设置RSI的周期为14
    let rsi_period = 14;
    // 如果价格数据中的价格数量小于RSI周期加1，则无法计算变化率，返回无信号的SignalStrength
    if price_data.prices.len() < rsi_period + 1 {
        // +1 是为了计算变化率
        return SignalStrength {
            buy_strength: 0.0,
            sell_strength: 0.0,
        };
    }
    // 初始化收益和亏损为0.0
    let mut gains = 0.0; // 收益
    let mut losses = 0.0; // 亏损
                          // 遍历价格数据中最后RSI周期内的每一天
    for i in price_data.prices.len() - rsi_period..price_data.prices.len() {
        // 计算每一天的价格变化
        let change = price_data.prices[i] - price_data.prices[i - 1];
        // 如果价格变化为正，则累加到收益中
        if change > 0.0 {
            gains += change;
        } else {
            // 如果价格变化为负，则累加到亏损中（取绝对值）
            losses += -change;
        }
    }
    // 计算平均收益和平均亏损
    let avg_gain = gains / rsi_period as f64;
    let avg_loss = losses / rsi_period as f64;
    // 根据平均收益和平均亏损计算RSI值
    let rsi = if avg_loss == 0.0 {
        100.0
    } else {
        100.0 - 100.0 / (1.0 + avg_gain / avg_loss)
    };

    // 根据RSI值判断信号强度
    if rsi < 30.0 {
        // RSI 低于 30，买入信号
        SignalStrength {
            buy_strength: (30.0 - rsi) / 30.0,
            sell_strength: 0.0,
        }
    } else if rsi > 70.0 {
        // RSI 高于 70，卖出信号
        SignalStrength {
            buy_strength: 0.0,
            sell_strength: (rsi - 70.0) / 30.0,
        }
    } else {
        // RSI 在 30 和 70 之间，无信号
        SignalStrength {
            buy_strength: 0.0,
            sell_strength: 0.0,
        }
    }
}

// 布林带信号，计算20SMA和标准差，当最新价格触及上下轨时给出信号
pub fn calculate_bollinger_signal(price_data: &PriceData) -> SignalStrength {
    // 设置布林带的周期为20
    let period = 20;
    // 如果价格数据中的价格数量小于周期，则返回一个买入和卖出强度都为0的信号强度
    if price_data.prices.len() < period {
        return SignalStrength {
            buy_strength: 0.0,
            sell_strength: 0.0,
        };
    }
    // 获取最近20个价格数据的切片
    let slice = &price_data.prices[price_data.prices.len() - period..];
    // 计算这20个价格数据的简单移动平均数（SMA）
    let sma = slice.iter().sum::<f64>() / period as f64;
    // 计算这20个价格数据的方差
    let variance = slice.iter().map(|x| (x - sma).powi(2)).sum::<f64>() / period as f64;
    // 计算标准差（std_dev）
    let std_dev = variance.sqrt();
    // 计算布林带的上轨
    let upper = sma + 2.0 * std_dev;
    // 计算布林带的下轨
    let lower = sma - 2.0 * std_dev;
    // 获取最后一个价格
    let last_price = *price_data.prices.last().unwrap();

    // 如果最后一个价格小于等于下轨，则计算买入强度，卖出强度为0
    if last_price <= lower {
        SignalStrength {
            buy_strength: (lower - last_price) / (2.0 * std_dev),
            sell_strength: 0.0,
        }
    // 如果最后一个价格大于等于上轨，则计算卖出强度，买入强度为0
    } else if last_price >= upper {
        SignalStrength {
            buy_strength: 0.0,
            sell_strength: (last_price - upper) / (2.0 * std_dev),
        }
    // 如果最后一个价格在上下轨之间，则买入和卖出强度都为0
    } else {
        SignalStrength {
            buy_strength: 0.0,
            sell_strength: 0.0,
        }
    }
}

/// KDJ信号：计算RSV、K、D、J值，J值超买超卖时给出信号
pub fn calculate_kdj_signal(price_data: &PriceData) -> SignalStrength {
    // 设置周期为9
    let period = 9;
    // 检查价格数据是否足够，如果任意一个序列长度小于周期或收盘价序列为空，则返回信号强度为0
    if price_data.highs.len() < period
        || price_data.lows.len() < period
        || price_data.closes.is_empty()
    {
        return SignalStrength {
            buy_strength: 0.0,
            sell_strength: 0.0,
        };
    }
    // 计算最近周期内的最高价
    let recent_high = price_data.highs[price_data.highs.len() - period..]
        .iter()
        .cloned()
        .fold(f64::MIN, f64::max);
    // 计算最近周期内的最低价
    let recent_low = price_data.lows[price_data.lows.len() - period..]
        .iter()
        .cloned()
        .fold(f64::MAX, f64::min);
    // 获取当前收盘价
    let current_close = *price_data.closes.last().unwrap();

    // 计算RSV值（未成熟随机值），如果最高价等于最低价，则RSV为50，否则根据公式计算
    let rsv = if recent_high == recent_low {
        50.0
    } else {
        (current_close - recent_low) / (recent_high - recent_low) * 100.0
    };
    // 简化：K、D均直接采用RSV值，真实实现中应使用平滑递归
    let k = rsv;
    let d = rsv;
    let j = 3.0 * k - 2.0 * d; // 实际上 j == rsv

    if j < 20.0 {
        SignalStrength {
            buy_strength: (20.0 - j) / 20.0,
            sell_strength: 0.0,
        }
    } else if j > 80.0 {
        SignalStrength {
            buy_strength: 0.0,
            sell_strength: (j - 80.0) / 20.0,
        }
    } else {
        SignalStrength {
            buy_strength: 0.0,
            sell_strength: 0.0,
        }
    }
}

/// 均线交叉信号：判断短期SMA与长期SMA的交叉情况，金叉买入，死叉卖出
pub fn calculate_ma_cross_signal(price_data: &PriceData) -> SignalStrength {
    // 定义短期均线的窗口大小
    let short_window = 5;
    // 定义长期均线的窗口大小
    let long_window = 20;
    // 如果价格数据长度不足以计算长期均线，则返回无信号
    if price_data.prices.len() < long_window + 1 {
        return SignalStrength {
            buy_strength: 0.0,
            sell_strength: 0.0,
        };
    }
    // 获取价格数据
    let prices = &price_data.prices;
    // 获取当前时点的索引
    let idx_current = prices.len() - 1;
    // 获取前一个时点的索引
    let idx_prev = prices.len() - 2;

    // 计算前一个时点的SMA
    let short_sma_prev: f64 = prices[idx_prev + 1 - short_window..=idx_prev]
        .iter()
        .sum::<f64>()
        / short_window as f64;
    let long_sma_prev: f64 = prices[idx_prev + 1 - long_window..=idx_prev]
        .iter()
        .sum::<f64>()
        / long_window as f64;
    // 计算当前时点的SMA
    let short_sma_current: f64 = prices[idx_current + 1 - short_window..=idx_current]
        .iter()
        .sum::<f64>()
        / short_window as f64;
    let long_sma_current: f64 = prices[idx_current + 1 - long_window..=idx_current]
        .iter()
        .sum::<f64>()
        / long_window as f64;

    // 金叉：上一个时点短期均线低于长期均线，而当前时点短期均线上穿长期均线
    if short_sma_prev < long_sma_prev && short_sma_current >= long_sma_current {
        SignalStrength {
            buy_strength: 1.0,
            sell_strength: 0.0,
        }
    }
    // 死叉：上一个时点短期均线高于长期均线，而当前时点短期均线下穿长期均线
    else if short_sma_prev > long_sma_prev && short_sma_current <= long_sma_current {
        SignalStrength {
            buy_strength: 0.0,
            sell_strength: 1.0,
        }
    } else {
        SignalStrength {
            buy_strength: 0.0,
            sell_strength: 0.0,
        }
    }
}

// 使用示例
pub fn execute_trading_strategy(price_data: &PriceData) -> TradeSignal {
    let aggregator = SignalAggregator::new(0.6);
    let signals = generate_trading_signals(price_data);
    aggregator.generate_composite_signal(&signals)
}

#[cfg(test)]
mod tests {
    use crate::TradeSignal;

    use super::*;

    #[test]
    fn test_signal_aggregation() {
        let aggregator = SignalAggregator::new(0.6);
        let mut signals = HashMap::new();

        signals.insert(
            "MACD".to_string(),
            SignalStrength {
                buy_strength: 0.8,
                sell_strength: 0.2,
            },
        );

        signals.insert(
            "RSI".to_string(),
            SignalStrength {
                buy_strength: 0.7,
                sell_strength: 0.3,
            },
        );

        let signal = aggregator.generate_composite_signal(&signals);
        assert_eq!(signal, TradeSignal::Buy);
    }
}
