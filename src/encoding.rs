// src/encoding.rs

use crate::types::{Order, MESSAGE_TOTAL_SIZE, MSG_ORDER_SUBMIT,MSG_TRADE_BROADCAST,MSG_STATUS_BROADCAST};
use crate::types::{MatchResult, BroadcastStats};

use std::convert::TryInto; // 用于 slice 转固定大小数组

// Payload starts after Checksum (1 byte) and Message Type (1 byte)
const PAYLOAD_START: usize = 2;




// 假设的 Checksum 计算函数
pub fn calculate_checksum(buf: &[u8]) -> u8 {
    // Checksum is calculated over the payload (index 2 onwards)
    buf[2..].iter().fold(0, |acc, &x| acc ^ x)
}

// 序列化 Order 结构体
pub fn serialize_order(order: &Order) -> [u8; MESSAGE_TOTAL_SIZE] {
    let mut buf = [0u8; MESSAGE_TOTAL_SIZE];
    let payload_start = 2; // Checksum (0) + Type (1) = Start at index 2

    buf[1] = MSG_ORDER_SUBMIT;

    // 结构体字段序列化... (大端序)
    // Product ID (u16)
    buf[payload_start..payload_start + 2].copy_from_slice(&order.product_id.to_be_bytes());
    // Order ID (u64)
    buf[payload_start + 2..payload_start + 10].copy_from_slice(&order.order_id.to_be_bytes());
    // Price (u64)
    buf[payload_start + 10..payload_start + 18].copy_from_slice(&order.price.to_be_bytes());
    // Quantity (u32)
    buf[payload_start + 18..payload_start + 22].copy_from_slice(&order.quantity.to_be_bytes());
    // Order Type (u8)
    buf[payload_start + 22] = order.order_type;
    // Price Type (u8)
    buf[payload_start + 23] = order.price_type;
    // Submit Time (u64)
    buf[payload_start + 24..payload_start + 32].copy_from_slice(&order.submit_time.to_be_bytes());
    // Expire Time (u64)
    buf[payload_start + 32..payload_start + 40].copy_from_slice(&order.expire_time.to_be_bytes());

    // Checksum calculation and placement
    buf[0] = calculate_checksum(&buf);

    buf
}



///
/// 解码 MatchResult 结构体
/// 
/// 注意：该函数假设 buf 长度 >= MESSAGE_TOTAL_SIZE 且校验和已验证。
///
pub fn deserialize_match_result(buf: &[u8]) -> Result<MatchResult, &'static str> {
    if buf.len() < MESSAGE_TOTAL_SIZE {
        return Err("Buffer size is too small for MatchResult.");
    }

    let mut current_idx = PAYLOAD_START;

    // 1. Instance Tag ([u8; 8])
    let instance_tag: [u8; 8] = buf[current_idx..current_idx + 8].try_into().map_err(|_| "Failed to read instance_tag")?;
    current_idx += 8;

    // 2. Product ID (u16)
    let product_id_bytes: [u8; 2] = buf[current_idx..current_idx + 2].try_into().map_err(|_| "Failed to read product_id")?;
    let product_id = u16::from_be_bytes(product_id_bytes);
    current_idx += 2;

    // 3. Buy Order ID (u64)
    let buy_order_id_bytes: [u8; 8] = buf[current_idx..current_idx + 8].try_into().map_err(|_| "Failed to read buy_order_id")?;
    let buy_order_id = u64::from_be_bytes(buy_order_id_bytes);
    current_idx += 8;

    // 4. Sell Order ID (u64)
    let sell_order_id_bytes: [u8; 8] = buf[current_idx..current_idx + 8].try_into().map_err(|_| "Failed to read sell_order_id")?;
    let sell_order_id = u64::from_be_bytes(sell_order_id_bytes);
    current_idx += 8;

    // 5. Price (u64)
    let price_bytes: [u8; 8] = buf[current_idx..current_idx + 8].try_into().map_err(|_| "Failed to read price")?;
    let price = u64::from_be_bytes(price_bytes);
    current_idx += 8;

    // 6. Quantity (u32)
    let quantity_bytes: [u8; 4] = buf[current_idx..current_idx + 4].try_into().map_err(|_| "Failed to read quantity")?;
    let quantity = u32::from_be_bytes(quantity_bytes);
    current_idx += 4;

    // 7. Trade Time Network (u32)
    let trade_network_time_bytes: [u8; 4] = buf[current_idx..current_idx + 4].try_into().map_err(|_| "Failed to read trade_network_time")?;
    let trade_network_time = u32::from_be_bytes(trade_network_time_bytes);
    current_idx += 4;

    // 8. Internal Match Time (u32)
    // 注意: 您的序列化代码中这里实际上是重复写入了 trade_network_time 的值，
    // 解码时，我们根据 MatchResult 结构体字段来读，它应是 internal_match_time
    // 假设序列化代码的意图是 Trade Time (u32) + Internal Match Time (u32)。
    let internal_match_time_bytes: [u8; 4] = buf[current_idx..current_idx + 4].try_into().map_err(|_| "Failed to read internal_match_time")?;
    let internal_match_time = u32::from_be_bytes(internal_match_time_bytes);
    // current_idx += 4; // 不需要再增加，因为这是最后一个字段

    Ok(MatchResult {
        instance_tag,
        product_id,
        buy_order_id,
        sell_order_id,
        price,
        quantity,
        trade_network_time,
        internal_match_time,
    })
}


