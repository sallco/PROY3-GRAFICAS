# Matriz de evaluación y recorrido

Esta matriz identifica las características implementadas y cómo verificarlas. Los puntajes corresponden al menú del PDF y no constituyen una calificación otorgada. El máximo académico es 100, aunque el menú ofrece 200 puntos.

## Requisitos que habilitan la evaluación

| Requisito obligatorio | Implementación | Verificación |
| --- | --- | --- |
| Pipeline propio por software | Transformaciones, clipping, rasterización, profundidad, materiales e iluminación en CPU. | `math`, `renderer`, `shaders`; inspeccionar las llamadas de presentación en `main`. |
| Sol, nave propia controlable y planeta con órbita/rotación temporal | Estrella central, nave OBJ y seis mundos animados mediante tiempo simulado. | Iniciar, volar con `W/A/S/D`, observar el sistema y cambiar `orbit_speed`. |
| Acercarse, rodear y alejarse sin cambiar escena | Control libre y asistencia local dentro de una escena persistente. | Video 00:00–00:46 y ejecución interactiva. |
| Modelos, transformaciones, profundidad y shader propio | OBJ triangulados, transformaciones por objeto y profundidad compartida; múltiples materiales propios. | Pruebas de profundidad y capturas. |
| Fuente, recursos, instrucciones y video | Código, originales, mallas generadas, README y MP4 incluidos. | Compilar desde clon limpio y seguir el README. |

## Menú del PDF

| Categoría / característica | Puntos solicitados | Dónde observarla y cómo explicarla |
| --- | ---: | --- |
| Cinco planetas distinguibles | 50 | Tahiti, Obsidia, Prisma, Artaud y Palimpsesto: geometría/material, órbita, rotación y acercamiento. Tierra Plana amplía el contenido sin incrementar este máximo. |
| Cuatro tipos de modelos propios sobre superficies | 20 | Árboles anulares, agujas de basalto, cristales y pórticos de observatorio. `geometry::surface_models`; OBJ de referencia en `assets/models/superficie`. |
| Distribución procedural reproducible mediante ruido | 10 | Candidatos esféricos filtrados por OpenSimplex; cambiar `seed`, recargar y restaurar la misma semilla. `scene::generate_instances`. |
| Posición y orientación sobre superficie curva en rotación | 5 | Las instancias se transforman desde el espacio local del planeta; las normales de Obsidia incluyen su deformación. Prueba `instances_follow_curvature_and_rotation`. |
| Cuatro shaders propios diferentes | 24 | Plasma estelar, redes bioluminiscentes, fracturas de lava y bandas con vórtice. No se cuentan variaciones de color como shaders diferentes. |
| Shader animado | 8 | Evolución del plasma, emisión de Tahiti, frentes térmicos y corrientes de Artaud. El tiempo simulado alimenta los materiales. |
| Mapeo de normales | 8 | Mapa procedural, UV esféricas, muestreo bilineal y base tangente; comparar con `F6`, especialmente en Prisma. |
| Deformación geométrica planetaria | 10 | El radio de vértices de Obsidia cambia mediante ruido; inspeccionar la silueta y `terrain::world_height`. |
| Generación/subdivisión propia de geometría | 10 | Subdivisión con caché de aristas y proyección esférica; Tierra Plana se construye con anillos, tapa, pared y base cónica. |
| Modo local para rodear planetas | 8 | `F` cerca del cuerpo, traslación lateral y regulación radial de altura sin cortar el vuelo. |
| Aceleración, frenado y giro progresivos | 8 | `W`, `X`, flechas; integración fija y amortiguación exponencial. Prueba a 30 y 144 FPS de presentación. |
| Atracción gravitatoria planetaria | 10 | La nave modifica su trayectoria sin empuje; comparar posiciones y velocidades. Prueba `gravity_changes_unpowered_trajectory`. |
| Colisiones | 4 | Aproximación directa y con impulso; esferas barridas y cono cerrado para Tierra Plana. |
| Descarte previo a rasterizar | 7 | `F3` y `F5`; comparar objetos descartados y triángulos, manteniendo el encuadre. |
| Niveles de detalle | 8 | `F3` y `F4`; alejarse del planeta y observar contadores. La prueba y el benchmark comparan trabajo real. |
| Coherencia visual | Hasta 5 | Paletas por mundo, nave consistente, cielo común, jerarquía de interfaz y composición del núcleo. |
| Claridad de experiencia | Hasta 5 | Ayuda integrada, destinos señalados, botón de salto, recuperación y recorrido documentado. |
| **Total del menú** | **200** | Sujeto a funcionamiento y explicación durante la evaluación. |

