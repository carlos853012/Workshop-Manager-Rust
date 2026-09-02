# Guía de Replicación, Sharding y Particionamiento

## Orden recomendado de escalado (de menor a mayor complejidad operacional)

1. **Optimización de queries e índices** — casi siempre el mayor retorno con menor costo.
2. **Vertical scaling** (servidor más grande) — simple, pero con límite físico y de costo creciente.
3. **Caching** (ej. Redis delante de queries costosas y frecuentes) — reduce carga sin cambiar la base de datos en sí.
4. **Réplicas de lectura** — escala lecturas distribuyendo queries de solo lectura a réplicas, manteniendo un único nodo de escritura (primary). Relativamente simple de implementar en la mayoría de motores modernos.
5. **Particionamiento (partitioning)** dentro de una misma instancia — divide una tabla grande en particiones más chicas (por rango de fecha, por región, etc.) para mejorar performance de queries y mantenimiento (ej. borrar datos viejos eliminando una partición completa).
6. **Sharding** (particionamiento horizontal entre múltiples instancias/servidores) — el recurso de mayor complejidad operacional; considerarlo solo cuando las opciones anteriores ya no son suficientes para el volumen de escritura o almacenamiento.

## Réplicas de lectura — consideraciones

- Las réplicas suelen tener replicación asíncrona (lag entre el primary y las réplicas) — la aplicación debe tolerar leer datos ligeramente desactualizados en las réplicas, o dirigir explícitamente a el primary las lecturas que requieren datos actualizados al instante (ej. justo después de una escritura propia del mismo usuario).

## Sharding — consideraciones críticas

- **La clave de partición (shard key) es la decisión más importante y más costosa de cambiar después.** Debe elegirse según el patrón de acceso dominante (qué campo se usa más frecuentemente para filtrar/buscar) para evitar que la mayoría de las queries necesiten consultar todos los shards (scatter-gather, que anula buena parte del beneficio del sharding).
- Evita shard keys que generen "hot shards" (ej. una clave basada en timestamp donde todas las escrituras recientes van al mismo shard) — busca una distribución razonablemente uniforme de carga entre shards.
- Transacciones y joins que cruzan shards son significativamente más costosos o imposibles según el motor — el diseño de datos debe minimizar la necesidad de operaciones cross-shard.
- Sharding agrega complejidad operacional real: rebalanceo de datos al agregar shards, backups por shard, monitoreo distribuido — asegúrate de que el equipo puede operar esa complejidad antes de recomendarlo.

## Particionamiento (dentro de una instancia) — casos de uso comunes

- Tablas de eventos/logs con particionamiento por fecha, permitiendo borrar datos viejos eficientemente (drop de partición en vez de DELETE masivo) y mejorando performance de queries que filtran por rango de fecha reciente.
- Datos multi-tenant particionados por tenant/cliente cuando el volumen por tenant es alto.

## Al recomendar escalado

Pregunta primero cuál es el cuello de botella real medido (¿es CPU, memoria, I/O, número de conexiones, volumen de escritura, volumen de lectura?) antes de recomendar una solución — escalar la dimensión equivocada no resuelve el problema y agrega complejidad innecesaria.
