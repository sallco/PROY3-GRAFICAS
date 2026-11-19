# Arquitectura y validación técnica

Nadir separa el estado simulado, las mallas y el cálculo de cada imagen. El pipeline evaluado se ejecuta en CPU. Raylib proporciona ventana, teclado/ratón, exportación PNG, texto de interfaz y presentación de la textura del framebuffer.

## Flujo de datos

```text
scene.toml + OBJ
        │
        ▼
Validación y preparación de mallas, instancias, normales y skybox
        │
        ▼
Entrada → simulación fija → nave, órbitas y transformaciones
        │
        ▼
LOD → descarte de objetos → transformación modelo/vista/proyección
        │
        ▼
Clipping homogéneo → descarte de caras → rasterización baricéntrica
        │
        ▼
Profundidad compartida → atributos del fragmento visible
        │
        ▼
Materiales e iluminación paralelos → framebuffer RGBA8
        │
        ▼
Presentación escalada e interfaz 2D
```

## Responsabilidades

| Módulo | Responsabilidad |
| --- | --- |
| `config` | Lectura TOML, límites de parámetros y validación de órbitas. |
| `obj` / `geometry` | Carga de triángulos, normales, subdivisión, disco cerrado y modelos propios. |
| `math` | Bases ortonormales, transformaciones uniformes y proyección propia. |
| `scene` | Órbitas, selección de LOD, ruido, instancias y comandos de dibujo. |
| `flight` | Pasos fijos, aceleración, orientación, gravedad, cámara, colisiones y estados de viaje. |
| `renderer` | Clipping, visibilidad, rasterización, profundidad y sombreado paralelo. |
| `shaders` / `terrain` | Materiales, mapa normal y geografía del disco. |
| `sky` / `glow` | Skybox cúbico, halo, bloom solar, warping y aproximación visual de Umbra. |
| `flat_collision` | Barrido contra la base cónica, el borde y la tapa del disco. |
| `app` / `main` | Composición del cuadro, configuración y modos de ejecución. |
| `museum` / `ui` | Cámara y reloj del museo, menú principal, catálogo, controles y HUD. |
| `demo` | Recorrido reproducible y transmisión de cuadros a FFmpeg. |

No se invocan funciones de dibujo 3D, carga automática de modelos, shaders GPU, cámaras 3D ni física de Raylib. Sus tipos vectoriales se utilizan como estructuras matemáticas.

## Geometría y transformaciones

Los OBJ se cargan con índices positivos y caras trianguladas. El parser valida coordenadas finitas, índices existentes y normales calculables. Ignora las referencias MTL porque los materiales se implementan en código.

Cada comando de dibujo contiene malla, transformación uniforme, identificador de material, objeto, radio envolvente y política de caras. La transformación mantiene una base ortonormal; su operación inversa recupera coordenadas locales. La cámara obtiene una base desde ojo, objetivo y dirección superior y construye las coordenadas homogéneas de perspectiva sin recurrir a una cámara 3D de Raylib.

La esfera base tiene 192 triángulos. La subdivisión comparte puntos medios de aristas y los proyecta a la esfera unitaria. Los cuatro niveles esféricos tienen **192, 768, 3072 y 12 288 triángulos**; el último se activa durante el acercamiento. Cada mundo sólido deforma sus niveles con un campo propio. Obsidia combina macizos de baja frecuencia, crestas y conos con calderas; Tahiti presenta colinas suaves; Prisma, plataformas heladas; Palimpsesto, dunas y una cuenca salina. Se recalculan las normales y se utiliza el mismo campo para anclar las instancias. Esta deformación cambia la silueta y es independiente del detalle aparente de materiales.

Tierra Plana utiliza anillos concéntricos, segmentos angulares, cara superior, pared y base cónica puntiaguda. Sus tres niveles tienen distinta resolución angular y radial. La geografía se define en coordenadas polares: la latitud depende del radio y la longitud del ángulo. Los mismos polígonos continentales alimentan el material y el relieve geométrico. El borde y los estratos inferiores completan el volumen.

Los cuatro tipos de modelos de superficie se generan en CPU. Se ubican sobre candidatos distribuidos mediante ángulo áureo, priorizados y filtrados por OpenSimplex antes de limitar su cantidad. Esto conserva cobertura sobre ambos hemisferios; los árboles se restringen al bioma vegetal y los pórticos evitan el centro salino. La semilla modifica ubicaciones y escala. Cada instancia almacena una posición y una base locales; al dibujarla, la transformación del planeta transporta ambas. En el cuerpo deformado se estima la normal de la superficie mediante diferencias finitas.

## Recorte, profundidad y materiales

Los triángulos se recortan en espacio homogéneo contra `w ± x`, `w ± y` y `w ± z`. Se interpolan atributos al crear intersecciones. Solo después se divide por `w`. El procedimiento evita proyecciones descontroladas al atravesar el plano cercano o mirar hacia atrás.

