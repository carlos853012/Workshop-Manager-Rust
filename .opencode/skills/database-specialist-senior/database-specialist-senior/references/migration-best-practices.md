# Mejores Prácticas de Migraciones de Base de Datos

## Principio central: compatibilidad hacia atrás durante el despliegue

Durante un despliegue, generalmente conviven brevemente la versión anterior y la nueva de la aplicación (o al menos hay una ventana donde el rollback debe ser posible). El esquema debe soportar ambas versiones durante esa ventana.

## Patrón seguro para cambios de esquema (expand-contract / parallel change)

Para cambios que romperían la versión anterior de la aplicación (ej. renombrar una columna, cambiar un tipo, eliminar una columna en uso):

1. **Expand**: agregar el nuevo campo/estructura sin tocar ni eliminar el anterior. Ambas versiones de la app pueden convivir (la vieja usa el campo viejo, la nueva puede empezar a usar el nuevo).
2. **Migrar datos**: backfill de datos existentes al nuevo campo/estructura, idealmente en background y sin bloquear la tabla completa si es grande.
3. **Cambiar la aplicación**: desplegar la versión de la app que usa el nuevo campo/estructura.
4. **Contract**: una vez que todas las instancias de la aplicación usan el nuevo campo y ya no hay riesgo de rollback a la versión vieja, eliminar el campo/estructura anterior.

Este patrón evita ventanas de incompatibilidad y permite rollback seguro en cualquier punto antes del paso final de "contract".

## Consideraciones para tablas grandes

- Algunas operaciones DDL bloquean la tabla completa durante su ejecución (varía según motor y tipo de operación) — para tablas con mucho tráfico, esto puede causar downtime perceptible.
- Considera herramientas de migración online (ej. `pt-online-schema-change`/`gh-ost` para MySQL, o las capacidades nativas de migración concurrente de PostgreSQL como `CREATE INDEX CONCURRENTLY`) quando el tamaño de la tabla y la sensibilidad al bloqueo lo justifiquen.
- Ejecuta migraciones largas en horarios de bajo tráfico si no se puede evitar el bloqueo por completo.

## Checklist antes de ejecutar una migración en producción

- [ ] La migración es reversible, o hay un plan claro de qué hacer si necesita revertirse.
- [ ] Se probó en un ambiente con datos representativos del volumen real (una migración que tarda 1 segundo en staging con 100 filas puede tardar horas en producción con 100 millones).
- [ ] Se comunicó al equipo la ventana de migración si hay riesgo de impacto perceptible.
- [ ] Hay backup reciente antes de ejecutar cambios estructurales significativos.
- [ ] La migración no bloquea indefinidamente si algo sale mal a mitad de camino (tiene manejo de errores/timeout apropiado).

## Migraciones de datos (no solo de esquema)

- Ejecuta en lotes (batches) en vez de una sola transacción gigante, especialmente para volúmenes grandes — reduce el tiempo de bloqueo y el riesgo de una transacción larga que consuma recursos excesivos.
- Verifica idempotencia: si la migración de datos se interrumpe y se reintenta, no debería duplicar o corromper datos ya migrados.
