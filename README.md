#  Diorama con Raytracing — Archipiélago flotante

**Proyecto 2 · Gráficas por Computadora · Universidad del Valle de Guatemala**
Samuel Antonio Robledo López

Raytracer en tiempo real escrito en **Rust**, implementado desde cero. La matemática vectorial, las rotaciones, las texturas, el skybox, la refracción, la reflexión, las sombras, el bloom y el multihilo usan **solo la librería estándar**. `raylib` se usa únicamente para abrir la ventana, leer el input y mostrar el framebuffer.

##  Video

##  La escena

Un archipiélago flotante estilo Minecraft con un ciclo de día y noche completo:

- **Isla principal:** una casa con ventanas de vidrio, una lámpara de glowstone, árboles, un farol y un pilar de oro. Tiene un **lago que se desborda en una cascada** al vacío, y en su parte inferior asoman **menas de diamante**.
- **Isla inferior:** recibe la cascada en una poza y tiene su propio árbol.
- **Isla del End:** un pilar de obsidiana con un **cristal del End** que gira, flota y emite luz rosa.
- **Isla del Nether:** un **portal** de obsidiana con superficie translúcida, refractiva, emisiva y animada.
- **Dragón del End:** vuela en círculos sobre las islas. Aletea con alas articuladas, mueve la cola y la mandíbula, tiene ojos que brillan y proyecta su sombra sobre la escena.
- **Mobs nocturnos:** al caer la noche aparecen zombies, creepers y un enderman. Deambulan por la isla, esquivan obstáculos, no caen al vacío y desaparecen al amanecer.

##  Rúbrica

| Criterio | Implementación |
|---|---|
| Complejidad (30) | 4 islas, 18 tipos de bloque, cascada, dragón animado de 30 piezas, 5 mobs con IA de deambulación, cristal giratorio, DDA de voxeles, objetos dinámicos con rotación arbitraria (OBB) |
| Atractivo visual (20) | Ciclo día → atardecer → noche → amanecer, bloom HDR, AO por voxel, sombras de color, modo foto con antialiasing y sombras suaves |
| Rotación + zoom (20) | Cámara orbital: rotación con teclado, mouse o auto-rotación; zoom con rueda o Q/E |
| Materiales (5 × 5) | 17 materiales con textura y parámetros propios (tabla abajo) |
| Refracción (10) | Agua (IOR 1.33), vidrio (1.5) y portal (1.2), con Snell, Fresnel (Schlick) y reflexión interna total |
| Reflexión (5) | Oro, obsidiana, agua, vidrio y piedra, con rebotes recursivos |
| Skybox (20) | 3 cubemaps de 6 caras (día, atardecer, noche) generados proceduralmente y mezclados según la hora, con sol y luna dibujados en tiempo real |

##  Materiales

| Material | Albedo | Specular | Transparencia | Reflectividad | IOR | Extra |
|---|---|---|---|---|---|---|
| Pasto | 0.92 | 0.04 | 0 | 0 | — | Textura distinta arriba, a los lados y abajo |
| Tierra | 0.90 | 0.02 | 0 | 0 | — | |
| Piedra | 0.88 | 0.25 | 0 | 0.03 | — | |
| Cobblestone | 0.86 | 0.15 | 0 | 0 | — | |
| Tablones de roble | 0.88 | 0.12 | 0 | 0 | — | |
| Tronco | 0.88 | 0.06 | 0 | 0 | — | Anillos arriba, corteza a los lados |
| Hojas | 0.90 | 0.15 | 0 | 0 | — | Recorte por alpha |
| **Vidrio** | 1.00 | 1.20 | 0.88 | 0.08 | 1.50 | Refracción |
| **Agua** | 1.00 | 1.40 | 0.78 | 0.15 | 1.33 | Refracción, olas, cascada animada |
| Arena | 0.92 | 0.05 | 0 | 0 | — | |
| **Oro** | 0.80 | 1.00 | 0 | 0.45 | — | Reflexión |
| Glowstone | 1.00 | 0.30 | 0 | 0 | — | Emisivo; se enciende de noche |
| **Obsidiana** | 0.90 | 0.90 | 0 | 0.12 | — | Reflexión |
| **Portal del Nether** | 1.00 | 0.60 | 0.60 | 0.05 | 1.20 | Refracción, emisivo, animado |
| Netherrack | 0.90 | 0.05 | 0 | 0 | — | |
| End Stone | 0.92 | 0.08 | 0 | 0 | — | |
| Mena de diamante | 0.88 | 0.80 | 0 | 0.05 | — | Destellos especulares |

##  Controles

| Tecla | Acción |
|---|---|
| Flechas / WASD | Girar la cámara |
| Arrastrar mouse | Girar la cámara |
| Rueda / Q, E | Acercar y alejar |
| R | Giro automático |
| N | Adelantar 6 horas |
| T | Velocidad del tiempo (x1, x4, x0.25) |
| P | Pausar y mejorar imagen (antialiasing y sombras suaves) |
| B | Resplandor (bloom) on/off |
| 1 2 3 4 | Resolución (1 = máxima) |
| F12 | Guardar captura |
| H | Mostrar/ocultar controles |

##  Cómo correrlo

```bash
cargo run --release
```

##  Detalles técnicos

- **Recorrido DDA** (Amanatides & Woo) sobre un grid de 40×28×40.
- **Objetos dinámicos como cajas orientadas (OBB):** el rayo se transforma al espacio local de cada caja. Las piezas se agrupan con esferas envolventes para descartarlas rápido. Usan articulaciones jerárquicas (pivote + rotación) para piernas, brazos, alas y mandíbula.
- **Medios transparentes:** el rayo sabe si está dentro de agua, vidrio o portal, y solo se detiene en interfaces reales.
- **Fresnel de Schlick** con reflexión interna total.
- **Sombras de color:** la luz que atraviesa medios transparentes se tiñe. El dragón y los mobs también proyectan sombra.
- **Ciclo día/noche:** se mezclan 3 cubemaps y se interpolan la luz principal (sol o luna), la luz ambiental y la intensidad de las lámparas.
- **Post-proceso:** pipeline HDR con bloom gaussiano a media resolución, tonemapping con hombro suave y viñeta.
- **Modo foto:** jitter subpixel con secuencia de Halton y jitter de luces, promediando los frames de forma progresiva.
- **Multihilo** con `std::thread::scope`.

##  Estructura

```
src/
├── main.rs      # Ventana, input, ciclo día/noche, HUD
├── camera.rs    # Cámara orbital (rotación + zoom)
├── render.rs    # Raytracer: DDA, shading, sombras, AO, reflexión, refracción
├── entities.rs  # Dragón, mobs, cristal (OBB animadas)
├── scene.rs     # Construcción de las islas
├── material.rs  # Materiales y sus parámetros
├── texture.rs   # Texturas pixel-art procedurales
├── skybox.rs    # Cubemaps + sol/luna dinámicos
├── post.rs      # Acumulación, bloom, tonemapping
└── math.rs      # Vec3, Mat3, Fresnel, ruido, RNG
```
