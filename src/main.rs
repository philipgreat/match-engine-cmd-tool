// src/main.rs

use clap::Parser;
use std::net::{ToSocketAddrs, UdpSocket};

// 导入其他模块
mod types;
mod encoding;
mod network;
mod params;

use types::{Order, get_nanos_since_epoch, MESSAGE_TOTAL_SIZE, MSG_ORDER_CANCEL};
use encoding::{serialize_order, calculate_checksum,decode_broadcast_message};
use network::{create_multicast_listener, send_message};
use params::{Args, Command, SubmitArgs, CancelArgs};


const DEFAULT_ORDER EXECUTION_ADDR: &str = "239.0.0.1:5000";
const DEFAULT_STATUS_ADDR: &str = "239.0.0.2:5001";
// 监听组播时，绑定地址需要包含端口，但IP通常是0.0.0.0
// 为了简化，我们只监听 trade_addr 或 status_addr 的端口
const DEFAULT_LISTEN_IP: &str = "0.0.0.0";


fn main() -> Result<(), String> {
    let args = Args::parse();
    let trade_addr = &args.trade_addr;
    let result_addr = &args.result_addr;
    
    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| format!("Failed to bind UDP socket: {}", e))?;

    let listener_socket = create_multicast_listener(result_addr)?;
    println!("📡 Starting Broadcast Listener on {}", result_addr);
    

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
    println!("Result  Address: {}", result_addr);

    // 2. 根据子命令执行逻辑
    match args.command {
        Command::Submit(submit_args) => {
            handle_submit(submit_args, &socket, trade_addr)?;
        }
        Command::Cancel(cancel_args) => {
            handle_cancel(cancel_args, &socket, trade_addr)?;
        }
    }

    receive_broadcasts(listener_socket)
        .map_err(|e| format!("Broadcast receiver failed: {}", e))?;


    Ok(())
}

fn handle_submit(args: SubmitArgs, socket: &UdpSocket, trade_addr: &str) -> Result<(), String> {
    // 1. 时间戳和订单 ID 计算
    let submit_time = get_nanos_since_epoch()?;
    let expire_time = if args.expire > 0 {
        let expire_nanos: u64 = args.expire.checked_mul(1_000_000_000)
            .ok_or_else(|| "Expiration duration overflow".to_string())?;
        
        submit_time.checked_add(expire_nanos)
            .ok_or_else(|| "Expiration time overflow".to_string())?
    } else {
        0 // 0 means GTC
    };

    // 订单 ID 简单使用 submit_time
    let order_id = submit_time; 

    // 2. 构建 Order 结构体
    let order = Order {
        product_id: args.product_id,
        order_id: order_id,
        price: args.price,
        quantity: args.quantity,
        order_type: args.order_type,
        price_type: args.price_type,
        submit_time,
        expire_time,
    };

    // 3. 序列化消息
    let serialized_message = serialize_order(&order);

    // 4. 发送消息
    send_message(socket, trade_addr, &serialized_message)?;

    // 5. 打印结果
    println!("--- Order Submit Request (Sent to {}) ---", trade_addr);
    println!("Order ID: {}", order_id);
    println!("Product ID: {}", order.product_id);
    println!("Price: {}, Quantity: {}", order.price, order.quantity);
    println!("Serialized Message ({} bytes): {:?}", MESSAGE_TOTAL_SIZE, serialized_message);
    
    Ok(())
}

fn handle_cancel(args: CancelArgs, socket: &UdpSocket, trade_addr: &str) -> Result<(), String> {
    // 1. 构建撤单消息
    let mut cancel_buf = [0u8; MESSAGE_TOTAL_SIZE];
    cancel_buf[1] = MSG_ORDER_CANCEL; // 消息类型

    // Order ID (假设从第 2 个字节开始)
    cancel_buf[2..10].copy_from_slice(&args.order_id.to_be_bytes());
    
    // 2. 计算 Checksum 并放置
    cancel_buf[0] = calculate_checksum(&cancel_buf);

    // 3. 发送消息
    send_message(socket, trade_addr, &cancel_buf)?;

    // 4. 打印结果
    println!("--- Order Cancel Request (Sent to {}) ---", trade_addr);
    println!("Order ID to Cancel: {}", args.order_id);
    println!("Serialized Message ({} bytes): {:?}", MESSAGE_TOTAL_SIZE, cancel_buf);
    
    Ok(())
}


fn receive_broadcasts(listener_socket: UdpSocket) -> Result<(), String> {
    println!("\n=============================================");
    println!("Listening for broadcast messages...");
    println!("Press Ctrl+C to stop...");
    println!("=============================================");

    // Buffer capable of holding 40 messages
    let mut buf = [0u8; MESSAGE_TOTAL_SIZE * 40]; 

    loop {
        match listener_socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                // Slice the buffer to the actual received length
                let received_data = &buf[..len];

                // Split the received data into chunks of MESSAGE_TOTAL_SIZE
                let chunks = received_data.chunks_exact(MESSAGE_TOTAL_SIZE);
                
                // Capture any trailing bytes that don't form a full message
                let remainder = chunks.remainder();
                if !remainder.is_empty() {
                    eprintln!(
                        "[{}] Warning: Received incomplete trailing data ({} bytes)", 
                        src, remainder.len()
                    );
                }

                // Iterate through each complete message chunk
                for (index, chunk) in chunks.enumerate() {
                    // 1. Verify Checksum for the specific chunk
                    if calculate_checksum(chunk) != chunk[0] {
                        eprintln!("[{}] Message #{} checksum failed, skipping", src, index+1);
                        continue;
                    }

                    // 2. Decode the specific message chunk
                    match decode_broadcast_message(chunk) {
                        Ok(decoded_msg) => {
                            println!("[{}] Message #{}: {}", src, index+1, decoded_msg);
                        },
                        Err(e) => {
                            eprintln!("[{}] Message #{} decoding error: {}", src, index+1, e);
                        }
                    }
                }
            }
            Err(e) => {
                // Ignore non-fatal interruption errors
                if e.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(format!("Socket receive error: {}", e));
            }
        }
    }
}