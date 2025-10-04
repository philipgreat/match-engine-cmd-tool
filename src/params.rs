// src/params.rs

use clap::{Parser, Subcommand};
use crate::types::{ORDER_TYPE_BUY, ORDER_TYPE_SELL, ORDER_PRICE_TYPE_LIMIT, ORDER_PRICE_TYPE_MARKET};

// --- 命令行参数结构体 ---

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// 交易引擎的组播地址 (IP:Port)，用于发送订单和撤单请求
    #[arg(long, default_value = "239.0.0.1:5000")]
    pub trade_addr: String,

    /// 接收交易结果和状态的组播地址 (IP:Port)。默认为 239.0.0.2:5001
    #[arg(long, default_value = "239.0.0.2:5001")]
    pub result_addr: String, // <--- 新增字段
    
    // 提交订单的子命令
    #[clap(subcommand)]
    pub command: Command,
}


#[derive(Subcommand, Debug)]
pub enum Command {
    /// 提交一个新的订单
    Submit(SubmitArgs),
    /// 撤销一个订单
    Cancel(CancelArgs),
}

#[derive(Parser, Debug)]
pub struct SubmitArgs {
    /// 产品 ID (u16)
    #[arg(long)]
    pub product_id: u16,

    /// 价格 (u64)
    #[arg(long)]
    pub price: u64,

    /// 数量 (u32)
    #[arg(long)]
    pub quantity: u32,
    
    /// 订单类型：buy 或 sell
    #[arg(long, value_parser = parse_order_type)]
    pub order_type: u8,

    /// 价格类型：limit 或 market
    #[arg(long, value_parser = parse_price_type)]
    pub price_type: u8,

    /// 订单过期时间，以秒为单位 (GTC/0 means never expire)
    #[arg(long, default_value = "0")]
    pub expire: u64,
}

#[derive(Parser, Debug)]
pub struct CancelArgs {
    /// 要撤销的唯一订单 ID (u64)
    #[arg(long)]
    pub order_id: u64,
}

// 辅助解析函数
fn parse_order_type(s: &str) -> Result<u8, String> {
    match s.to_lowercase().as_str() {
        "buy" => Ok(ORDER_TYPE_BUY),
        "sell" => Ok(ORDER_TYPE_SELL),
        _ => Err(format!("Invalid order type: {}. Must be 'buy' or 'sell'", s)),
    }
}

fn parse_price_type(s: &str) -> Result<u8, String> {
    match s.to_lowercase().as_str() {
        "limit" => Ok(ORDER_PRICE_TYPE_LIMIT),
        "market" => Ok(ORDER_PRICE_TYPE_MARKET),
        _ => Err(format!("Invalid price type: {}. Must be 'limit' or 'market'", s)),
    }
}