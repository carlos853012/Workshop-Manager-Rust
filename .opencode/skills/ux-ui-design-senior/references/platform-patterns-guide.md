# Guía de Patrones de Diseño Multiplataforma (Desktop, Web, Mobile)

## Principio central: comparte estructura, adapta interacción

- **Comparte entre plataformas**: tokens de diseño (color, tipografía, espaciado, iconografía de marca), arquitectura de información (misma estructura de contenido y jerarquía de tareas), tono de voz/contenido, lógica de negocio y datos.
- **Adapta por plataforma**: patrón de navegación, gestos/método de interacción, densidad de información, convenciones específicas del sistema operativo.
- Nunca fuerces el mismo patrón de navegación en todas las plataformas por conveniencia de desarrollo — el usuario trae expectativas entrenadas por su sistema operativo/contexto, y contradecirlas genera fricción real medible en usabilidad.

## Desktop (Windows / macOS / Linux)

- **Navegación**: barra de menú superior (especialmente macOS), sidebars persistentes, paneles acoplables/redimensionables (dockable panels), múltiples ventanas.
- **Interacción**: hover confiable como affordance (el usuario puede "explorar" pasando el mouse sin comprometerse a un clic), clic derecho para menús contextuales, atajos de teclado extensivos esperados por usuarios avanzados, drag-and-drop entre paneles/ventanas.
- **Densidad**: mayor densidad de información por pantalla es aceptable y a menudo deseable — el usuario tiene más espacio de pantalla y mayor precisión de puntero (mouse) que en touch.
- **Diferencias entre sistemas operativos** (no tratar "desktop" como uniforme):
  - macOS (Human Interface Guidelines): barra de menú global arriba de la pantalla (no de la ventana), botones de ventana a la izquierda, preferencia por diseño minimalista.
  - Windows (Fluent Design): barra de menú/controles dentro de cada ventana, botones de ventana a la derecha, mayor tolerancia a densidad visual.
  - Linux (GTK/Qt según entorno de escritorio): variable según el entorno (GNOME, KDE), generalmente sigue convenciones más cercanas a GTK o Qt Human Interface Guidelines respectivamente.

## Web

- **Navegación**: menú superior horizontal, breadcrumbs para jerarquías profundas, menú hamburguesa en versiones responsive/móviles del sitio.
- **Interacción**: no se puede asumir hover disponible (dispositivos touch acceden a sitios web) — todo affordance importante debe funcionar también sin hover. El botón "atrás" del navegador debe comportarse de forma predecible (no romper el estado esperado). La URL es parte de la experiencia: refleja y permite compartir/enlazar estado (ej. una búsqueda con filtros debería reflejarse en la URL).
- **Responsive real**: diseño fluido con breakpoints (no un diseño fijo que simplemente se encoge) — reorganiza la jerarquía de información según el espacio disponible, no solo reescala los mismos elementos.
- **Patrones comunes**: skeleton loaders durante carga, decisión consciente entre modal vs. página dedicada (modales para tareas cortas y contextuales, páginas dedicadas para tareas complejas o que ameritan ser compartidas/marcadas), scroll infinito (bueno para descubrimiento/exploración) vs. paginación (mejor cuando el usuario necesita referenciar o volver a una posición específica).
- **Performance percibida como parte de UX**: en web, la velocidad de carga es en sí misma un factor de usabilidad — un diseño visualmente perfecto pero lento se percibe como mala experiencia.

## Mobile (iOS / Android)

- **Navegación**: tab bar inferior es el patrón dominante en ambas plataformas para navegación principal, pero con diferencias — iOS incluye gesto nativo de swipe-back desde el borde izquierdo para retroceder; Android depende históricamente del botón/gesto de sistema "atrás", que puede comportarse distinto según la versión de Android.
- **Interacción**: diseño pensado para el pulgar (thumb zone) — las acciones más frecuentes deben estar en la zona de alcance cómodo del pulgar, no en las esquinas superiores en apps de una mano. Gestos nativos esperados: swipe para eliminar/archivar, pull-to-refresh, long-press para acciones contextuales.
- **Diferencias de lenguaje de diseño**:
  - Android sigue Material Design (Google): Floating Action Button (FAB) para la acción primaria, elevación/sombras para jerarquía, ripple effect en interacciones táctiles.
  - iOS sigue Human Interface Guidelines (Apple): menor uso de sombras/elevación, navegación con back button en la esquina superior izquierda, componentes como action sheets y modales con presentación específica.
  - No todos los patrones tienen equivalente directo entre plataformas (ej. el FAB de Material Design no tiene un análogo idiomático directo en iOS) — adaptar la intención (acción primaria accesible) usando el patrón nativo de cada plataforma, no forzar el componente idéntico.
- **Progressive disclosure**: mobile tiene mucho menos espacio — prioriza una tarea/decisión a la vez por pantalla, oculta detalle secundario detrás de una acción explícita (expandir, siguiente pantalla) en vez de la densidad aceptable en desktop.

## Diseñar una vez, adaptar bien (estrategia multiplataforma)

1. **Define la estructura de información y el flujo de tareas una sola vez** — el problema del usuario y los pasos lógicos para resolverlo no cambian entre plataformas.
2. **Adapta la capa de navegación al contexto**: sidebar persistente en desktop, tab bar inferior en mobile, top nav con posible colapso a hamburguesa en web responsive.
3. **Un mismo componente puede (y a veces debe) verse distinto por plataforma** sin romper la identidad de marca, si comparte tokens de diseño base (color, tipografía) — ejemplo: un selector de fecha nativo de iOS, uno de Android, y uno web pueden tener interacciones completamente distintas y aun así sentirse parte del mismo producto por consistencia de marca y tono.
4. **Prueba con los métodos de input reales de cada plataforma**: no asumas que un diseño probado con mouse+teclado en desktop funcionará igual de bien con touch en mobile, ni que un layout pensado para touch aprovecha bien el espacio y precisión de desktop.
5. **Prioriza consistencia de marca y de modelo mental sobre consistencia pixel-perfect** — el usuario no compara tu app en iOS contra tu app en Android lado a lado; sí nota si tu app en iOS no se comporta como el resto de las apps de iOS que usa a diario.

## Errores comunes a señalar

- Diseñar primero para una plataforma (típicamente web o iOS) y "adaptar" mecánicamente a las demás sin reconsiderar patrones de navegación/interacción específicos.
- Replicar un componente exótico de una plataforma (ej. un patrón de navegación muy específico de iOS) en otra donde el usuario no tiene esa expectativa ni referencia previa.
- Ignorar diferencias de tamaño de pantalla/densidad dentro de la misma categoría de plataforma (ej. tratar "mobile" como un solo tamaño, sin considerar tablets o pantallas plegables).
- Priorizar la conveniencia de reutilizar un mismo código de UI entre plataformas por sobre la experiencia nativa esperada por el usuario — la eficiencia de desarrollo es una restricción real a comunicar, pero no debería decidir el patrón de UX sin que quede explícito el trade-off de usabilidad que implica.