## Rúbrica de la plataforma

| Criterio visible | Evidencia |
| --- | --- |
| Dos criterios subjetivos de 30 y 20 puntos | La captura no contiene sus descripciones ampliadas. Se aporta dirección visual coherente, interfaz, recorrido y documentación; no se inventan condiciones adicionales. |
| Cinco elementos planeta/estrella/luna | Seis planetas y una estrella central; cinco planetas iniciales cubren por sí solos el máximo visible. |
| Warping animado | `G`, secuencia de tres segundos con distorsión y trazos radiales; video 01:16–01:19 y 01:30–01:33. |
| Nave propia que siga a la cámara | La nave permanece visible en tercera persona y la cámara acompaña su posición y orientación durante el vuelo 3D. |
| Skybox con estrellas | Seis caras procedurales, muestreadas por dirección, con estrellas y banda galáctica. |
| Condiciones que eviten atravesar elementos | Protección del sol, cuerpos, modelos de superficie y núcleo; cámara también protegida. Los anillos decorativos no se presentan como sólidos. |
| Movimiento 3D de cámara | Seguimiento de nave con traslación libre, orientación, alabeo y ratón/flechas. |
| Órbitas planetarias | Trayectorias inclinadas y rotación independiente, dependientes del tiempo simulado. |

## Guion de demostración interactiva

1. Iniciar en release, seleccionar **Explorar sistema** y mostrar la nave y la ayuda. Avanzar con `W`, frenar con `X` y orientar con las flechas.
2. Acercarse a Tahiti, activar `F` y usar `A/D` para rodearlo. Mostrar árboles anulares y emisión superficial. Alejarse con `S`.
3. Seleccionar mundos con `Tab`, orientar hacia el marcador y utilizar el impulso para viajar. Examinar lava, hielo, tormenta, dunas y pórticos.
4. En Tierra Plana, recorrer el borde y observar la base. Mostrar la protección de tapas y pared.
5. Activar `F3`; comparar `F4/F5` desde un mismo encuadre. Comparar normales con `F6`.
6. Activar `G`, maniobrar alrededor de Umbra y regresar. Mostrar que el sistema ha seguido evolucionando.
7. Modificar una semilla o velocidad orbital y usar `R`; explicar el parámetro y restaurar su valor.

## Video incluido

[Descargar o reproducir el MP4](video/nadir.mp4). Duración: **96 segundos**, 1280 × 720, 24 cuadros/s.

| Intervalo | Contenido |
| --- | --- |
| 00:00–00:12 | Vuelo y aproximación mediante entradas al controlador. |
| 00:12–00:38 | Vuelta completa a Tahiti, sin corte de escena. |
| 00:38–00:46 | Alejamiento continuo. |
| 00:46–01:16 | Montaje de los otros cinco mundos; Tierra Plana muestra superficie, borde y punta inferior. |
| 01:16–01:19 | Warping hacia el núcleo. |
| 01:19–01:30 | Vuelo en Umbra. |
| 01:30–01:33 | Regreso al sistema. |
| 01:33–01:36 | Cierre del recorrido. |

El video es una captura reproducible del renderer con pilotaje programado y un montaje identificado de vistas. No sustituye la demostración de controles ni las comparaciones técnicas en la aplicación.

## Inspección adicional mediante el museo

Desde el menú principal o con `M` en vuelo, abrir el catálogo y seleccionar cualquiera de sus ocho figuras. Inspeccionar superficies y modelos con zoom, órbita y desplazamiento; pausar la animación y volver a activarla. En la nave, `V` compara los 116 480 triángulos originales con los 3261 de vuelo. Regresar con `Esc` conserva el estado de la exploración. El museo facilita explicar los materiales y la geometría; no se contabiliza como una categoría adicional del menú de puntos.
