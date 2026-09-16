//! Transporte P2P mínimo de la red quipu.
//!
//! Fase 0: descubrimiento por mDNS en la red local + request/response de
//! `SignedObject` por hash de contenido, sobre libp2p (TCP + Noise + Yamux).
//!
//! Este crate tampoco interpreta los objetos que transporta: mueve bytes
//! firmados entre peers. La Fase 2 agrega Kademlia DHT y NAT traversal
//! sobre esta misma estructura.

use async_trait::async_trait;
use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use futures::StreamExt;
use libp2p::{
    mdns, noise,
    request_response::{self, ProtocolSupport, ResponseChannel},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, StreamProtocol, Swarm,
};
use quipu_store::{ObjectHash, ObjectStore, SignedObject};
use serde::{Deserialize, Serialize};
use std::io;
use std::sync::Arc;
use std::time::Duration;

// Re-exports para que los consumidores no necesiten depender de libp2p.
pub use libp2p::{Multiaddr as NetAddr, PeerId as NetPeerId};

/// Nombre del protocolo de intercambio de objetos en la red.
pub const OBJECT_PROTOCOL: &str = "/quipu/objects/1.0.0";

/// Tamaño máximo de un mensaje request/response (32 MiB).
/// Objetos más grandes necesitarán chunking — fuera de scope de Fase 0.
const MAX_MESSAGE_LEN: u64 = 32 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum NetError {
    #[error("error configurando el transporte: {0}")]
    Transport(String),
    #[error("error construyendo el behaviour: {0}")]
    Behaviour(String),
    #[error("error al escuchar en la dirección: {0}")]
    Listen(String),
}

/// Pedido de un objeto por hash de contenido.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectRequest {
    pub hash: ObjectHash,
}

/// Respuesta: el objeto si el peer lo tiene, `None` si no.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectResponse {
    pub object: Option<SignedObject>,
}

/// Serialización de mensajes: postcard (binario compacto, basado en serde).
#[derive(Debug, Clone, Default)]
pub struct ObjectCodec;

fn invalid_data(err: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, err.to_string())
}

async fn read_capped<T: AsyncRead + Unpin + Send>(io: &mut T) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    io.take(MAX_MESSAGE_LEN + 1).read_to_end(&mut buf).await?;
    if buf.len() as u64 > MAX_MESSAGE_LEN {
        return Err(invalid_data("el mensaje excede el tamaño máximo"));
    }
    Ok(buf)
}

#[async_trait]
impl request_response::Codec for ObjectCodec {
    type Protocol = StreamProtocol;
    type Request = ObjectRequest;
    type Response = ObjectResponse;

    async fn read_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        postcard::from_bytes(&read_capped(io).await?).map_err(invalid_data)
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        postcard::from_bytes(&read_capped(io).await?).map_err(invalid_data)
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        request: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&postcard::to_stdvec(&request).map_err(invalid_data)?)
            .await?;
        io.close().await
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        response: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&postcard::to_stdvec(&response).map_err(invalid_data)?)
            .await?;
        io.close().await
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "QuipuEvent")]
struct QuipuBehaviour {
    mdns: mdns::tokio::Behaviour,
    objects: request_response::Behaviour<ObjectCodec>,
}

/// Evento interno del swarm (unión de los eventos de cada protocolo).
// Enum interno del derive de NetworkBehaviour — el tamaño es normal acá.
#[allow(clippy::large_enum_variant)]
enum QuipuEvent {
    Mdns(mdns::Event),
    Objects(request_response::Event<ObjectRequest, ObjectResponse>),
}

impl From<mdns::Event> for QuipuEvent {
    fn from(e: mdns::Event) -> Self {
        QuipuEvent::Mdns(e)
    }
}

impl From<request_response::Event<ObjectRequest, ObjectResponse>> for QuipuEvent {
    fn from(e: request_response::Event<ObjectRequest, ObjectResponse>) -> Self {
        QuipuEvent::Objects(e)
    }
}

/// Eventos que el nodo expone hacia afuera.
/// `PeerId` acá es la identidad de red de libp2p — distinta del PeerId
/// criptográfico de `quipu-identity` (autor de objetos).
#[derive(Debug)]
pub enum NodeEvent {
    /// El nodo empezó a escuchar en esta dirección.
    ListeningOn(Multiaddr),
    /// mDNS encontró un peer nuevo en la red local.
    PeerDiscovered { peer: PeerId, addr: Multiaddr },
    /// Un peer dejó de anunciarse por mDNS.
    PeerExpired { peer: PeerId },
    /// Se estableció una conexión con un peer (tras `dial` o inbound).
    Connected { peer: PeerId },
    /// Un peer nos pidió un objeto; `served` indica si lo teníamos.
    InboundRequest {
        from: PeerId,
        hash: ObjectHash,
        served: bool,
    },
    /// Recibimos el objeto que pedimos.
    ObjectReceived { from: PeerId, object: SignedObject },
    /// El peer respondió pero no tenía el objeto.
    ObjectNotFound { from: PeerId },
    /// El pedido saliente falló (peer caído, timeout, etc).
    RequestFailed { peer: PeerId, reason: String },
}

