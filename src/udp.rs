use std::{io, net::SocketAddr};

use doip_codec::EncodeError;
use doip_definitions::{
    header::{DoipPayload, DoipVersion, PayloadType},
    message::{
        ActionCode, EntityStatusResponse, GenericNack, NackCode, PowerInformationResponse,
        SyncStatus, VehicleAnnouncementMessage, VehicleIdentificationRequestEid,
        VehicleIdentificationRequestVin,
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

            let _ = match res.header.payload_type {
                PayloadType::VehicleIdentificationRequest => {
                    self.send_vehicle_announcement(addr).await
                }
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
            };
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
