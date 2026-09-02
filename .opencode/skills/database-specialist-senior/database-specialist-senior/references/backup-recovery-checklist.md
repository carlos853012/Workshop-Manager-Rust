# Checklist de Backup y Recuperación ante Desastres

## Definir objetivos primero

- **RPO (Recovery Point Objective)**: cuántos datos como máximo se puede permitir perder, medido en tiempo (ej. "máximo 5 minutos de datos perdidos"). Determina la frecuencia de backup/replicación necesaria.
- **RTO (Recovery Time Objective)**: cuánto tiempo como máximo puede tardar la recuperación completa. Determina qué tan automatizado y probado debe estar el proceso de restauración.

## Tipos de backup

- **Backup completo (full)**: copia completa de los datos. Simple de restaurar, pero más costoso en tiempo/espacio si se hace frecuentemente.
- **Backup incremental/diferencial**: solo los cambios desde el último backup. Más eficiente, pero la restauración requiere aplicar la cadena completa (full + incrementales), lo que puede ser más lento y más propenso a fallar si falta un eslabón de la cadena.
- **Point-in-time recovery (PITR)**: usando logs de transacciones (WAL en PostgreSQL, binlog en MySQL) para poder restaurar a un momento específico entre backups completos, minimizando la pérdida de datos (RPO bajo).

## Checklist

- [ ] Frecuencia de backup definida según el RPO acordado, no arbitrariamente.
- [ ] Backups almacenados en una ubicación distinta a la base de datos original (otra región/zona de disponibilidad, otro proveedor si es crítico) — un backup en el mismo servidor que falla no protege contra ese fallo.
- [ ] **Los backups se restauran periódicamente como prueba** (ej. trimestralmente) — un backup nunca restaurado no es una garantía, es una suposición no verificada.
- [ ] Tiempo de restauración medido realísticamente (no asumido) y comparado contra el RTO objetivo.
- [ ] Backups cifrados si contienen datos sensibles, con gestión de claves apropiada (ver skill de seguridad).
- [ ] Retención definida (cuánto tiempo se guardan los backups) según necesidades de negocio/regulatorias, con eliminación segura de los que ya no se necesitan.
- [ ] Documentación clara y accesible de cómo restaurar (runbook), que no dependa de que una persona específica esté disponible durante el incidente.

## Consideraciones adicionales

- Para sistemas críticos, considerar replicación multi-región además de backups, si el RTO/RPO exigen recuperación casi inmediata ante la caída de una región completa.
- El plan de backup/DR debe probarse con un ejercicio completo (no solo restaurar un backup aislado, sino simular el escenario de recuperación completo) al menos una vez, no solo confiar en la teoría del plan escrito.
