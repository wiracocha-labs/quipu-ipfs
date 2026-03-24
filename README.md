# 🌐 quipu-ipfs

> Red descentralizada construida desde cero en Rust, inspirada en IPFS. Infraestructura P2P segura, modular y eficiente — diseñada para ser la base de una IA que no le pertenece a nadie.

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Language: Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Status: En construcción](https://img.shields.io/badge/Status-En%20construcción-yellow.svg)]()
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

**quipu-ipfs** es la infraestructura de red para construir algo diferente: un modelo de IA descentralizado, de código abierto, que no le pertenece a ninguna corporación ni gobierno. Un modelo que corre sobre nodos distribuidos alrededor del mundo — incluyendo hardware modesto como Raspberry Pi.

El nombre *Quipu* viene del sistema de registro de información de los Andes prehispánicos: información distribuida, sin servidor central, que pertenecía a todos.

---

## 🏗️ ¿Qué construye este repositorio?

Una red P2P en Rust con los siguientes componentes:

```
quipu-ipfs/
├── Descubrimiento de nodos        → cómo los nodos se encuentran entre sí
├── DHT (Distributed Hash Table)   → corazón del sistema, como IPFS
├── Almacenamiento distribuido     → chunks de datos repartidos entre nodos
├── Protocolo de comunicación      → TCP/UDP entre nodos
└── Integración con compresión     → preparado para wiracocha-compress
```

Este repositorio es **la capa de red**. Es la base sobre la que correrán los modelos de IA distribuidos.

---

## 🔭 Visión a 10 años

```
2025-2026  →  Protocolo de red base y comunicación entre nodos
2026-2028  →  DHT estable, almacenamiento distribuido funcional
2028-2030  →  Integración con wiracocha-compress y capa de ML
2030-2032  →  Federated Learning sobre la red
2032-2035  →  Modelo de IA descentralizado corriendo en nodos Raspberry Pi
```

El timing no es accidental: en 10 años, el hardware como Raspberry Pi tendrá la capacidad computacional necesaria para correr modelos medianos. Estamos construyendo la infraestructura hoy.

---

## ⚙️ Stack técnico

| Componente | Tecnología | Por qué |
|---|---|---|
| Lenguaje | **Rust** | Memoria segura, rendimiento cercano a C, ideal para nodos 24/7 |
| P2P base | **libp2p** | Usado en Ethereum y Polkadot, probado en producción |
| Async runtime | **Tokio** | El estándar de async en Rust |
| Serialización | **serde** | Rápido y flexible |
| Compresión | **wiracocha-compress** *(próximamente)* | Compresor optimizado para pesos de modelos IA |

---

## 🚀 Estado actual

- [x] Repositorio creado y licenciado (AGPL-3.0)
- [ ] Estructura base del proyecto Rust
- [ ] Comunicación TCP entre dos nodos
- [ ] Protocolo de descubrimiento de nodos
- [ ] DHT básico funcional
- [ ] Almacenamiento de chunks distribuido
- [ ] Integración con wiracocha-compress
- [ ] Testnet con nodos Raspberry Pi

---

## 🤝 Cómo contribuir

Este es un proyecto a largo plazo, de código abierto, construido por voluntarios que creen que la infraestructura de IA no debería pertenecer a nadie en particular.

### Áreas donde más se necesita ayuda

- **Rust / sistemas distribuidos** → protocolo de comunicación, DHT
- **Criptografía** → hashes, verificación de integridad entre nodos
- **Networking** → optimización de latencia, descubrimiento de nodos
- **DevOps / embedded** → tests en Raspberry Pi y hardware modesto
- **Documentación** → hacer el proyecto accesible a más personas

### Para empezar

```bash
# Clona el repositorio
git clone https://github.com/wiracocha-labs/quipu-ipfs.git
cd quipu-ipfs

# Asegúrate de tener Rust instalado
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Compila el proyecto
cargo build
```

Revisa los [Issues abiertos](https://github.com/wiracocha-labs/quipu-ipfs/issues) para ver en qué puedes ayudar. Si tienes preguntas, abre una [Discussion](https://github.com/wiracocha-labs/quipu-ipfs/discussions).

---

## 🧩 Ecosistema Wiracocha Labs

Este repositorio es parte de un proyecto más grande:

| Repositorio | Descripción | Estado |
|---|---|---|
| **quipu-ipfs** | Red P2P descentralizada *(este repo)* | 🔨 En construcción |
| **wiracocha-compress** | Compresor optimizado para pesos de modelos IA | 📋 Planeado |

---

## 📜 Licencia

Este proyecto está licenciado bajo [GNU Affero General Public License v3.0](LICENSE).

Esto significa que cualquier modificación o uso en producción también debe ser de código abierto. Nadie puede tomar este trabajo y cerrarlo.

---

## 🌍 Sobre Wiracocha Labs

Organización de investigación y desarrollo open source enfocada en descentralización, privacidad y nuevas formas de comunicación digital.

[GitHub](https://github.com/wiracocha-labs) · [Discussions](https://github.com/wiracocha-labs/quipu-ipfs/discussions)