La rasterización evalúa coordenadas baricéntricas y limita el recorrido al rectángulo del framebuffer. La profundidad NDC usa interpolación lineal de pantalla; los atributos locales/globales y normales se interpolan con corrección de perspectiva. Un único búfer de profundidad resuelve la visibilidad de todos los objetos.

El rasterizador conserva únicamente el fragmento más cercano de cada píxel. El sombreado posterior divide la imagen entre hasta ocho hilos con regiones exclusivas de memoria. No se necesitan bloqueos para escribir colores. Los fragmentos interpolados transportan posición local, posición global, normal e identificadores de objeto/material.

El mapa normal se genera al inicio, a 256 × 128, a partir de derivadas de un campo procedural sobre la esfera. Se muestrea mediante UV esféricas, interpolación bilineal y repetición horizontal. Una base tangente transforma las normales muestreadas al espacio global; los polos utilizan un eje alternativo estable. `F6` permite aislar su contribución. No se presenta este efecto como deformación geométrica.

La estrella presenta granulación animada blanca y dorada y un halo cálido. Un bloom de tres pasadas separables, calculado en CPU a un cuarto de resolución, usa únicamente fragmentos solares visibles. Respeta la profundidad y no modifica los objetos que ocultan el sol. Los planetas y la nave reciben iluminación desde el origen estelar, con contribución ambiental estilizada y respuestas especulares según material. Tahiti y Obsidia incorporan emisión; Artaud deforma coordenadas para su vórtice. La animación de materiales usa tiempo simulado.

## Cielo y núcleo galáctico

El skybox almacena seis caras procedurales de 512 × 512 y se consulta mediante direcciones de cámara. Las estrellas se distribuyen mediante hashes de semilla fija; una banda de ruido define la nebulosa. El fondo no se desplaza al trasladar la nave.

Umbra se representa con intersecciones analíticas de rayos de fondo: región central opaca, plano del disco de acreción y distorsión de la dirección que consulta el cielo. Un arco secundario aproxima visualmente una imagen del disco alrededor de la sombra. Las bandas varían con radio, ángulo y tiempo. Estos efectos son una composición artística, no una simulación relativista.

La nave se rasteriza encima del fondo con su profundidad habitual. La interfaz se dibuja después, sin participar en la distorsión. El warping modifica la apertura aparente y añade trazos radiales animados. Los recursos de ambos destinos existen desde el inicio.

## Simulación y navegación

La simulación avanza a **120 Hz** con un acumulador de doble precisión. La posición de los planetas se calcula a partir del tiempo simulado; la orientación y la velocidad de la nave se integran con un paso constante. La presentación interpola posiciones. Cada cuadro incorpora como máximo 0,1 s de tiempo real para limitar recuperación tras una suspensión prolongada: en ese caso la simulación ralentiza su avance, en lugar de intentar recuperar indefinidamente tiempo perdido.

El empuje, la velocidad angular y el frenado son progresivos. La velocidad máxima libre es de 15 unidades/s y aumenta a 45 con impulso. La asistencia local, dentro de cuatro radios del planeta cercano, reduce el límite relativo a 4,5 unidades/s y utiliza dirección radial y velocidad orbital como referencias. La gravedad planetaria modifica continuamente la trayectoria y permanece acotada cerca de las superficies.

La cámara sigue a la nave con amortiguación exponencial y puede cambiar de orientación durante la pausa. La pausa detiene traslación, órbitas, materiales y transición galáctica.

### Colisiones

La nave utiliza una esfera de radio 0,32. Las pruebas se realizan sobre el segmento completo de movimiento respecto al centro anterior y actual del cuerpo. Se resuelve el primer impacto y se elimina la componente de velocidad que penetra la superficie. La protección de cámara utiliza el mismo sistema con margen propio.

Los planetas usan esferas conservadoras; Obsidia usa una envolvente de 1,38 radios nominales para cubrir el relieve pronunciado; los otros sólidos usan sus límites propios. Los modelos de superficie emplean esferas conservadoras individuales. Tierra Plana utiliza la intersección de un cono, una pared cilíndrica y una tapa superior, expandidos por el radio de la nave. La prueba barre el segmento contra las tres superficies; el discriminante se calcula en doble precisión para conservar impactos sobre la punta. El espacio vacío bajo el borde permanece transitable. La resolución interior expulsa por la frontera más próxima. El sol y la zona de seguridad de Umbra también bloquean la navegación.

Los límites no pretenden reproducir contacto exacto con cada triángulo. No se permite aterrizar. Los anillos de Artaud y las copas anulares son elementos decorativos sin colisión independiente.

### Viaje

El salto dura tres segundos y cambia de región en su punto medio. Conserva el estado de la nave al salir del sistema, prepara una posición segura en Umbra y rechaza nuevas solicitudes durante la transición. Al regresar restaura posición y orientación, detiene la velocidad y resuelve posibles intersecciones con cuerpos que hayan seguido moviéndose.

