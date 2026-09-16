//! Demo de la Fase 0: dos nodos en la misma red local se descubren por mDNS
//! y se piden un objeto por hash.
//!
//! Terminal A:  quipu-node put "hola quipu"   → imprime el hash y queda sirviendo
//! Terminal B:  quipu-node get <HASH>          → descubre el peer, pide el objeto

use anyhow::{bail, Context, Result};
use quipu_identity::Identity;
use quipu_net::{NetPeerId, Node, NodeEvent};
use quipu_store::{MemoryStore, ObjectHash, ObjectStore, SignedObject};
use std::collections::{HashMap, HashSet};
use std::env;
use std::process;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::info;

const GET_TIMEOUT: Duration = Duration::from_secs(60);

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "quipu=info".into()),
        )
        .init();

    let args: Vec<String> = env::args().collect();
    let result = match args.get(1).map(String::as_str) {
        Some("put") => {
            let data = args.get(2).context("falta el contenido: put \"texto\"")?;
            run_put(data.clone()).await
        }
        Some("get") => {
            let hash = args.get(2).context("falta el hash: get <HASH_HEX>")?;
            run_get(hash.clone()).await
        }
        _ => {
            usage();
            process::exit(2);
        }
    };

    if let Err(e) = result {
        eprintln!("error: {e:#}");
        process::exit(1);
    }
    Ok(())
}

fn usage() {
    eprintln!(
        "quipu-node — demo Fase 0\n\n\
         USO:\n\
         \x20 quipu-node put \"texto\"   publica un objeto firmado y lo sirve en la LAN\n\
         \x20 quipu-node get <HASH>      descubre un peer por mDNS y le pide el objeto\n"
    );
}

/// Publica un objeto firmado y queda escuchando para servirlo.
async fn run_put(data: String) -> Result<()> {
    let identity = Identity::generate();
    let object = SignedObject::new("note", data.into_bytes(), &identity);

    let store = Arc::new(MemoryStore::new());
    let hash = store.put(object)?;

    let mut node = Node::new(store.clone())?;
    node.listen()?;

    println!("autor:   {}", identity.peer_id());
    println!("peer id (red): {}", node.local_peer_id());
    println!("hash:    {hash}");
    println!("\nSirviendo. En otra terminal de la misma red local:\n");
    println!("    cargo run -p quipu-node -- get {hash}\n");

    loop {
        match node.next_event().await {
            NodeEvent::ListeningOn(addr) => info!(%addr, "escuchando"),
            NodeEvent::PeerDiscovered { peer, addr } => {
                println!("peer descubierto: {peer} @ {addr}");
            }
            NodeEvent::InboundRequest { from, hash, served } => {
                println!("pedido entrante de {from} por {hash} — servido: {served}");
            }
            NodeEvent::PeerExpired { peer } => println!("peer expiró: {peer}"),
            _ => {}
        }
    }
}

/// Descubre un peer en la LAN y le pide el objeto por hash.
async fn run_get(hash_hex: String) -> Result<()> {
    let hash = ObjectHash::from_hex(&hash_hex)?;
    let store = Arc::new(MemoryStore::new());

    let mut node = Node::new(store)?;
    node.listen()?;

    println!("peer id (red): {}", node.local_peer_id());
    println!("buscando objeto {hash} — esperando descubrir un peer por mDNS...\n");

    const MAX_ATTEMPTS: u32 = 5;
    const RETRY_DELAY: Duration = Duration::from_secs(2);
    let mut attempts: HashMap<NetPeerId, u32> = HashMap::new();
    let mut requested: HashSet<NetPeerId> = HashSet::new();

    let found = timeout(GET_TIMEOUT, async {
        loop {
            match node.next_event().await {
                NodeEvent::PeerDiscovered { peer, addr } => {
                    println!("peer descubierto: {peer} @ {addr} — conectando");
                    // Un peer puede anunciar varias direcciones (LAN, interfaces
                    // virtuales...). Dialear cada una en paralelo evita que una
                    // dirección mala bloquee el descubrimiento.
                    node.dial(addr);
                }
                NodeEvent::Connected { peer } => {
                    if requested.insert(peer) {
                        println!("conectado a {peer} — pidiendo objeto");
                        node.request_object(peer, hash);
                    }
                }
                NodeEvent::ObjectReceived { from, object } => {
                    return Ok::<_, anyhow::Error>((from, object));
                }
                NodeEvent::ObjectNotFound { from } => {
                    println!("{from} no tiene el objeto — sigo esperando otros peers");
                }
                NodeEvent::RequestFailed { peer, reason } => {
                    let n = attempts.entry(peer).or_insert(0);
                    *n += 1;
                    if *n <= MAX_ATTEMPTS {
                        println!(
                            "pedido a {peer} falló ({reason}) — reintento {}/{MAX_ATTEMPTS}",
                            *n - 1
                        );
                        sleep(RETRY_DELAY).await;
                        node.request_object(peer, hash);
                    } else {
                        println!("pedido a {peer} falló definitivamente: {reason}");
                    }
                }
                _ => {}
            }
        }
    })
    .await;

    match found {
        Err(_) => bail!("timeout: ningún peer de la red local tenía el objeto en {GET_TIMEOUT:?}"),
        Ok(Err(e)) => Err(e),
        Ok(Ok((from, object))) => {
            println!("\nobjeto recibido de {from}:");
            println!("  kind:       {}", object.kind);
            println!("  author:     {}", object.author);
            println!("  created_at: {}", object.created_at);
            println!(
                "  signature:  {}",
                if object.verify_signature() {
                    "válida ✓"
                } else {
                    "INVÁLIDA ✗"
                }
            );
            match String::from_utf8(object.data.clone()) {
                Ok(text) => println!("  data:       {text:?}"),
                Err(_) => println!("  data:       {} bytes binarios", object.data.len()),
            }
            Ok(())
        }
    }
}
