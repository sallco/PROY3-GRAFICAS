# Nadir

### Observatorio de mundos imposibles

Seis mundos en movimiento, una nave de exploración y una ruta hacia el núcleo galáctico. **Nadir** construye cada imagen mediante un rasterizador escrito en Rust que se ejecuta en CPU: desde las transformaciones y el recorte de triángulos hasta la profundidad, la iluminación y los materiales procedurales.

![Vuelo alrededor de Tahiti](docs/imagenes/vuelo.png)

**Autor:** sallco · **Modalidad:** individual · **Curso:** Gráficas por Computadora, Universidad del Valle de Guatemala.

[Demostración en video](docs/video/nadir.mp4) · [Arquitectura](docs/ARQUITECTURA.md) · [Evaluación](docs/EVALUACION.md) · [Recursos](docs/RECURSOS.md)

## La experiencia

La exploración comienza frente a Tahiti. La nave puede aproximarse, rodear el planeta y alejarse sin cambiar de escena. La asistencia local facilita maniobrar cerca de las superficies; el impulso de crucero permite recorrer el sistema. Las órbitas, la rotación, la gravedad y los materiales continúan evolucionando durante el vuelo.

| Mundo | Identidad |
| --- | --- |
| **Tahiti** | Bosque de copas anulares escalonadas, colinas verdes y lagunas violetas con bioluminiscencia tenue. |
| **Obsidia** | Macizo volcánico fracturado: crestas irregulares, conos con calderas, agujas de basalto y lava naranja animada. |
| **Prisma** | Glaciares blancos, grietas azul profundo y grandes agujas cristalinas con reflejos violetas. |
| **Artaud** | Atmósfera de bandas ámbar y malva, tormenta oscura en espiral y tres anillos separados. |
| **Palimpsesto** | Desierto de cobre: campos de dunas, roca expuesta, una cuenca salina y pórticos de observatorio. |
| **Tierra Plana** | Disco de océanos y continentes, relieve continental, nubes, borde helado y una base rocosa cónica y puntiaguda. |

| Tahiti | Obsidia |
| --- | --- |
| ![Tahiti](docs/imagenes/tahiti.png) | ![Obsidia](docs/imagenes/obsidia.png) |
| **Prisma** | **Artaud** |
| ![Prisma](docs/imagenes/prisma.png) | ![Artaud](docs/imagenes/artaud.png) |
| **Palimpsesto** | **Tierra Plana** |
| ![Palimpsesto](docs/imagenes/palimpsesto.png) | ![Tierra Plana](docs/imagenes/tierra-plana.png) |

**Tahiti** es el planeta-bosque: un ecosistema de colinas verdes, lagunas violetas y árboles de copas anulares escalonadas. La identidad de cada mundo se apoya en relieve, escala de objetos y respuesta del material, además de su paleta.

El sol utiliza granulación blanca y dorada, halo cálido y resplandor calculado en CPU. [Ver el sol](docs/imagenes/sol.png). La base de Tierra Plana puede observarse [desde abajo](docs/imagenes/tierra-plana-inferior.png).

**Umbra** ocupa una región distinta del sistema planetario. El botón de viaje o la tecla `G` activa una secuencia de warping y conduce al agujero negro supermasivo del núcleo galáctico. Allí se conserva el vuelo libre. Un segundo salto devuelve la nave al sistema, cuya simulación continúa durante la expedición.

![Umbra: disco de acreción y distorsión del fondo](docs/imagenes/umbra.png)

Las escalas, velocidades y fenómenos visuales se eligieron para una experiencia legible e interactiva. La representación del agujero negro utiliza una aproximación artística en CPU; no integra ecuaciones relativistas.

## Compilación y ejecución

Verificado en Linux con **Rust 1.99.1**, Cargo y Raylib para Rust 5.5.1. La primera compilación necesita acceso a crates.io, CMake, compilador C/C++, Clang/libclang y bibliotecas de desarrollo de OpenGL y X11, incluyendo Xrandr, Xinerama, Xcursor y Xi. La ventana requiere una sesión gráfica; en Wayland se utiliza XWayland.

```bash
git clone https://github.com/sallco/PROY3-GRAFICAS.git
cd PROY3-GRAFICAS
cargo build --release --locked
cargo run --release --locked
```

