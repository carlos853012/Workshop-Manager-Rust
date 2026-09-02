# Guía de Diseño de Esquema / Modelo de Datos

## Bases de datos relacionales

### Normalización vs. desnormalización
- Normaliza (evitar redundancia, 3FN como buen default general) cuando la integridad de datos y la consistencia en escrituras son prioritarias, y las lecturas pueden tolerar joins.
- Desnormaliza deliberadamente (duplicar datos, pre-calcular agregados) cuando el patrón de lectura es intensivo y los joins repetidos son un cuello de botella medido, no solo sospechado. Documenta la decisión y cómo se mantiene la consistencia de los datos duplicados (triggers, jobs de sincronización, aceptación de eventual inconsistencia leve).

### Claves e integridad
- Define claves primarias explícitas (evitar depender solo del orden de inserción).
- Usa foreign keys para relaciones cuando el motor lo soporta — la integridad referencial a nivel de base de datos es más confiable que validarla solo a nivel de aplicación.
- Usa constraints (`UNIQUE`, `CHECK`, `NOT NULL`) para reglas de negocio que deben cumplirse siempre, no solo validarlas en el código de la aplicación (la validación en aplicación es necesaria pero no suficiente si hay múltiples puntos de escritura).

### Tipos de datos
- Usa el tipo más específico y restrictivo posible (ej. `ENUM`/`CHECK` para valores limitados, tipos de fecha/hora nativos en vez de strings, tipos numéricos apropiados para dinero — evitar floats para valores monetarios, usar tipos decimales/fixed-point).

## Bases de datos documento (NoSQL)

- Diseña el esquema de documentos alrededor de las queries que se ejecutarán, no de la normalización de entidades — es normal y esperado embeber datos relacionados si se consultan siempre juntos.
- Evita documentos que crecen sin límite (ej. array de comentarios embebido en un documento de post que puede crecer indefinidamente) — considera un límite práctico o una colección separada si el crecimiento no tiene techo natural.
- Sé consistente con la forma de referenciar entre documentos (embeber vs. referenciar por ID) según si los datos relacionados se consultan siempre juntos (embeber) o de forma independiente/con alta tasa de cambio propio (referenciar).

## Al diseñar para ambos casos

- Empieza identificando las queries/casos de uso más frecuentes y críticos, y diseña el esquema para que esas queries sean eficientes — no diseñes solo pensando en las entidades del dominio de forma abstracta.
- Considera el crecimiento esperado de cada tabla/colección — una decisión de esquema razonable a 10,000 filas puede no serlo a 100 millones.
- Documenta las decisiones de esquema no obvias (por qué se desnormalizó algo, por qué se eligió cierta clave de partición) para que alguien en el futuro entienda el razonamiento.
