use std::{io, net::SocketAddr};

use doip_codec::EncodeError;
use doip_definitions::{
    header::{DoipPayload, DoipVersion, PayloadType},
    message::{
        ActionCode, DoipMessage, EntityStatusResponse, GenericNack, NackCode,
        PowerInformationResponse, SyncStatus, VehicleAnnouncementMessage,
        VehicleIdentificationRequestEid, VehicleIdentificationRequestVin,
    },
};
use doip_sockets::udp::UdpSocket;

use crate::config::ServerConfig;
pub struct UdpServer {
    config: ServerConfig,
    socket: UdpSocket,
}

impl UdpServer {
    pub async fn start(config: ServerConfig) -> io::Result<Self> {
        let mut socket = UdpSocket::bind(config.address).await?;
        socket.set_protocol_version(DoipVersion::Iso13400_2012);

        Ok(UdpServer { config, socket })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Listening on: {}",
            self.socket.get_socket_ref().local_addr()?
        );
        loop {
            let (res, addr) = self.socket.recv().await.unwrap().unwrap();

            let _ = self.handle_message(res, addr).await;
        }
    }

    async fn handle_message(
        &mut self,
        res: DoipMessage,
        addr: SocketAddr,
    ) -> Result<(), EncodeError> {
        match res.header.payload_type {
            PayloadType::VehicleIdentificationRequest => self.send_vehicle_announcement(addr).await,
            PayloadType::VehicleIdentificationRequestEid => {
                match VehicleIdentificationRequestEid::from_bytes(&res.payload.to_bytes()) {
                    Ok(vir_eid) => {
                        if vir_eid.eid == self.config.eid {
                            self.send_vehicle_announcement(addr).await
                        } else {
                            Ok(())
                        }
                    }
                    Err(_) => Ok(()),
                }
            }
            PayloadType::VehicleIdentificationRequestVin => {
                match VehicleIdentificationRequestVin::from_bytes(&res.payload.to_bytes()) {
                    Ok(vir_vin) => {
                        if vir_vin.vin == self.config.vin {
                            self.send_vehicle_announcement(addr).await
                        } else {
                            Ok(())
                        }
                    }
                    Err(_) => Ok(()),
                }
            }
            PayloadType::EntityStatusRequest => self.send_entity_status_response(addr).await,
            PayloadType::PowerInformationRequest => {
                self.send_power_information_response(addr).await
            }
            _ => self.send_generic_nack(addr).await,
        }
    }

    async fn send_vehicle_announcement(&mut self, addr: SocketAddr) -> Result<(), EncodeError> {
        self.socket
            .send(
                VehicleAnnouncementMessage {
                    vin: self.config.vin,
                    logical_address: self.config.logical_address,
                    eid: self.config.eid,
                    gid: self.config.gid,
                    further_action: match self.config.routing_is_activated {
                        true => ActionCode::NoFurtherActionRequired,
                        false => ActionCode::RoutingActivationRequired,
                    },
                    vin_gid_sync: match self.config.vin_gid_is_synced {
                        true => Some(SyncStatus::VinGidSynchronized),
                        false => Some(SyncStatus::VinGidNotSynchronised),
                    },
                },
                addr,
            )
            .await
    }

    async fn send_generic_nack(&mut self, addr: SocketAddr) -> Result<(), EncodeError> {
        self.socket
            .send(
                GenericNack {
                    nack_code: NackCode::IncorrectPatternFormat,
                },
                addr,
            )
            .await
    }

    async fn send_entity_status_response(&mut self, addr: SocketAddr) -> Result<(), EncodeError> {
        self.socket
            .send(
                EntityStatusResponse {
                    node_type: self.config.node_type,
                    max_concurrent_sockets: self.config.max_concurrent_sockets,
                    currently_open_sockets: self.config.currently_open_sockets,
                    max_data_size: self.config.max_data_size,
                },
                addr,
            )
            .await
    }

    async fn send_power_information_response(
        &mut self,
        addr: SocketAddr,
    ) -> Result<(), EncodeError> {
        self.socket
            .send(
                PowerInformationResponse {
                    power_mode: self.config.power_mode,
                },
                addr,
            )
            .await
    }
}

#[cfg(test)]
mod test_udp {
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