Se recomienda la compilación **release**. La ventana mide 1280 × 720; el framebuffer se calcula a 768 × 432 y se escala para su presentación. Los recursos y la configuración predeterminada se resuelven desde el directorio del proyecto incorporado durante la compilación.

Al iniciar se muestra el menú principal: `Enter` abre la exploración y `M` entra al museo de modelos. También se puede elegir con el ratón.

El ratón se captura con `C`; las flechas permiten orientar la nave desde el inicio. El botón de viaje funciona con el cursor libre.

## Controles de vuelo

| Entrada | Acción |
| --- | --- |
| `W` / `S` | Empuje longitudinal hacia delante o atrás. |
| `A` / `D` | Traslación lateral. |
| `Espacio` / `Ctrl izquierdo` | Traslación vertical. |
| Flechas o ratón capturado | Cambiar orientación. |
| `Q` / `E` | Alabeo. |
| `Shift izquierdo` | Impulso de crucero; velocidad máxima de 45 unidades/s en vuelo libre. |
| `X` | Frenado progresivo. |
| `F` | Activar o desactivar asistencia local cerca del planeta más próximo. |
| `Tab` | Seleccionar el destino indicado en pantalla; no teletransporta la nave. |
| `G` o botón superior | Viajar a Umbra o regresar. |
| `C` | Capturar o liberar el ratón. |
| `P` | Pausar la simulación; la orientación permanece disponible. |
| `H` | Mostrar u ocultar ayuda. |
| `R` | Recargar configuración y restablecer el vuelo. |
| `M` | Abrir el museo conservando el estado del vuelo. |
| `Esc` | Volver al menú principal; desde el menú, salir. |

Para rodear un planeta, activar `F`, orientar la nave hacia su superficie y utilizar `A`/`D` para recorrerla lateralmente. `Espacio`/`Ctrl` regulan la componente radial de movimiento. La asistencia ajusta velocidad y referencia de control; no pilota automáticamente.

[Ver la interfaz con ayuda y métricas](docs/imagenes/interfaz.png).

El indicador de distancia radial usa la distancia al centro menos el radio nominal. En Tierra Plana es una referencia de navegación, no una medida de altura sobre el disco.

### Instrumentación

| Tecla | Comparación |
| --- | --- |
| `F3` | Tiempo de CPU, FPS de presentación, resolución, objetos, triángulos y fragmentos. |
| `F4` | Activar/desactivar LOD; al desactivarlo todos los mundos usan detalle alto. |
| `F5` | Activar/desactivar descarte de objetos por frustum. |
| `F6` | Activar/desactivar mapeo de normales. |

Las métricas permiten demostrar que las optimizaciones modifican el trabajo del rasterizador. Los anillos y copas anulares son geometría decorativa y no tienen colisión; los cuerpos y modelos de superficie sí emplean volúmenes conservadores.

## Museo de modelos

El museo es una vista separada de la exploración, accesible desde el menú principal o mediante `M` durante el vuelo. Su catálogo contiene **ocho figuras**: el sol, Tahiti, Obsidia, Prisma, Artaud, Palimpsesto, Tierra Plana y la nave.

![Museo: inspección de Prisma](docs/imagenes/museo.png)

Cada figura se renderiza en CPU sobre un fondo de estudio con el máximo detalle geométrico disponible. Los planetas conservan sus modelos de superficie y anillos. La rotación axial, las nubes, la lava, las tormentas, la granulación solar y la emisión siguen usando los mismos materiales y algoritmos del mundo libre; el sol conserva su resplandor. La cámara acompaña el centro del cuerpo para mantener la figura encuadrada.

El museo tiene un reloj independiente. La exploración queda suspendida mientras se visita el museo o el menú: al regresar se conservan posición, velocidad, orientación, cámara, destino y pausa del vuelo. Pausar una figura permite seguir inspeccionándola con la cámara.

| Entrada del museo | Acción |
| --- | --- |
| Clic en el catálogo / `1`–`8` | Seleccionar una figura. |
| `Tab` / `Shift` + `Tab` | Seleccionar la figura siguiente o anterior. |
| Arrastrar con el botón izquierdo / flechas | Orbitar alrededor de la figura. |
| Rueda / `+` / `-` | Acercar o alejar mediante zoom. |
| Arrastrar con el botón derecho / `Shift` + flechas | Desplazar el encuadre para inspeccionar un detalle. |
| `Espacio` | Activar o desactivar el giro automático de la vista. |
| `P` | Pausar o reanudar las animaciones y el giro automático. |
| `R` | Restablecer el encuadre, sin reiniciar el tiempo de animación. |
| `V`, al observar la nave | Comparar el modelo original con la malla de vuelo. |
| `F3` | Mostrar tiempo de CPU, cantidad de triángulos y zoom. |
| `Esc` / `M` | Regresar a la pantalla desde la que se abrió el museo. |

