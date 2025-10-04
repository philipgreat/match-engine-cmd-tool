// src/network.rs

use std::net::{UdpSocket, ToSocketAddrs};
use std::net::{ IpAddr, Ipv4Addr,SocketAddr};
use socket2::{Domain, Protocol, Socket, Type};


// 创建并配置发送用的 UDP Socket


// 通用的 Multicast/Unicast 发送函数
pub fn send_message(socket: &UdpSocket, addr: &str, message: &[u8]) -> Result<(), String> {
    match socket.send_to(message, addr) {
        Ok(bytes_sent) => {
            if bytes_sent == message.len() {
                Ok(())
            } else {
                Err(format!("Partial send: {} of {} bytes sent.", bytes_sent, message.len()))
            }
        }
        Err(e) => Err(format!("Failed to send message to {}: {}", addr, e)),
    }
}


// ... (之前的 send_message 和 create_multicast_socket 保持不变)
pub fn create_multicast_listener(addr: &str) -> Result<UdpSocket, String> {
    let mut addrs = addr.to_socket_addrs()
        .map_err(|e| format!("Invalid multicast address format: {}", e))?;

    let socket_addr = addrs.next().ok_or("No address found")?;
    let ip = socket_addr.ip();
    let port = socket_addr.port();

    if !ip.is_multicast() {
        return Err(format!("Address {} is not a multicast address.", ip));
    }

    // 使用 socket2::Socket 进行底层配置
    let domain = if ip.is_ipv4() { Domain::IPV4 } else { Domain::IPV6 };
    let socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))
        .map_err(|e| format!("Failed to create socket: {}", e))?;

    // 1. 设置 SO_REUSEADDR
    socket.set_reuse_address(true)
        .map_err(|e| format!("Failed to set SO_REUSEADDR: {}", e))?;
    
    // 2. 设置 SO_REUSEPORT（在部分 Unix 系统上推荐）

    
    let multicast_addr = addr.parse::<SocketAddr>().unwrap();

    let bind_addr = socket2::SockAddr::from(multicast_addr);


    // 3. 绑定到 0.0.0.0:port
    socket.bind(&bind_addr)
        .map_err(|e| format!("Failed to bind listener socket to port {}: {}", port, e))?;




    // 4. 加入组播组
    if let IpAddr::V4(multicast_ip) = ip {
        socket.join_multicast_v4(&multicast_ip, &Ipv4Addr::UNSPECIFIED)
            .map_err(|e| format!("Failed to join multicast group {}: {}", multicast_ip, e))?;
    } else {
        return Err("IPv6 multicast not implemented in listener setup.".to_string());
    }

    // 5. 转换为 std::net::UdpSocket
    let listener: UdpSocket = socket.into();
    Ok(listener)
}