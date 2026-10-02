# 🌐 quipu-ipfs

> Red descentralizada construida desde cero en Rust, inspirada en IPFS. Infraestructura P2P segura, modular y eficiente — diseñada para ser la base de una IA que no le pertenece a nadie.

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Language: Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Status: Fase 0](https://img.shields.io/badge/Status-Fase%200-yellow.svg)]()
[![Org: Wiracocha Labs](https://img.shields.io/badge/Org-Wiracocha%20Labs-purple.svg)](https://github.com/wiracocha-labs)

---

## 🧭 ¿Por qué existe este proyecto?

Todos los modelos de IA más poderosos del mundo pertenecen a alguien:

```
GPT-4    → Microsoft / OpenAI   → USA
Gemini   → Google               → USA
DeepSeek → ByteDance            → China
Grok     → xAI / Elon Musk      → USA
```

Pueden censurarlos. Pueden apagarlos. Pueden sesgarlo. Pueden venderte con ellos.

**quipu-ipfs** es la infraestructura de red para construir algo diferente: un modelo de IA descentralizado, de código abierto, que no le pertenece a ninguna corporación ni gobierno. Un modelo que corre sobre nodos distribuidos alrededor del mundo — incluyendo hardware modesto.

El nombre *Quipu* viene del sistema de registro de información de los Andes prehispánicos: información distribuida, sin servidor central, que pertenecía a todos.

---

## 🧱 Principio rector

El núcleo de la red **no sabe qué es un "mensaje" o un "post"** — solo sabe firmar, almacenar y rutear objetos de datos genéricos entre identidades criptográficas. La mensajería es la primera aplicación construida sobre esa base, no algo especial integrado en el core.

Esta decisión es deliberadamente más lenta de construir a corto plazo, a cambio de no tener que reescribir el núcleo cuando lleguen apps futuras.

---

## 🏗️ Estructura del workspace

```
quipu-ipfs/
└── crates/
    ├── quipu-identity   → identidad criptográfica (keypair ed25519, firmar/verificar)
    ├── quipu-store      → SignedObject genérico + trait ObjectStore (memoria por ahora)
    ├── quipu-net        → transporte P2P: mDNS + request/response de objetos por hash
    └── quipu-node       → binario demo: dos nodos en LAN se descubren e intercambian objetos
```

Las aplicaciones (mensajería cifrada, feeds, distribución de modelos) se construyen **sobre** estas capas — el core nunca interpreta el `kind` de un objeto.

---

## 🚀 Demo funcional (Fase 0)

Dos nodos en la misma red local se descubren por mDNS y se piden un objeto por su hash:

```bash
cargo build

# Terminal A — publica un objeto firmado y lo sirve
cargo run -p quipu-node -- put "hola quipu"

# Terminal B — lo pide por el hash que imprimió A
cargo run -p quipu-node -- get <HASH>
```

---

## 🔭 Visión a 10 años

El objetivo final: un modelo de IA que vive en la red, no en un datacenter — sin dueño corporativo, accesible desde hardware modesto.

Las fases concretas (con criterios de salida verificables) están en [ROADMAP.md](./ROADMAP.md): fundamentos genéricos → mensajería cifrada → red real en internet → plataforma para terceros → distribución de modelos → federated learning.

El roadmap se actualiza con evidencia, no con intención: si una medición contradice una suposición, el roadmap se corrige.

---

## ⚙️ Stack técnico

| Componente | Tecnología | Por qué |
|---|---|---|
| Lenguaje | **Rust** | Memoria segura, rendimiento cercano a C, ideal para nodos 24/7 |
| P2P | **libp2p** | mDNS hoy; Kademlia DHT y NAT traversal en Fase 2 sin reescribir |
| Identidad | **ed25519** (dalek) | Firmas rápidas y claves chicas |
| Hash de contenido | **blake3** | Direccionamiento de objetos por contenido |
| Async runtime | **Tokio** | El estándar de async en Rust |
| Serialización | **serde + postcard** | Objetos binarios compactos por la red |

---

## 📊 Estado actual

**Fase 0 — Fundamentos genéricos: completa** ✅

- [x] `quipu-identity` — keypair ed25519, firmar/verificar
- [x] `quipu-store` — `SignedObject` + `ObjectStore` en memoria
- [x] `quipu-net` — descubrimiento mDNS + request/response por hash
- [x] Demo funcional entre dos procesos reales
- [x] `cargo build` + `cargo test` pasan (15 tests)

**Siguiente: Fase 1** — mensajería cifrada (`quipu-crypto` + `quipu-messaging`).

---

## 🤝 Cómo contribuir

Este es un proyecto a largo plazo, de código abierto, construido por voluntarios que creen que la infraestructura de IA no debería pertenecer a nadie en particular.

### Áreas donde más se necesita ayuda

- **Rust / sistemas distribuidos** → protocolo de comunicación, DHT
- **Criptografía** → cifrado end-to-end, Double Ratchet (Fase 2)
- **Networking** → NAT traversal, descubrimiento de nodos
- **DevOps / embedded** → tests en hardware modesto
- **Documentación** → hacer el proyecto accesible a más personas

### Para empezar

```bash
git clone https://github.com/wiracocha-labs/quipu-ipfs.git
cd quipu-ipfs
cargo build && cargo test
```

Revisa los [Issues abiertos](https://github.com/wiracocha-labs/quipu-ipfs/issues) o el [ROADMAP](./ROADMAP.md) para ver en qué puedes ayudar.

---

## 🧩 Ecosistema Wiracocha Labs

| Repositorio | Descripción | Estado |
|---|---|---|
| **quipu-ipfs** | Red P2P descentralizada *(este repo)* | 🔨 Fase 0 completa |
| **[chaka](https://github.com/wiracocha-labs/chaka)** | Investigación: compresión de deltas entre versiones de modelo | 🔬 Investigando |
| **[yachay](https://github.com/wiracocha-labs/yachay)** | Recomendador de modelos locales según hardware | ✅ v0.1.0 |
| **[chasqui](https://github.com/wiracocha-labs/chasqui-app)** | Plataforma de comunicación descentralizada (primer producto) | 🔨 En desarrollo |

---

## 📜 Licencia

Este proyecto está licenciado bajo [GNU Affero General Public License v3.0](LICENSE).

Esto significa que cualquier modificación o uso en producción también debe ser de código abierto. Nadie puede tomar este trabajo y cerrarlo.

---

## 🌍 Sobre Wiracocha Labs

Organización de investigación y desarrollo open source enfocada en descentralización, privacidad y nuevas formas de comunicación digital.

[GitHub](https://github.com/wiracocha-labs) · [Discussions](https://github.com/wiracocha-labs/quipu-ipfs/discussions)
