// src/types.rs

use std::time::{SystemTime, UNIX_EPOCH, Duration};

// --- Message Type Constants ---
pub const MSG_ORDER_SUBMIT: u8 = 1;      // Client -> Engine: Order submission
pub const MSG_ORDER_CANCEL: u8 = 2;      // Client -> Engine: Order cancellation
pub const MSG_TRADE_BROADCAST: u8 = 10;  // Engine -> Client: Trade broadcast
pub const MSG_STATUS_BROADCAST: u8 = 11; // Engine -> Client: Status broadcast

// --- Order Type Constants ---
pub const ORDER_TYPE_BUY: u8 = 1;          // Order side: Buy
pub const ORDER_TYPE_SELL: u8 = 2;         // Order side: Sell
pub const ORDER_PRICE_TYPE_LIMIT: u8 = 1;  // Order price type: Limit
pub const ORDER_PRICE_TYPE_MARKET: u8 = 2; // Order price type: Market

// --- Message Size Constant ---
pub const MESSAGE_TOTAL_SIZE: usize = 50; // All network packets are 50 bytes fixed size.

// --- Data Structure Definitions ---

// Order Structure (for MSG_ORDER_SUBMIT)
#[derive(Debug, Clone)]
pub struct Order {
    pub product_id: u16,    // Product identifier (2 bytes)
    pub order_id: u64,      // Unique order ID (8 bytes)
    pub price: u64,         // Price (8 bytes)
    pub quantity: u32,      // Quantity (4 bytes)
    pub order_type: u8,     // Order side (BUY/SELL) (1 byte)
    pub price_type: u8,     // Price type (LIMIT/MARKET) (1 byte)
    pub submit_time: u64,   // Submission timestamp (Nanoseconds) (8 bytes)
    pub expire_time: u64,   // Expiration timestamp (Nanoseconds. 0 means GTC) (8 bytes)
    // Total Payload Size: 40 bytes
}

// 获取自 Unix Epoch (1970-01-01) 以来的纳秒数
pub fn get_nanos_since_epoch() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("SystemTime before UNIX EPOCH: {}", e))
        .and_then(|duration| duration.as_nanos().try_into().map_err(|_| "Timestamp too large".to_string()))
}


pub fn serialize_stats_result(stats: &BroadcastStats) -> [u8; MESSAGE_TOTAL_SIZE] {
    let mut buf = [0u8; MESSAGE_TOTAL_SIZE];

    // Payload starts after Checksum (1 byte) and Message Type (1 byte)
    let payload_start_idx = 2;
    let mut current_idx = payload_start_idx;

    // Assuming MSG_STATUS_BROADCAST and calculate_checksum are defined elsewhere
    buf[1] = MSG_STATUS_BROADCAST;

    // --- Payload Serialization (Total 30 bytes) ---

    // 1. Instance Tag ([u8; 8])
    // Size: 8 bytes
    buf[current_idx..current_idx + 8].copy_from_slice(&stats.instance_tag);
    current_idx += 8; // Index: 10

    // 2. Product ID (u16)
    // Size: 2 bytes
    buf[current_idx..current_idx + 2].copy_from_slice(&stats.product_id.to_be_bytes());
    current_idx += 2; // Index: 12

    // 3. Order Book Size (u32)
    // Size: 4 bytes (FIXED from u64)
    buf[current_idx..current_idx + 4].copy_from_slice(&stats.bids_size.to_be_bytes());
    current_idx += 4; // Index: 16

    buf[current_idx..current_idx + 4].copy_from_slice(&stats.ask_size.to_be_bytes());
    current_idx += 4; // Index: 16

    // 4. Matched Orders (u32)
    // Size: 4 bytes (FIXED from u64)
    buf[current_idx..current_idx + 4].copy_from_slice(&stats.matched_orders.to_be_bytes());
    current_idx += 4; // Index: 20

    // 5. Total Received Orders (u32)
    // Size: 4 bytes (FIXED from u64)
    buf[current_idx..current_idx + 4].copy_from_slice(&stats.total_received_orders.to_be_bytes());
    current_idx += 4; // Index: 24

    // 6. Start Time (u64)
    // Size: 8 bytes
    buf[current_idx..current_idx + 8].copy_from_slice(&stats.start_time.to_be_bytes());
    current_idx += 8; // Index: 32 (Last index written: 31)

    // Checksum calculation and placement
    // Last data byte is at index 31. Padding goes from index 32 up to MESSAGE_TOTAL_SIZE - 1.
    buf[0] = calculate_checksum(&buf);

    buf
}

#[derive(Debug, Clone)]
pub struct BroadcastStats {
    pub instance_tag: [u8; 8],      // 8-byte engine instance tag
    pub product_id: u16,            // Product identifier (2 bytes)
    pub bids_size: u32,             // Current order book size (4 bytes)
    pub ask_size: u32,              // Current order book size (4 bytes)
    pub matched_orders: u32,        // Total matched orders count (4 bytes)
    pub total_received_orders: u32, // Total received orders count (4 bytes)
    pub start_time: u64,            // Program start time (Nanoseconds) (8 bytes)
                                    // Total Payload Size: 42 bytes
}

// Match Result Structure (for MSG_TRADE_BROADCAST)
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub instance_tag: [u8; 8],    // 8-byte engine instance tag
    pub product_id: u16,          // Product identifier (2 bytes)
    pub buy_order_id: u64,        // Buyer's order ID (8 bytes)
    pub sell_order_id: u64,       // Seller's order ID (8 bytes)
    pub price: u64,               // Trade price (8 bytes)
    pub quantity: u32,            // Trade quantity (4 bytes)
    pub trade_network_time: u32,  // Trade timestamp (Nanoseconds) (8 bytes)
    pub internal_match_time: u32, // Total Payload Size: 46 bytes
}


fn calculate_checksum(buf: &[u8]) -> u8 {
    // Checksum is calculated over the payload (index 2 onwards)
    buf[2..].iter().fold(0, |acc, &x| acc ^ x)
}