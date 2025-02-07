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
    pub highs: Vec<f64>, // 最高价数据
    pub lows: Vec<f64>, //  最低价数据
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

    pub fn generate_composite_signal(&self, signals: &HashMap<String, SignalStrength>) -> TradeSignal {
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
fn calculate_macd_signal(price_data: &PriceData) -> SignalStrength {
    if price_data.prices.len() < 26 {
        return SignalStrength {
            buy_strength: 0.0,
            sell_strength: 0.0,
        };        
    }

    let short_window = 12;
    let long_window = 26;
    let short_ema = price_data.prices[price_data.prices.len() - short_window..].iter().sum::<f64>() / short_window as f64;
    let long_ema = price_data.prices[price_data.prices.len() - long_window..].iter().sum::<f64>() / long_window as f64;
    let macd_hist = short_ema - long_ema;

    if macd_hist > 0.0 {
        SignalStrength {
            buy_strength: macd_hist,
            sell_strength: 0.0,
        }
    } else {
        SignalStrength {
            buy_strength: 0.0,
            sell_strength: macd_hist.abs(),
        }
    }
        
}

pub fn calculate_rsi_signal(price_data: &PriceData) -> SignalStrength {
    let rsi_period = 14;
    if price_data.prices.len() < rsi_period + 1 { // +1 是为了计算变化率
        return SignalStrength {
            buy_strength: 0.0,
            sell_strength: 0.0,
        };
    }
    let mut gains = 0.0; // 收益
    let mut losses = 0.0; // 亏损
    for i in price_data.prices.len() - rsi_period..price_data.prices.len() {
        let change = price_data.prices[i] - price_data.prices[i - 1];
        if change > 0.0 {
            gains += change;
        } else {
            losses += -change;
        }
    }
    let avg_gain = gains / rsi_period as f64;
    let avg_loss = losses / rsi_period as f64;
    let rsi = if avg_loss == 0.0 { 100.0 } else { 100.0 - 100.0 / (1.0 + avg_gain / avg_loss) };

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
    let period = 20;
    if price_data.prices.len() < period {
        return SignalStrength { buy_strength: 0.0, sell_strength: 0.0 };
    }
    let slice = &price_data.prices[price_data.prices.len()-period..];
    let sma = slice.iter().sum::<f64>() / period as f64;
    let variance = slice.iter().map(|x| (x - sma).powi(2)).sum::<f64>() / period as f64;
    let std_dev = variance.sqrt();
    let upper = sma + 2.0 * std_dev;
    let lower = sma - 2.0 * std_dev;
    let last_price = *price_data.prices.last().unwrap();

    if last_price <= lower {
         SignalStrength { buy_strength: (lower - last_price) / (2.0 * std_dev), sell_strength: 0.0 }
    } else if last_price >= upper {
         SignalStrength { buy_strength: 0.0, sell_strength: (last_price - upper) / (2.0 * std_dev) }
    } else {
         SignalStrength { buy_strength: 0.0, sell_strength: 0.0 }
    }
}

/// KDJ信号：计算RSV、K、D、J值，J值超买超卖时给出信号
pub fn calculate_kdj_signal(price_data: &PriceData) -> SignalStrength {
    let period = 9;
    if price_data.highs.len() < period || price_data.lows.len() < period || price_data.closes.is_empty() {
        return SignalStrength { buy_strength: 0.0, sell_strength: 0.0 };
    }
    let recent_high = price_data.highs[price_data.highs.len()-period..]
        .iter().cloned().fold(f64::MIN, f64::max);
    let recent_low = price_data.lows[price_data.lows.len()-period..]
        .iter().cloned().fold(f64::MAX, f64::min);
    let current_close = *price_data.closes.last().unwrap();

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
         SignalStrength { buy_strength: (20.0 - j) / 20.0, sell_strength: 0.0 }
    } else if j > 80.0 {
         SignalStrength { buy_strength: 0.0, sell_strength: (j - 80.0) / 20.0 }
    } else {
         SignalStrength { buy_strength: 0.0, sell_strength: 0.0 }
    }
}

/// 均线交叉信号：判断短期SMA与长期SMA的交叉情况，金叉买入，死叉卖出
pub fn calculate_ma_cross_signal(price_data: &PriceData) -> SignalStrength {
    let short_window = 5;
    let long_window = 20;
    if price_data.prices.len() < long_window + 1 {
         return SignalStrength { buy_strength: 0.0, sell_strength: 0.0 };
    }
    let prices = &price_data.prices;
    let idx_current = prices.len() - 1;
    let idx_prev = prices.len() - 2;

    // 计算前一个时点的SMA
    let short_sma_prev: f64 = prices[idx_prev+1-short_window..=idx_prev]
        .iter().sum::<f64>() / short_window as f64;
    let long_sma_prev: f64 = prices[idx_prev+1-long_window..=idx_prev]
        .iter().sum::<f64>() / long_window as f64;
    // 计算当前时点的SMA
    let short_sma_current: f64 = prices[idx_current+1-short_window..=idx_current]
        .iter().sum::<f64>() / short_window as f64;
    let long_sma_current: f64 = prices[idx_current+1-long_window..=idx_current]
        .iter().sum::<f64>() / long_window as f64;

    // 金叉：上一个时点短期均线低于长期均线，而当前时点短期均线上穿长期均线
    if short_sma_prev < long_sma_prev && short_sma_current >= long_sma_current {
         SignalStrength { buy_strength: 1.0, sell_strength: 0.0 }
    }
    // 死叉：上一个时点短期均线高于长期均线，而当前时点短期均线下穿长期均线
    else if short_sma_prev > long_sma_prev && short_sma_current <= long_sma_current {
         SignalStrength { buy_strength: 0.0, sell_strength: 1.0 }
    } else {
         SignalStrength { buy_strength: 0.0, sell_strength: 0.0 }
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
        
        signals.insert("MACD".to_string(), SignalStrength {
            buy_strength: 0.8,
            sell_strength: 0.2,
        });
        
        signals.insert("RSI".to_string(), SignalStrength {
            buy_strength: 0.7,
            sell_strength: 0.3,
        });
        
        let signal = aggregator.generate_composite_signal(&signals);
        assert_eq!(signal, TradeSignal::Buy);
    }
}