# Guía de Selección de Motor de Base de Datos

## Marco de decisión: empieza por el patrón de acceso, no por el motor

Preguntas clave antes de elegir:
- ¿Las relaciones entre entidades son centrales al dominio (ej. pedidos-productos-clientes con muchos joins), o los datos se acceden principalmente por clave y de forma auto-contenida?
- ¿Se necesitan transacciones ACID multi-entidad (ej. mover dinero entre cuentas), o cada operación es independiente?
- ¿El esquema de los datos es estable y bien definido, o cambia frecuentemente/es heterogéneo entre registros?
- ¿Cuál es el patrón de escala dominante: más lecturas, más escrituras, o ambos por igual?

## Familias de bases de datos y cuándo considerarlas

### Relacional (PostgreSQL, MySQL, etc.)
- Mejor ajuste cuando: hay relaciones complejas entre entidades, se necesitan transacciones ACID robustas, el esquema es relativamente estable, y se valora la flexibilidad de queries ad-hoc (SQL).
- Trade-off: escalar horizontalmente (más allá de réplicas de lectura) es más complejo que en muchas alternativas NoSQL.
- Default razonable para la mayoría de aplicaciones de negocio salvo que haya una razón específica para otra cosa.

### Documento (MongoDB, DynamoDB en modo documento, etc.)
- Mejor ajuste cuando: el esquema es heterogéneo o cambia frecuentemente, los datos se acceden principalmente como documentos auto-contenidos (poco join entre colecciones), y se prioriza velocidad de desarrollo con esquema flexible.
- Trade-off: relaciones complejas entre documentos y transacciones multi-documento son más costosas o limitadas que en relacional.

### Clave-valor (Redis, DynamoDB en modo simple, etc.)
- Mejor ajuste cuando: acceso extremadamente rápido por clave, casos de uso como caché, sesiones, contadores, colas simples.
- Trade-off: consultas más allá de "buscar por clave" son limitadas o inexistentes.

### Columnar/analítico (Cassandra, ClickHouse, BigQuery, Redshift)
- Mejor ajuste cuando: cargas de trabajo analíticas sobre grandes volúmenes (agregaciones sobre millones/miles de millones de filas), o alta escritura distribuida (Cassandra).
- Trade-off: no están optimizadas para transacciones OLTP típicas ni para queries con muchos joins.

### Grafo (Neo4j, Amazon Neptune)
- Mejor ajuste cuando: el dominio es fundamentalmente sobre relaciones y su travesía (redes sociales, detección de fraude, recomendaciones basadas en conexiones).
- Trade-off: motor especializado, menos común en el equipo promedio, y no ideal para todo lo demás.

### Series de tiempo (TimescaleDB, InfluxDB)
- Mejor ajuste cuando: datos con timestamp como dimensión central (métricas, IoT, monitoreo), con necesidad de agregaciones eficientes sobre ventanas de tiempo.

## Errores comunes a señalar

- Elegir NoSQL "por escalabilidad" sin haber agotado alternativas más simples con el motor relacional (índices, réplicas de lectura, particionamiento) que suelen resolver la mayoría de los casos reales.
- Usar múltiples motores especializados desde el día 1 sin necesidad real (complejidad operacional innecesaria) — empezar con un motor relacional sólido y agregar especializados solo cuando un caso de uso concreto lo justifique es generalmente más pragmático.
- Ignorar la experiencia del equipo con el motor al decidir — un motor "técnicamente superior" que nadie en el equipo sabe operar bien es un riesgo, no una ventaja inmediata.
