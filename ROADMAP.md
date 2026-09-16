# Roadmap — `quipu-ipfs`

> Red P2P descentralizada en Rust. Capa genérica de identidad, almacenamiento
> y transporte sobre la cual se construyen aplicaciones — empezando por
> mensajería encriptada, y a futuro redes sociales y otras apps descentralizadas.
> Visión a 10 años: ser la infraestructura de red de un modelo de IA
> descentralizado que no le pertenece a ninguna corporación ni gobierno.

---

## Principio rector

El núcleo de la red **no sabe qué es un "mensaje" o un "post"** — solo sabe
firmar, almacenar y rutear objetos de datos genéricos entre identidades
criptográficas. La mensajería es la primera aplicación construida sobre esa
base, no algo especial integrado en el core. Esta decisión es deliberadamente
más lenta de construir a corto plazo, a cambio de no tener que reescribir el
núcleo cuando lleguen apps futuras (redes sociales, comunicación P2P
genérica, eventualmente la capa de IA distribuida).

---

## Fase 0 — Fundamentos genéricos (corto plazo)

Objetivo: validar que la arquitectura en capas funciona, con las piezas más
simples y testeables posible, antes de meterle complejidad de red o cifrado.

- [x] `quipu-identity` — identidad criptográfica (keypair ed25519, firmar/verificar)
- [x] `quipu-store` — `SignedObject` genérico + trait `ObjectStore` (implementación en memoria)
- [x] `quipu-net` — transporte P2P mínimo: descubrimiento por mDNS en red local + request/response de objetos por hash
- [x] Demo funcional: dos nodos en la misma red local se descubren y se piden un objeto

**Criterio de salida de esta fase:** `cargo build` + `cargo test` pasan en todos los crates, y el demo de descubrimiento mDNS funciona entre dos procesos reales (no solo en teoría).

## Fase 1 — Mensajería encriptada (primer producto entregable)

Objetivo: primer caso de uso real con valor propio, sin depender de que la
parte de IA exista.

- [ ] `quipu-crypto` — cifrado punto a punto (Diffie-Hellman + cifrado simétrico), separado del store genérico a propósito
- [ ] `quipu-messaging` — chats sobre `quipu-store` + `quipu-crypto` + `quipu-net`
- [ ] CLI mínima: dos nodos en red local se mandan un mensaje cifrado de extremo a extremo
- [ ] Entrega offline básica (el mensaje se guarda en el store del remitente hasta que el destino esté disponible)

**Criterio de salida:** alguien fuera del proyecto puede instalar, correr dos nodos, y mandar/recibir un mensaje cifrado sin tocar código.

## Fase 2 — Red real (más allá de la red local)

Objetivo: que la red funcione entre nodos que no están en la misma LAN —
condición necesaria para que esto sea una red P2P real y no un demo de
laboratorio.

- [ ] Kademlia DHT para descubrimiento de nodos por internet (no solo mDNS)
- [ ] Persistencia en disco del store (hoy es solo en memoria)
- [ ] NAT traversal / relay para nodos detrás de redes domésticas
- [ ] Forward secrecy real (Double Ratchet, como Signal/Matrix) reemplazando el cifrado simplificado de la Fase 1

**Criterio de salida:** dos nodos en redes domésticas distintas, sin IP pública, logran descubrirse y comunicarse.

## Fase 3 — Plataforma para terceros

Objetivo: habilitar que otras apps (redes sociales, foros, lo que sea) se
construyan sobre la misma capa base, sin tocar el núcleo.

- [ ] Documentar `quipu-store` como SDK: cómo definir un nuevo `kind` de objeto
- [ ] Ejemplo de referencia en `apps/`: una app simple no relacionada a mensajería (ej. un feed público de posts firmados, sin cifrado) que demuestre que el core es realmente agnóstico
- [ ] Permisos/reglas de visibilidad genéricas (quién puede leer/escribir qué `kind`), necesarias para que apps públicas y privadas convivan en la misma red

**Criterio de salida:** una app de ejemplo distinta a mensajería corre sobre la red sin modificar `quipu-identity`, `quipu-store` ni `quipu-net`.

## Fase 4 — Integración con `compresor`

Objetivo: la red empieza a transportar pesos de modelos de IA, conectando
con el trabajo de investigación en compresión que avanza en paralelo
(repo `compresor`).

- [ ] Definir `kind="model-weights"` o similar sobre `quipu-store`
- [ ] Distribución de modelos pequeños ya existentes (curados, no necesariamente comprimidos por Wiracocha Labs) como primer caso de uso real
- [ ] Sincronización de deltas entre versiones de modelo (si la investigación de `compresor` en ese nicho avanza)

**Nota:** esta fase no espera a que `compresor` "termine" — avanza con lo que esté disponible en cada momento, sea un modelo curado de terceros o una técnica propia ya validada.

## Fase 5 — Federated Learning sobre la red

Objetivo: nodos entrenando/actualizando un modelo de forma distribuida,
no solo sirviendo inferencia.

- [ ] Protocolo de agregación de actualizaciones entre nodos
- [ ] Mecanismos de verificación de integridad (evitar que un nodo malicioso envenene el modelo compartido)

## Fase 6 — Modelo de IA descentralizado corriendo en hardware modesto (horizonte ~10 años)

Objetivo final del proyecto: el modelo vive en la red, no en un datacenter.

- [ ] Validación continua de qué hardware es viable en cada momento (no asumir Raspberry Pi desde el día uno — medir, no proyectar)
- [ ] Nodos heterogéneos: algunos aportan más cómputo (servidores, desktops potentes), otros solo participan en almacenamiento/distribución
- [ ] El objetivo es a 10 años, pero la investigación y la infraestructura se construyen desde ahora — no se espera a que se cumpla el plazo para empezar

---

## Cómo leer este roadmap

- Las fases están en orden de dependencia técnica, no necesariamente de calendario estricto — Fase 1 y el trabajo de `compresor` pueden avanzar en paralelo si los recursos lo permiten.
- Cada fase tiene un criterio de salida verificable (algo que se puede probar, no solo "sentir que está listo").
- Este documento se actualiza con evidencia, no con intención: si una fase revela que una suposición anterior era incorrecta (como ya pasó con el supuesto de Raspberry Pi corriendo modelos grandes hoy), el roadmap se corrige, no se fuerza la suposición original.
