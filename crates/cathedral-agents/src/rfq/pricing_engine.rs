use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum OrderSide {
    Buy,
    Sell,
}

pub trait PricingStrategy: Send + Sync {
    fn calculate_price(
        &self,
        asset_ref: &str,
        amount: u64,
        side: OrderSide,
        context: &PricingContext,
    ) -> f64;
}

pub struct PricingContext {
    pub market_price: f64,
    pub liquidity: u64,
    pub volatility: f64,
    pub timestamp: DateTime<Utc>,
    pub peer_reputation: f64, // 0.0 - 1.0
}

pub struct MarketSpreadPricing {
    spread_base: f64,              // spread base (ex: 0.01 = 1%)
    spread_volatility_factor: f64, // fator de volatilidade
    min_spread: f64,
    max_spread: f64,
}

impl MarketSpreadPricing {
    pub fn new(spread_base: f64, volatility_factor: f64, min_spread: f64, max_spread: f64) -> Self {
        Self {
            spread_base,
            spread_volatility_factor: volatility_factor,
            min_spread,
            max_spread,
        }
    }

    fn calculate_spread(&self, volatility: f64) -> f64 {
        let spread = self.spread_base + (volatility * self.spread_volatility_factor);
        spread.clamp(self.min_spread, self.max_spread)
    }
}

impl PricingStrategy for MarketSpreadPricing {
    fn calculate_price(
        &self,
        _asset_ref: &str,
        amount: u64,
        side: OrderSide,
        context: &PricingContext,
    ) -> f64 {
        let spread = self.calculate_spread(context.volatility);
        let base_price = context.market_price;

        let liquidity_ratio = amount as f64 / (context.liquidity + 1) as f64;
        let size_adj = (liquidity_ratio * 0.5).min(0.05);

        let total_spread = spread + size_adj;

        match side {
            OrderSide::Buy => base_price * (1.0 + total_spread),
            OrderSide::Sell => base_price * (1.0 - total_spread),
        }
    }
}