## Museo y navegación entre pantallas

La interfaz distingue tres pantallas: menú principal, vuelo y museo. Solo la pantalla de vuelo procesa entrada de navegación y avanza `Flight`. El museo almacena selección, orientación de cámara, desplazamiento, zoom, pausa y giro automático en un estado independiente. Al entrar libera el cursor; al volver restaura su modo de captura, si estaba activo.

`Scene::add_body_calls` reúne geometría, anillos e instancias de un cuerpo y se comparte entre vuelo y museo. `Scene::exhibit_calls` selecciona únicamente la figura solicitada, con el nivel esférico máximo o el disco más detallado. No modifica la selección de LOD del vuelo. La nave original y la optimizada se cargan y normalizan al preparar la escena; el museo usa la original por defecto.

`App::render_exhibit` utiliza el mismo rasterizador, profundidad, shaders y máscara de resplandor solar sobre un fondo de estudio calculado en CPU. La cámara sigue el centro orbital del cuerpo, por lo que la figura permanece centrada mientras su rotación y materiales evolucionan. El reloj se inicia en un instante de presentación de 12 s y avanza mientras el museo está activo, independientemente del tiempo guardado del sistema.

El zoom modifica el campo de visión entre límites acotados; la cámara permanece fuera de la figura. El desplazamiento de encuadre utiliza sus ejes derecho y superior. Esta separación permite inspeccionar detalles con zoom sin atravesar la superficie ni perder la orientación. Restablecer la vista no retrocede las animaciones. La pausa detiene el reloj y la órbita automática, pero permite manipular la cámara.

Las pruebas del museo renderizan las ocho figuras y comprueban que los fragmentos pertenecen exclusivamente al objeto elegido, que se utiliza máximo detalle, que las dos mallas de nave conservan el encuadre y que el vuelo y su LOD permanecen idénticos tras la inspección. La revisión nativa también compara la imagen solar pausada y en movimiento y recorre el catálogo con entradas de ratón.

## Rendimiento

El descarte por frustum compara esferas envolventes con los planos de cámara antes de transformar vértices y rasterizar triángulos. El LOD usa tamaño proyectado e histéresis. Las instancias de superficie se omiten en el nivel más lejano.

El benchmark recorre 72 encuadres por combinación de LOD y descarte; resume los últimos 60, después de 12 cuadros de calentamiento. Usa tiempos y cámaras reproducibles, sin abrir ventana. Reporta el cálculo CPU completo del cuadro, incluyendo cielo, selección de detalle, rasterización y sombreado; excluye presentación e interfaz.

Medición del 19 de noviembre de 2026, AMD Ryzen 7 7445HS, 12 procesadores lógicos, Rust 1.99.1, release y framebuffer de 768 × 432:

| LOD | Descarte | Media CPU | Percentil 95 | FPS CPU equivalentes | Triángulos procesados/cuadro |
| --- | --- | ---: | ---: | ---: | ---: |
| Desactivado | Desactivado | 26,44 ms | 35,89 ms | 37,82 | 106 660 |
| Desactivado | Activado | 22,42 ms | 36,40 ms | 44,61 | 31 752 |
| Activado | Desactivado | 21,51 ms | 28,68 ms | 46,50 | 43 162 |
| Activado | Activado | 20,51 ms | 27,68 ms | 48,75 | 16 830 |

La combinación reduce aproximadamente un 84 % los triángulos procesados frente a detalle alto sin descarte. La mejora de tiempo es menor porque el costo del cielo y del sombreado de píxeles visibles permanece. Los FPS de esta tabla son el inverso del tiempo medio CPU, no una medición de la presentación gráfica.

El [CSV](evidencia/benchmark.csv) conserva las muestras y [el entorno registrado](evidencia/entorno.json) acompaña los resultados. Los valores dependen de carga del sistema, compilador, resolución y encuadre; no se extrapolan a otros equipos. Las métricas interactivas separan tiempo CPU y FPS de presentación.

## Verificación

Las pruebas automatizadas comprueban lógica y resultados observables: visibilidad conservada por el descarte, profundidad independiente del orden, colisiones barridas esféricas y cónicas, resplandor con oclusión, límites de relieve, cobertura de ambos hemisferios, reproducibilidad y equivalencia de navegación a distintas frecuencias de presentación. La revisión visual utiliza las capturas incluidas y el video. La ventana real se abrió en XWayland y recibió entradas de teclado para comprobar interfaz, comparaciones y transición galáctica; cerró con código 0. La prueba de resistencia cubre diez minutos de tiempo simulado, no diez minutos de evaluación manual.

El alcance no incluye aterrizaje, caminata, combate, cuerpos que colisionen entre sí, sombras geométricas completas ni relatividad general.
