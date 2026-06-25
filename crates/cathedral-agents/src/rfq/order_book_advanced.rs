use std::collections::{HashMap, VecDeque};
use tokio::sync::Mutex;
use chrono::{Utc, DateTime};
use crate::rfq::pricing_engine::OrderSide;

#[derive(Clone, Debug)]
pub struct Order {
    pub id: String,
    pub asset_ref: String,
    pub side: OrderSide,
    pub amount: u64,
    pub filled: u64,
    pub price: f64,
    pub policy: ExecutionPolicy,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub peer_did: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExecutionPolicy {
    FOK,
    IOC,
    GTC,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OrderStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Cancelled,
    Expired,
    Settled,
}

pub struct OrderBookAdvanced {
    buy_orders: Mutex<Vec<Order>>,
    sell_orders: Mutex<Vec<Order>>,
    history: Mutex<Vec<Execution>>,
}

impl OrderBookAdvanced {
    pub fn new() -> Self {
        Self {
            buy_orders: Mutex::new(Vec::new()),
            sell_orders: Mutex::new(Vec::new()),
            history: Mutex::new(Vec::new()),
        }
    }

    pub async fn add_order(&self, order: Order) {
        let mut orders = match order.side {
            OrderSide::Buy => self.buy_orders.lock().await,
            OrderSide::Sell => self.sell_orders.lock().await,
        };
        orders.push(order.clone());
        self.sort_orders(&mut orders, &order.side);
    }

    fn sort_orders(&self, orders: &mut Vec<Order>, side: &OrderSide) {
        match side {
            OrderSide::Buy => {
                orders.sort_by(|a, b| {
                    b.price.partial_cmp(&a.price)
                        .unwrap()
                        .then(a.created_at.cmp(&b.created_at))
                });
            }
            OrderSide::Sell => {
                orders.sort_by(|a, b| {
                    a.price.partial_cmp(&b.price)
                        .unwrap()
                        .then(a.created_at.cmp(&b.created_at))
                });
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Execution {
    pub id: String,
    pub asset_ref: String,
    pub buy_order_id: String,
    pub sell_order_id: String,
    pub amount: u64,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
    pub status: ExecutionStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    Confirmed,
    Failed,
}