pub fn deserialize_stats_result(buf: &[u8]) -> Result<BroadcastStats, &'static str> {
    if buf.len() < MESSAGE_TOTAL_SIZE {
        return Err("Buffer size is too small for BroadcastStats.");
    }
    
    let mut current_idx = PAYLOAD_START;

    // 1. Instance Tag ([u8; 8])
    let instance_tag: [u8; 8] = buf[current_idx..current_idx + 8].try_into().map_err(|_| "Failed to read instance_tag")?;
    current_idx += 8;

    // 2. Product ID (u16)
    let product_id_bytes: [u8; 2] = buf[current_idx..current_idx + 2].try_into().map_err(|_| "Failed to read product_id")?;
    let product_id = u16::from_be_bytes(product_id_bytes);
    current_idx += 2;

    // 3. Bids Size (u32)
    let bids_size_bytes: [u8; 4] = buf[current_idx..current_idx + 4].try_into().map_err(|_| "Failed to read bids_size")?;
    let bids_size = u32::from_be_bytes(bids_size_bytes);
    current_idx += 4;

    // 4. Ask Size (u32)
    let ask_size_bytes: [u8; 4] = buf[current_idx..current_idx + 4].try_into().map_err(|_| "Failed to read ask_size")?;
    let ask_size = u32::from_be_bytes(ask_size_bytes);
    current_idx += 4;

    // 5. Matched Orders (u32)
    let matched_orders_bytes: [u8; 4] = buf[current_idx..current_idx + 4].try_into().map_err(|_| "Failed to read matched_orders")?;
    let matched_orders = u32::from_be_bytes(matched_orders_bytes);
    current_idx += 4;

    // 6. Total Received Orders (u32)
    let total_received_orders_bytes: [u8; 4] = buf[current_idx..current_idx + 4].try_into().map_err(|_| "Failed to read total_received_orders")?;
    let total_received_orders = u32::from_be_bytes(total_received_orders_bytes);
    current_idx += 4;

    // 7. Start Time (u64)
    let start_time_bytes: [u8; 8] = buf[current_idx..current_idx + 8].try_into().map_err(|_| "Failed to read start_time")?;
    let start_time = u64::from_be_bytes(start_time_bytes);
    // current_idx += 8; // 不需要再增加，因为这是最后一个字段

    Ok(BroadcastStats {
        instance_tag,
        product_id,
        bids_size,
        ask_size,
        matched_orders,
        total_received_orders,
        start_time,
    })
}


/// 根据消息类型分派并解码结果
pub fn decode_broadcast_message(buf: &[u8]) -> Result<String, String> {
    if buf.len() < MESSAGE_TOTAL_SIZE {
        return Err("Received buffer is too small.".to_string());
    }

    let msg_type = buf[1];

    // 假设校验和在网络接收前已经被检查

    match msg_type {
        MSG_TRADE_BROADCAST => {
            let result = deserialize_match_result(buf)
                .map_err(|e| format!("Failed to decode MatchResult: {}", e))?;
            
            Ok(format!("🔥 TRADE: Product={} | Price={} | Qty={} | BuyID={} | SellId={}| Net={}ns | Match={}ns", 
                result.product_id, result.price, result.quantity, result.buy_order_id, result.sell_order_id,
                result.trade_network_time,
                result.internal_match_time))
        },
        MSG_STATUS_BROADCAST => {
            let stats = deserialize_stats_result(buf)
                .map_err(|e| format!("Failed to decode BroadcastStats: {}", e))?;

            Ok(format!("📊 STATUS: Product={} | Bids={} | Asks={} | Matched={} | Received={}", 
                stats.product_id, stats.bids_size, stats.ask_size, stats.matched_orders, stats.total_received_orders))
        },
        _ => Err(format!("Unknown or unhandled message type: {:?}", buf)),
    }
}
