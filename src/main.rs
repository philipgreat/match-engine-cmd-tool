use clap::Parser;
use std::net::{UdpSocket, ToSocketAddrs, Ipv4Addr,IpAddr};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::io;
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

// 假设的 Checksum 计算函数（这里用一个简单的实现作为占位符）
fn calculate_checksum(buf: &[u8]) -> u8 {
    // Checksum is calculated over the payload (index 2 onwards)
    buf[2..].iter().fold(0, |acc, &x| acc ^ x)
}

// 序列化 Order 结构体
pub fn serialize_order(order: &Order) -> [u8; MESSAGE_TOTAL_SIZE] {
    let mut buf = [0u8; MESSAGE_TOTAL_SIZE];
    let payload_start = 2; // Checksum (0) + Type (1) = Start at index 2

    buf[1] = MSG_ORDER_SUBMIT;

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

// 获取自 Unix Epoch (1970-01-01) 以来的纳秒数
fn get_nanos_since_epoch() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("SystemTime before UNIX EPOCH: {}", e))
        .and_then(|duration| duration.as_nanos().try_into().map_err(|_| "Timestamp too large".to_string()))
}

// 通用的 Multicast 发送函数
fn send_multicast_message(socket: &UdpSocket, addr: &str, message: &[u8]) -> Result<(), String> {
    match socket.send_to(message, addr) {
        Ok(bytes_sent) => {
            if bytes_sent == message.len() {
                Ok(())
            } else {
                Err(format!("Partial send: {} of {} bytes sent.", bytes_sent, message.len()))
            }
        }
        Err(e) => Err(format!("Failed to send multicast message to {}: {}", addr, e)),
    }
}


// --- 命令行参数结构体 ---

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 交易引擎的组播地址 (IP:Port)，用于发送订单和撤单请求
    #[arg(long, default_value = "239.0.0.1:5000")]
    trade_addr: String,

    // 提交订单的子命令
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    /// 提交一个新的订单
    Submit(SubmitArgs),
    /// 撤销一个订单
    Cancel(CancelArgs),
}

// ... SubmitArgs 和 CancelArgs 结构体保持不变 ...

#[derive(Parser, Debug)]
struct SubmitArgs {
    /// 产品 ID (u16)
    #[arg(long)]
    product_id: u16,

    /// 价格 (u64)
    #[arg(long)]
    price: u64,

    /// 数量 (u32)
    #[arg(long)]
    quantity: u32,
    
    /// 订单类型：buy 或 sell
    #[arg(long, value_parser = parse_order_type)]
    order_type: u8,

    /// 价格类型：limit 或 market
    #[arg(long, value_parser = parse_price_type)]
    price_type: u8,

    /// 订单过期时间，以秒为单位 (GTC/0 means never expire)
    #[arg(long, default_value = "0")]
    expire: u64,
}

#[derive(Parser, Debug)]
struct CancelArgs {
    /// 要撤销的唯一订单 ID (u64)
    #[arg(long)]
    order_id: u64,
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

// --- 主逻辑 ---

fn main() -> Result<(), String> {
    let args = Args::parse();
    let trade_addr = &args.trade_addr;

    // 1. 创建 UDP Socket。
    // 绑定到 0.0.0.0:0 意味着绑定到所有可用接口的随机端口，适合作为发送端。
    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| format!("Failed to bind UDP socket: {}", e))?;

    // 2. 尝试解析组播地址并设置 TTL
    if let Ok(mut addrs) = trade_addr.to_socket_addrs() {
        if let Some(socket_addr) = addrs.next() {
            if socket_addr.ip().is_multicast() {
                // Multicast TTL (Time To Live) 默认为 1，我们设置为 1 以限制在本地网络。
                // 如果需要跨路由器，应设置为更高值。
                if let Err(e) = socket.set_multicast_ttl_v4(10) {
                    eprintln!("Warning: Failed to set Multicast TTL (this is often okay for simple sending): {}", e);
                }
            }
        }
    }
    
    println!("Target Trade Address: {}", trade_addr);

    match args.command {
        Command::Submit(submit_args) => {
            handle_submit(submit_args, &socket, trade_addr)?;
        }
        Command::Cancel(cancel_args) => {
            handle_cancel(cancel_args, &socket, trade_addr)?;
        }
    }

    Ok(())
}

fn handle_submit(args: SubmitArgs, socket: &UdpSocket, trade_addr: &str) -> Result<(), String> {
    // ... 时间戳计算逻辑保持不变 ...
    let submit_time = get_nanos_since_epoch()?;
    let expire_time = if args.expire > 0 {
        let expire_nanos: u64 = args.expire.checked_mul(1_000_000_000)
            .ok_or_else(|| "Expiration duration overflow".to_string())?;
        
        submit_time.checked_add(expire_nanos)
            .ok_or_else(|| "Expiration time overflow".to_string())?
    } else {
        0 // 0 means GTC
    };

    let order_id = submit_time; 

    let order = Order {
        product_id: args.product_id,
        order_id: order_id,
        price: args.price,
        quantity: args.quantity,
        order_type: args.order_type,
        price_type: args.price_type,
        submit_time,
        expire_time: submit_time + 1000*1000*1000,
    };

    let serialized_message = serialize_order(&order);

    // 调用新的发送函数
    send_multicast_message(socket, trade_addr, &serialized_message)?;

    println!("--- Order Submit Request (Sent to {}) ---", trade_addr);
    println!("Order ID: {}", order_id);
    println!("Product ID: {}", order.product_id);
    // ... 其他打印信息保持不变 ...
    println!("Serialized Message ({} bytes): {:?}", MESSAGE_TOTAL_SIZE, serialized_message);
    
    Ok(())
}

fn handle_cancel(args: CancelArgs, socket: &UdpSocket, trade_addr: &str) -> Result<(), String> {
    // ... 撤单消息序列化逻辑保持不变 ...
    let mut cancel_buf = [0u8; MESSAGE_TOTAL_SIZE];
    cancel_buf[1] = MSG_ORDER_CANCEL;
    
    // 假设 Order ID 从第 2 个字节开始
    cancel_buf[2..10].copy_from_slice(&args.order_id.to_be_bytes());
    
    // 计算 Checksum
    cancel_buf[0] = calculate_checksum(&cancel_buf);

    // 调用新的发送函数
    send_multicast_message(socket, trade_addr, &cancel_buf)?;

    println!("--- Order Cancel Request (Sent to {}) ---", trade_addr);
    println!("Order ID to Cancel: {}", args.order_id);
    println!("Serialized Message ({} bytes): {:?}", MESSAGE_TOTAL_SIZE, cancel_buf);
    
    Ok(())
}