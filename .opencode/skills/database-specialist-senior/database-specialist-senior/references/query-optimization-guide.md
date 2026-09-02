# Guía de Optimización de Queries

## Proceso de diagnóstico

1. **Obtén el plan de ejecución** antes de proponer cualquier cambio (`EXPLAIN ANALYZE` en PostgreSQL, `EXPLAIN` en MySQL, `.explain("executionStats")` en MongoDB). Sin esto, cualquier sugerencia es una suposición.
2. **Identifica el cuello de botella** en el plan:
   - Full table scan / collection scan donde se esperaría uso de índice.
   - Ordenamientos costosos en memoria (`Sort` sin índice que ya provea el orden).
   - Joins costosos (nested loop sobre tablas grandes sin índice en la columna de join).
   - Estimaciones de filas muy alejadas de la realidad (estadísticas desactualizadas).
3. **Verifica el volumen real de datos involucrado** — una query "lenta" sobre 50 filas probablemente no es un problema de índices sino de otra cosa (conexión, red, lógica de aplicación).

## Causas comunes de queries lentas

- **Falta de índice** en columnas usadas en `WHERE`, `JOIN`, `ORDER BY`.
- **Índice no usado** aunque exista, por ejemplo por usar una función sobre la columna indexada (`WHERE LOWER(email) = ...` no usa un índice normal sobre `email` salvo que sea un índice funcional/expression index).
- **N+1 queries**: una query por cada elemento de una lista en vez de una sola query batcheada (join o `WHERE IN`).
- **SELECT \*** trayendo más columnas de las necesarias, especialmente columnas grandes (texto largo, blobs) no usadas.
- **Paginación con OFFSET grande**: `OFFSET 100000` obliga a la base de datos a recorrer y descartar 100,000 filas — preferir paginación por cursor (`WHERE id > último_id_visto`) para datasets grandes.
- **Estadísticas desactualizadas**: el optimizador de queries decide el plan basado en estadísticas de la tabla; si están desactualizadas (tras cargas masivas de datos), puede elegir un plan subóptimo (ejecutar `ANALYZE`/equivalente).

## Al proponer una solución

- Prioriza el cambio de menor riesgo/mayor impacto primero (generalmente: agregar el índice faltante) antes de reescribir la query o cambiar el modelo de datos.
- Verifica el impacto en escrituras de cualquier índice nuevo propuesto — no es gratis.
- Si la query es fundamentalmente costosa por el volumen de datos y no hay optimización de índice/query que la resuelva, considera: caching de resultados, agregados pre-calculados, o mover la carga analítica a un motor especializado (ver `database-selection-guide.md`) en vez de forzar la solución en el motor transaccional.

## Ejemplo de razonamiento (ilustrativo, adaptar al caso real)

Una query que filtra por `status` y ordena por `created_at` en una tabla grande sin índice compuesto hará un scan completo y luego ordenará en memoria. Un índice compuesto `(status, created_at)` permite que la base de datos filtre y devuelva las filas ya en el orden correcto sin scan completo ni sort adicional — pero solo si el orden de las columnas en el índice coincide con cómo se filtra/ordena en la query real.
