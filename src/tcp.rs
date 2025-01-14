use std::io;

use crate::ServerConfig;
use doip_definitions::{
    header::{DoipPayload, PayloadType},
    message::{
        ActivationCode, AliveCheckResponse, DoipMessage, GenericNack, NackCode,
        RoutingActivationRequest, RoutingActivationResponse,
    },
};
use doip_sockets::tcp::{TcpListener, TcpSocket, TcpStream};

pub struct TcpServer {
    config: ServerConfig,
    listener: TcpListener,
}

impl TcpServer {
    pub async fn start(config: ServerConfig) -> io::Result<Self> {
        let sock = TcpSocket::new_v4()?;
        let _ = sock.bind(config.address);

        let listener = sock.listen(1024)?;
        Ok(TcpServer { config, listener })
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Listening on: {}", self.listener.get_ref().local_addr()?);

        loop {
            let (stream, addr) = self.listener.accept().await?;
            println!("New connection from: {}", addr);

            tokio::task::spawn(
                async move { TcpConnection::new(stream, self.config).await.run().await },
            );
        }
    }
}

struct TcpConnection {
    config: ServerConfig,
    stream: TcpStream,
}

impl TcpConnection {
    pub async fn new(stream: TcpStream, config: ServerConfig) -> Self {
        TcpConnection { config, stream }
    }

    pub async fn run(&mut self) {
        while let Some(Ok(res)) = self.stream.read().await {
            println!("Message: {:?}", res.header.payload_type);
            let _ = self.handle_message(res).await;
        }
    }

    async fn handle_message(&mut self, msg: DoipMessage) {
        match msg.header.payload_type {
            PayloadType::RoutingActivationRequest => {
                let routing_req =
                    RoutingActivationRequest::from_bytes(&msg.payload.to_bytes()).unwrap();

                self.send_routing_activation_response(routing_req).await
            }
            PayloadType::AliveCheckRequest => self.send_alive_check_response().await,
            PayloadType::DiagnosticMessage => self.handle_diagnostic_message().await,
            _ => {
                self.send_generic_nack(NackCode::IncorrectPatternFormat)
                    .await
            }
        }
    }

    async fn send_generic_nack(&mut self, nack_code: NackCode) {
        let _ = self.stream.send(GenericNack { nack_code }).await;
    }

    async fn send_routing_activation_response(&mut self, payload: RoutingActivationRequest) {
        let activation_code = match payload.activation_type {
            doip_definitions::message::ActivationType::Default => {
                ActivationCode::SuccessfullyActivated
            }
            _ => ActivationCode::DeniedMissingAuthentication,
        };

        let _ = self
            .stream
            .send(RoutingActivationResponse {
                logical_address: payload.source_address,
                source_address: self.config.logical_address,
                activation_code,
                buffer: [0x00, 0x00, 0x00, 0x00],
            })
            .await;
    }

    async fn send_alive_check_response(&mut self) {
        let _ = self
            .stream
            .send(AliveCheckResponse {
                source_address: self.config.logical_address,
            })
            .await;
    }

    async fn handle_diagnostic_message(&mut self) {}

    async fn send_diagnostic_message(&mut self) {}
    async fn send_diagnostic_message_ack(&mut self) {}
    async fn send_diagnostic_message_nack(&mut self) {}
}

#[cfg(test)]
mod test_tcp {
    use std::net::ToSocketAddrs;

    use doip_definitions::{
        definitions::{DOIP_ENTITY_STATUS_RESPONSE_MCTS_LEN, DOIP_ENTITY_STATUS_RESPONSE_NCTS_LEN},
        header::DoipVersion,
        message::{ActivationType, NodeType, PowerMode, RoutingActivationRequest},
    };
    use doip_sockets::tcp::TcpStream;
    use tokio::join;

    use crate::{ServerConfig, TcpServer};

    #[tokio::test]
    async fn test_read_write() -> Result<(), Box<dyn std::error::Error>> {
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
            if let Err(e) = TcpServer::start(config).await.unwrap().run().await {
                eprintln!("Error in TCP listener: {}", e);
            }
        });

        let payload = RoutingActivationRequest {
            source_address: [0x01, 0x02],
            activation_type: ActivationType::Default,
            buffer: [0x00, 0x00, 0x00, 0x00],
        };

        let handle1 = tokio::spawn(async move {
            let mut stream = TcpStream::connect(config.address).await.unwrap();
            let _ = stream.send(payload).await;
            let _ = stream.send(payload).await;
            let res = stream.read().await;
            dbg!(res.unwrap().unwrap().header.payload_type);
        });

        let handle2 = tokio::spawn(async move {
            let mut stream = TcpStream::connect(config.address).await.unwrap();
            let _ = stream.send(payload).await;
            let _ = stream.send(payload).await;
            let res = stream.read().await;
            dbg!(res.unwrap().unwrap().header.payload_type);
        });

        join!(handle1, handle2).0.unwrap();

        Ok(())
    }
}