/// Un nodo de la red quipu: swarm de libp2p + un store de objetos.
pub struct Node {
    swarm: Swarm<QuipuBehaviour>,
    store: Arc<dyn ObjectStore>,
}

impl Node {
    /// Crea un nodo con una identidad de red nueva y el store dado.
    pub fn new(store: Arc<dyn ObjectStore>) -> Result<Self, NetError> {
        let swarm = libp2p::SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| NetError::Transport(e.to_string()))?
            .with_behaviour(|key| {
                let local_peer = key.public().to_peer_id();
                Ok(QuipuBehaviour {
                    mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer)?,
                    objects: request_response::Behaviour::new(
                        [(StreamProtocol::new(OBJECT_PROTOCOL), ProtocolSupport::Full)],
                        request_response::Config::default(),
                    ),
                })
            })
            .map_err(|e| NetError::Behaviour(e.to_string()))?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        Ok(Self { swarm, store })
    }

    /// PeerId de red de este nodo.
    pub fn local_peer_id(&self) -> PeerId {
        *self.swarm.local_peer_id()
    }

    /// Empieza a escuchar en todas las interfaces, puerto TCP aleatorio.
    pub fn listen(&mut self) -> Result<(), NetError> {
        self.swarm
            .listen_on("/ip4/0.0.0.0/tcp/0".parse().expect("multiaddr válida"))
            .map_err(|e| NetError::Listen(e.to_string()))?;
        Ok(())
    }

    /// Pide un objeto por hash a un peer ya conectado.
    pub fn request_object(&mut self, peer: PeerId, hash: ObjectHash) {
        self.swarm
            .behaviour_mut()
            .objects
            .send_request(&peer, ObjectRequest { hash });
    }

    /// Inicia una conexión a una dirección concreta (ej. una anunciada por mDNS).
    /// Permite probar cada dirección de un peer en paralelo: una dirección
    /// mala no bloquea a las demás.
    pub fn dial(&mut self, addr: Multiaddr) {
        let _ = self.swarm.dial(addr);
    }

    /// Avanza el loop de eventos hasta que ocurre algo relevante para el
    /// llamador. Los pedidos entrantes se sirven internamente desde el store.
    pub async fn next_event(&mut self) -> NodeEvent {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    return NodeEvent::ListeningOn(address);
                }

                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    return NodeEvent::Connected { peer: peer_id };
                }

                SwarmEvent::Behaviour(QuipuEvent::Mdns(mdns::Event::Discovered(peers))) => {
                    for (peer, addr) in peers {
                        if peer == self.local_peer_id() {
                            continue;
                        }
                        self.swarm.add_peer_address(peer, addr.clone());
                        return NodeEvent::PeerDiscovered { peer, addr };
                    }
                }

                SwarmEvent::Behaviour(QuipuEvent::Mdns(mdns::Event::Expired(peers))) => {
                    for (peer, _addr) in peers {
                        if peer == self.local_peer_id() {
                            continue;
                        }
                        return NodeEvent::PeerExpired { peer };
                    }
                }

                SwarmEvent::Behaviour(QuipuEvent::Objects(request_response::Event::Message {
                    peer,
                    message,
                    ..
                })) => match message {
                    request_response::Message::Request {
                        request, channel, ..
                    } => {
                        let served = self.serve(request.hash, channel);
                        return NodeEvent::InboundRequest {
                            from: peer,
                            hash: request.hash,
                            served,
                        };
                    }
                    request_response::Message::Response { response, .. } => {
                        return match response.object {
                            Some(object) => NodeEvent::ObjectReceived { from: peer, object },
                            None => NodeEvent::ObjectNotFound { from: peer },
                        };
                    }
                },

                SwarmEvent::Behaviour(QuipuEvent::Objects(
                    request_response::Event::OutboundFailure { peer, error, .. },
                )) => {
                    return NodeEvent::RequestFailed {
                        peer,
                        reason: error.to_string(),
                    };
                }

                _ => {}
            }
        }
    }

    /// Responde un pedido entrante buscando en el store local.
    fn serve(&mut self, hash: ObjectHash, channel: ResponseChannel<ObjectResponse>) -> bool {
        let object = self.store.get(&hash);
        let found = object.is_some();
        // Si send_response falla (peer se desconectó), no hay nada que hacer.
        let _ = self
            .swarm
            .behaviour_mut()
            .objects
            .send_response(channel, ObjectResponse { object });
        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quipu_identity::Identity;

    #[test]
    fn request_serializes_roundtrip() {
        let req = ObjectRequest {
            hash: ObjectHash([7u8; 32]),
        };
        let bytes = postcard::to_stdvec(&req).unwrap();
        let back: ObjectRequest = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(back.hash, req.hash);
    }

    #[test]
    fn response_serializes_roundtrip() {
        let obj = SignedObject::new("note", b"hola".to_vec(), &Identity::generate());
        let res = ObjectResponse { object: Some(obj) };
        let bytes = postcard::to_stdvec(&res).unwrap();
        let back: ObjectResponse = postcard::from_bytes(&bytes).unwrap();
        assert!(back.object.unwrap().verify_signature());
    }
}