La nave se muestra inicialmente con sus **116 480 triángulos originales**. La comparación utiliza los mismos controles y encuadre para alternar con la malla de vuelo de 3261 triángulos. No se cargan modelos al cambiar de figura: ambas mallas y los demás recursos se preparan al iniciar.

[Menú principal](docs/imagenes/menu.png) · [Sol en el museo](docs/imagenes/museo-sol.png) · [Nave original](docs/imagenes/museo-nave.png)

## Configuración y demostración técnica

Los parámetros se encuentran en [`config/scene.toml`](config/scene.toml): semilla, resolución interna, radio estelar, intensidad del shader, radios planetarios, órbitas, velocidades, inclinación y cantidad de instancias.

```bash
cargo run --release --locked -- --config config/scene.toml
```

Para demostrar un cambio, editar un parámetro y presionar `R`. La nueva escena se aplica después de validar el archivo y reconstruir sus recursos. Ante un error de configuración se conserva la escena anterior y se muestra el motivo. La recarga correcta restablece el vuelo y el tiempo; no es una migración del estado actual.

Ejemplos: cambiar `seed` de 42 a 43 altera la distribución; modificar `orbit_speed` cambia la velocidad orbital; ajustar `shader_intensity` modifica la emisión; modificar un `radius` cambia el cuerpo y sus límites de protección. La validación rechaza órbitas cuyos volúmenes puedan superponerse.

## Evidencia reproducible

```bash
# Capturas a tiempos y cámaras fijos, sin abrir ventana.
cargo run --release --locked -- --export-captures --output output/capturas

# Cuatro combinaciones de LOD/descarte; guarda un CSV.
cargo run --release --locked -- --benchmark --output output/mediciones

# Exportar los cuatro modelos de superficie a OBJ.
cargo run --release --locked -- --export-models --output output/modelos

# Generar el video de 96 segundos; requiere FFmpeg y DejaVu Sans en Linux.
cargo run --release --locked -- --record-demo --output output/nadir.mp4
```

El [video incluido](docs/video/nadir.mp4) se genera con el mismo renderer. Sus primeros 46 segundos utilizan entradas calculadas para el controlador de vuelo: aproximación, vuelta completa a Tahiti y alejamiento continuo. Continúa con un montaje de los otros mundos y el viaje de ida y vuelta a Umbra. Los rótulos y la codificación del video se aplican con FFmpeg. Este recorrido reproducible complementa la ejecución interactiva.

## Verificación

```bash
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --release --locked
```

Las **36 pruebas automatizadas** cubren clipping, profundidad entre objetos, recursos, generación de geometría, semillas, orientación de instancias, equivalencia temporal, gravedad, colisiones, pausa, viaje de ida y vuelta y equivalencia de imagen con/sin descarte. Las pruebas del museo verifican aislamiento de las ocho figuras, máximo detalle, pausa, límites de cámara, comparación de mallas y conservación del vuelo.

La nave de vuelo conserva el modelo original y reduce sus **116 480 triángulos a 3261**. La generación puede repetirse con Blender:

```bash
blender --background --python scripts/optimizar_nave.py
```

Los resultados de rendimiento, sus condiciones y sus límites se detallan en [arquitectura](docs/ARQUITECTURA.md); las muestras individuales están en [el CSV](docs/evidencia/benchmark.csv). La velocidad depende de la resolución, el encuadre y el equipo.

## Autoría y recursos

Proyecto individual de **sallco** (`dcalderon918@gmail.com`): nave, implementación, materiales, modelos procedurales, integración y documentación. La esfera inicial es un recurso geométrico proporcionado para el curso. No se necesitan imágenes de texturas ni paquetes artísticos descargados.

Se utilizó asistencia de IA en la planificación, implementación, revisión y documentación. Esta declaración se incluye conforme a las especificaciones de entrega; no atribuye coautoría académica. El código y los algoritmos deben poder explicarse durante la evaluación.

La procedencia de archivos, las dependencias y los procedimientos de generación se describen en [recursos](docs/RECURSOS.md).
