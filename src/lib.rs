// #![warn(missing_docs)]

mod config;
mod tcp;
mod udp;

pub use crate::config::ServerConfig;
pub use crate::udp::UdpServer;

#[cfg(test)]
mod test_lib {
    use crate::{config::ServerConfig, udp::UdpServer};
    use doip_definitions::{
        definitions::{DOIP_ENTITY_STATUS_RESPONSE_MCTS_LEN, DOIP_ENTITY_STATUS_RESPONSE_NCTS_LEN},
        header::{DoipPayload, DoipVersion, PayloadType},
        message::{
            ActionCode, GenericNack, NackCode, NodeType, PowerMode, SyncStatus,
            VehicleAnnouncementMessage, VehicleIdentificationRequest,
        },
    };
    use doip_sockets::udp::UdpSocket;
    use std::net::ToSocketAddrs;

    #[tokio::test]
    async fn test_vehicle_identification_request() -> Result<(), Box<dyn std::error::Error>> {
        let config = ServerConfig {
            address: "127.0.0.1:8080".to_socket_addrs().unwrap().next().unwrap(),
            protocol_version: DoipVersion::Iso13400_2012,

            vin: [0; 17],
            gid: [0; 6],
            eid: [0; 6],
            vin_gid_is_synced: true,
            logical_address: [0; 2],
            routing_is_activated: false,

            node_type: NodeType::DoipGateway,
            max_concurrent_sockets: [0; DOIP_ENTITY_STATUS_RESPONSE_MCTS_LEN],
            currently_open_sockets: [0; DOIP_ENTITY_STATUS_RESPONSE_NCTS_LEN],
            max_data_size: [0x00, 0x00, 0xff, 0xff],

            power_mode: PowerMode::Ready,
        };

        // Spawn the listener in a separate task
        tokio::spawn(async move {
            if let Err(e) = UdpServer::start(config).await.unwrap().run().await {
                eprintln!("Error in UDP listener: {}", e);
            }
        });

        // Create a client socket to send and receive messages
        let mut client_socket = UdpSocket::bind("127.0.0.1:0").await?;
        let msg = VehicleIdentificationRequest {};

        // Send a message to the listener
        client_socket.send(msg, config.address).await?;

        let res = client_socket.recv().await;

        let (msg, addr) = res.unwrap().unwrap();
        assert!(msg.header.protocol_version == config.protocol_version);
        assert!(msg.payload.payload_type() == PayloadType::VehicleAnnouncementMessage);
        assert!(addr == config.address);

        drop(client_socket);

        Ok(())
    }

    #[tokio::test]
    async fn test_bad_request() -> Result<(), Box<dyn std::error::Error>> {
        let config = ServerConfig {
            address: "127.0.0.1:8082".to_socket_addrs().unwrap().next().unwrap(),
            protocol_version: DoipVersion::Iso13400_2012,

            vin: [0; 17],
            gid: [0; 6],
            eid: [0; 6],
            vin_gid_is_synced: true,
            logical_address: [0; 2],
            routing_is_activated: false,

            node_type: NodeType::DoipGateway,
            max_concurrent_sockets: [0; DOIP_ENTITY_STATUS_RESPONSE_MCTS_LEN],
            currently_open_sockets: [0; DOIP_ENTITY_STATUS_RESPONSE_NCTS_LEN],
            max_data_size: [0x00, 0x00, 0xff, 0xff],

            power_mode: PowerMode::Ready,
        };

        // Spawn the listener in a separate task
        tokio::spawn(async move {
            if let Err(e) = UdpServer::start(config).await.unwrap().run().await {
                eprintln!("Error in UDP listener: {}", e);
            }
        });

        // Create a client socket to send and receive messages
        let mut client_socket = UdpSocket::bind("127.0.0.1:0").await?;
        let msg = VehicleAnnouncementMessage {
            vin: config.vin,
            logical_address: config.logical_address,
            eid: config.eid,
            gid: config.gid,
            further_action: ActionCode::NoFurtherActionRequired,
            vin_gid_sync: Some(SyncStatus::VinGidSynchronized),
        };

        // Send a message to the listener
        client_socket.send(msg, config.address).await?;

        let res = client_socket.recv().await;

        let (msg, addr) = res.unwrap().unwrap();
        assert!(msg.header.protocol_version == config.protocol_version);
        assert!(msg.payload.payload_type() == PayloadType::GenericNack);
        assert!(
            GenericNack::from_bytes(&msg.payload.to_bytes())
                .unwrap()
                .nack_code
                == NackCode::IncorrectPatternFormat
        );
        assert!(addr == config.address);

        drop(client_socket);

        Ok(())
    }
}
