# Checklist de Planificación de Mantenimiento

## Mantenimiento preventivo vs. reactivo

- **Preventivo**: patching planificado, actualizaciones de dependencias/SO, renovación de certificados antes de que expiren, revisión periódica de capacidad. Se planifica con anticipación y no compite con incidentes.
- **Reactivo**: arreglar algo que ya falló. Un sistema sano tiene la mayoría de su esfuerzo de mantenimiento en la categoría preventiva, no reactiva.

## Cadencia recomendada según tipo de actualización

| Tipo | Cadencia sugerida |
|---|---|
| Parches de seguridad críticos | Días (según severidad, ver skill de seguridad) |
| Actualizaciones de seguridad no críticas | Ciclo regular (ej. mensual) |
| Actualizaciones menores de dependencias | Ciclo regular (ej. mensual/trimestral) |
| Actualizaciones mayores (breaking changes) | Planificadas con anticipación, con ventana de pruebas |
| Renovación de certificados | Automatizada si es posible (ej. Let's Encrypt con renovación automática); si es manual, con alerta 30+ días antes de expirar |
| Revisión de capacidad/costos | Trimestral o ante cambios significativos de tráfico |

## Checklist antes de una ventana de mantenimiento planificada

- [ ] Comunicación anticipada a usuarios/stakeholders afectados si hay downtime esperado.
- [ ] Plan de rollback definido si el mantenimiento no sale como se espera.
- [ ] Ventana elegida en horario de bajo tráfico si es posible.
- [ ] Verificación post-mantenimiento (smoke tests) antes de dar por cerrada la ventana.

## Gestión de sistemas legacy

- [ ] ¿Existe documentación mínima de cómo funciona y cómo se mantiene el sistema legacy, o depende de conocimiento tribal de una persona?
- [ ] ¿Se sabe qué pasaría si el sistema legacy falla completamente (plan de contingencia), especialmente si ya no tiene soporte activo del proveedor/comunidad?
- [ ] Prioriza migrar o aislar (encapsular detrás de una interfaz estable) sistemas legacy críticos que ya no reciben actualizaciones de seguridad, en vez de postergarlo indefinidamente.

## Al priorizar mantenimiento pendiente

Prioriza por: (1) riesgo de seguridad si no se hace, (2) probabilidad de causar un incidente si se posterga más, (3) costo creciente de postergarlo (la deuda de mantenimiento generalmente se vuelve más cara con el tiempo, no más barata).
