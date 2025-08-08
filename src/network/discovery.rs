use crate::config::Config;
use crate::message::{Message, MessageHandler};
use anyhow::{Context, Result};
use local_ip_address::local_ip;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::UdpSocket;

use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub struct DiscoveryService {
    config: Config,
    socket: UdpSocket,
    message_handler: MessageHandler,
    peer_id: Uuid,
}

impl DiscoveryService {
    pub async fn new(
        mut config: Config,
        event_sender: mpsc::UnboundedSender<crate::message::ChatEvent>,
    ) -> Result<Self> {
        // Use any available port for listening, but still broadcast to the standard port
        let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0); // 0 = any available port
        
        let socket = UdpSocket::bind(&bind_addr)
            .await
            .context("Failed to bind UDP socket for discovery")?;
        
        socket.set_broadcast(true)
            .context("Failed to enable broadcast on UDP socket")?;
        
        let actual_addr = socket.local_addr()?;
        info!("Discovery service listening on {}", actual_addr);
        
        let message_handler = MessageHandler::new(config.username.clone(), event_sender);
        let peer_id = message_handler.peer_id();
        
        Ok(Self {
            config,
            socket,
            message_handler,
            peer_id,
        })
    }

    pub async fn start_discovery(self) -> Result<()> {
        info!("Starting peer discovery...");
        
        let config = self.config.clone();
        let peer_id = self.peer_id;
        
        // Create a separate socket for listening on the standard discovery port
        let standard_port_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.discovery_port);
        let listen_socket = match UdpSocket::bind(&standard_port_addr).await {
            Ok(socket) => {
                info!("Listening for discovery messages on standard port {}", config.discovery_port);
                Some(Arc::new(socket))
            }
            Err(_) => {
                info!("Standard discovery port {} already in use, will only broadcast", config.discovery_port);
                None
            }
        };
        
        // Use our own socket for broadcasting
        let broadcast_socket = Arc::new(self.socket);
        let mut message_handler = self.message_handler;
        
        // Start listening task (only if we could bind to standard port)
        let listen_task = if let Some(listen_sock) = listen_socket {
            Some(tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                
                loop {
                    match listen_sock.recv_from(&mut buf).await {
                        Ok((len, addr)) => {
                            let data = &buf[..len];
                            
                            if let Ok(message) = serde_json::from_slice::<Message>(data) {
                                debug!("Received discovery message from {}: {:?}", addr, message);
                                
                                // Don't process our own messages
                                if let Message::Discovery { peer_id: received_peer_id, .. } = &message {
                                    if *received_peer_id == peer_id {
                                        continue;
                                    }
                                }
                                
                                if let Err(e) = message_handler.handle_message(message, addr.ip()) {
                                    warn!("Failed to handle discovery message: {}", e);
                                }
                            } else {
                                debug!("Received invalid discovery message from {}", addr);
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive discovery message: {}", e);
                        }
                    }
                }
            }))
        } else {
            None
        };
        
        // Start broadcasting task
        let broadcast_config = config.clone();
        let broadcast_task = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::send_discovery_broadcast_static(&broadcast_socket, &broadcast_config, peer_id).await {
                    warn!("Failed to send discovery broadcast: {}", e);
                }
            }
        });
        
        // Run tasks concurrently
        if let Some(listen_task) = listen_task {
            tokio::select! {
                result = listen_task => {
                    error!("Discovery listening task ended: {:?}", result);
                }
                result = broadcast_task => {
                    error!("Discovery broadcast task ended: {:?}", result);
                }
            }
        } else {
            // Only run broadcast task
            if let Err(e) = broadcast_task.await {
                error!("Discovery broadcast task panicked: {:?}", e);
            }
        }
        
        Ok(())
    }



    async fn send_discovery_broadcast_static(socket: &Arc<UdpSocket>, config: &Config, peer_id: Uuid) -> Result<()> {
        let message = Message::discovery(
            config.username.clone(),
            config.tcp_port_range.0, // Use first port in range for now
            peer_id,
        );
        
        let data = serde_json::to_vec(&message)
            .context("Failed to serialize discovery message")?;
        
        // Get local network addresses to broadcast to
        let broadcast_addrs = Self::get_broadcast_addresses_static()?;
        
        for addr in broadcast_addrs {
            let target = SocketAddr::new(addr, config.discovery_port);
            
            match socket.send_to(&data, target).await {
                Ok(_) => {
                    debug!("Sent discovery broadcast to {}", target);
                }
                Err(e) => {
                    debug!("Failed to send discovery to {}: {}", target, e);
                }
            }
        }
        
        Ok(())
    }

    fn get_broadcast_addresses_static() -> Result<Vec<IpAddr>> {
        let mut broadcast_addrs = Vec::new();
        
        // Get local IP address
        match local_ip() {
            Ok(local_ip) => {
                match local_ip {
                    IpAddr::V4(ipv4) => {
                        // Generate broadcast address for common private network ranges
                        let octets = ipv4.octets();
                        
                        // For 192.168.x.x networks
                        if octets[0] == 192 && octets[1] == 168 {
                            let broadcast = Ipv4Addr::new(192, 168, octets[2], 255);
                            broadcast_addrs.push(IpAddr::V4(broadcast));
                        }
                        // For 10.x.x.x networks
                        else if octets[0] == 10 {
                            let broadcast = Ipv4Addr::new(10, octets[1], octets[2], 255);
                            broadcast_addrs.push(IpAddr::V4(broadcast));
                        }
                        // For 172.16-31.x.x networks
                        else if octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31 {
                            let broadcast = Ipv4Addr::new(172, octets[1], octets[2], 255);
                            broadcast_addrs.push(IpAddr::V4(broadcast));
                        }
                        
                        info!("Local IP: {}, broadcasting to: {:?}", local_ip, broadcast_addrs);
                    }
                    IpAddr::V6(_) => {
                        warn!("IPv6 not supported for broadcast discovery");
                    }
                }
            }
            Err(e) => {
                warn!("Failed to get local IP: {}", e);
                // Fallback to common broadcast addresses
                broadcast_addrs.push(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 255)));
                broadcast_addrs.push(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 255)));
            }
        }
        
        // Add limited broadcast as fallback
        broadcast_addrs.push(IpAddr::V4(Ipv4Addr::BROADCAST));
        
        Ok(broadcast_addrs)
    }

    pub fn get_peers(&self) -> &std::collections::HashMap<uuid::Uuid, crate::message::Peer> {
        self.message_handler.peers()
    }
}
