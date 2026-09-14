# Guía de Sistemas de Diseño

## Qué incluir en un sistema de diseño

- **Tokens de diseño**: valores base reutilizables (color, tipografía, espaciado, radios de borde, sombras) — la fuente de verdad para que todo el producto sea visualmente consistente.
- **Componentes**: elementos de UI reutilizables (botones, inputs, cards, modales) con sus variantes y estados (default, hover, disabled, error, loading).
- **Patrones**: combinaciones de componentes para resolver problemas recurrentes (ej. patrón de formulario con validación, patrón de tabla con paginación).
- **Guías de contenido/voz**: cómo se escribe el texto de interfaz de forma consistente (tono, terminología, formato de fechas/números).

## Principio central: documentar el "cuándo", no solo el "cómo"

La causa más común de inconsistencia en productos con sistema de diseño no es la falta de componentes, sino la falta de claridad sobre cuándo usar cada uno. Para cada componente, documenta:
- Cuándo usarlo (y cuándo NO usarlo, con ejemplos si ayuda).
- Qué variantes existen y cuándo corresponde cada una.
- Reglas de contenido asociadas (ej. longitud máxima de texto en un botón).

## Gobernanza

- Define un proceso claro para proponer nuevos componentes o cambios a existentes — sin proceso, el sistema se fragmenta con variantes ad-hoc creadas por distintos equipos.
- Balancea consistencia con flexibilidad: un sistema demasiado rígido que no permite excepciones justificadas termina siendo evitado por los equipos que necesitan resolver casos reales no contemplados.
- Versiona el sistema de diseño y comunica cambios breaking de forma clara a los equipos que lo consumen, igual que se versionaría una API.

## Señales de que un sistema de diseño necesita atención

- Múltiples componentes que resuelven visualmente lo mismo pero fueron creados por separado (indica que no se descubrió o no se siguió el existente).
- Documentación desactualizada respecto al código/diseño real (la fuente de verdad se fragmenta entre el sistema documentado y lo que realmente se usa).
- Equipos que evitan el sistema de diseño porque "es más rápido hacerlo a mano" — generalmente señala fricción real en cómo se accede o usa el sistema, vale la pena investigar la causa en vez de solo exigir cumplimiento.

## Relación con la ejecución visual/código

Esta guía cubre la estructura y gobernanza del sistema de diseño como producto. Para la ejecución visual concreta (paleta de colores, tipografía, código de los componentes en este entorno), consulta la skill `frontend-design`.
