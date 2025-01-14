use std::net::SocketAddr;

use doip_definitions::{
    definitions::{
        DOIP_COMMON_EID_LEN, DOIP_COMMON_VIN_LEN, DOIP_ENTITY_STATUS_RESPONSE_MCTS_LEN,
        DOIP_ENTITY_STATUS_RESPONSE_MDS_LEN, DOIP_ENTITY_STATUS_RESPONSE_NCTS_LEN,
        DOIP_VEHICLE_ANNOUNCEMENT_ADDRESS_LEN, DOIP_VEHICLE_ANNOUNCEMENT_GID_LEN,
    },
    header::DoipVersion,
    message::{NodeType, PowerMode},
};

#[derive(Copy, Clone)]
pub struct ServerConfig {
    pub address: SocketAddr,
    pub protocol_version: DoipVersion,

    pub vin: [u8; DOIP_COMMON_VIN_LEN],
    pub gid: [u8; DOIP_VEHICLE_ANNOUNCEMENT_GID_LEN],
    pub eid: [u8; DOIP_COMMON_EID_LEN],
    pub vin_gid_is_synced: bool,
    pub logical_address: [u8; DOIP_VEHICLE_ANNOUNCEMENT_ADDRESS_LEN],
    pub routing_is_activated: bool,

    pub node_type: NodeType,
    pub max_concurrent_sockets: [u8; DOIP_ENTITY_STATUS_RESPONSE_MCTS_LEN],
    pub currently_open_sockets: [u8; DOIP_ENTITY_STATUS_RESPONSE_NCTS_LEN],
    pub max_data_size: [u8; DOIP_ENTITY_STATUS_RESPONSE_MDS_LEN],

    pub power_mode: PowerMode,
}
