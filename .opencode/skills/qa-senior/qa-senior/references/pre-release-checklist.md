# Checklist Crítico de Pre-Producción

Usa este checklist antes de aprobar un release o cambio para producción. No todo aplica siempre — filtra según el tamaño e impacto del cambio, pero no omitas nada de la sección "Nunca saltarse" sin justificación explícita del usuario.

## Nunca saltarse (aunque el cambio parezca chico)

- [ ] **Plan de rollback**: ¿se puede revertir el cambio rápido si algo falla? ¿Es reversible una migración de datos?
- [ ] **Smoke test post-deploy**: lista corta de flujos críticos a verificar inmediatamente después de desplegar (login, flujo de pago, endpoints más usados).
- [ ] **Monitoreo/alertas**: ¿hay logs, métricas o alertas que avisen si algo sale mal después del deploy? ¿Alguien está mirando en las primeras horas?
- [ ] **Impacto en datos existentes**: si hay migración de esquema o datos, ¿qué pasa con registros que no cumplen el nuevo formato?

## Funcional

- [ ] Regresión de flujos críticos de negocio (no solo la feature nueva — verificar que no se rompió nada alrededor).
- [ ] Casos negativos y de límite de la funcionalidad nueva (ver `test-case-template.md`).
- [ ] Compatibilidad con roles/permisos existentes (¿un usuario con permisos limitados sigue viendo lo que debe ver, ni más ni menos?).

## No funcional — con frecuencia subestimadas

- [ ] **Performance/carga**: ¿el cambio introduce queries N+1, llamadas a APIs externas sin timeout, o payloads más grandes? ¿Se probó con volumen de datos realista, no solo con datos de prueba mínimos?
- [ ] **Seguridad básica**:
  - Validación de inputs en backend (no confiar solo en validación de frontend).
  - Manejo correcto de autenticación/autorización en endpoints nuevos.
  - No exponer datos sensibles en logs, respuestas de error, o payloads innecesarios.
  - Dependencias nuevas escaneadas por vulnerabilidades conocidas (`cargo audit`, `npm audit`/`pnpm audit`, `pip-audit`, según stack).
- [ ] **Manejo de errores y resiliencia**: ¿qué pasa si un servicio externo del que depende el cambio está caído o lento? ¿hay timeouts, reintentos, circuit breakers razonables?
- [ ] **Compatibilidad**: navegadores/dispositivos soportados (web), versiones de OS soportadas (móvil), versiones de API si hay clientes externos consumiéndola.
- [ ] **Accesibilidad básica** (si es UI): navegación por teclado, contraste, labels en formularios — al menos lo mínimo, no requiere una auditoría completa para cada cambio chico.
- [ ] **Idempotencia**: si la acción se puede reintentar (doble click, reintento de red), ¿se duplican datos o efectos (ej. doble cobro)?
- [ ] **Concurrencia**: ¿dos usuarios/procesos pueden generar una condición de carrera con este cambio?

## Proceso / despliegue

- [ ] Feature flag o despliegue gradual (canary/rollout progresivo) disponible para cambios de alto riesgo, en vez de "todo o nada".
- [ ] Comunicación al equipo/soporte sobre el cambio, especialmente si afecta comportamiento visible al usuario.
- [ ] Documentación o changelog actualizado si el cambio afecta a otros equipos o consumidores de una API.

## Cómo presentar el resultado

Al aplicar este checklist a un cambio específico, no te limites a marcar casillas — para cada punto relevante indica:
- Si aplica o no al cambio en cuestión.
- Si aplica y no está cubierto, clasifícalo como bloqueante o no bloqueante para el release, con una breve justificación.

Prioriza señalar lo que falta por sobre confirmar lo que ya está bien — el valor de esta revisión está en detectar huecos, no en generar una lista larga de confirmaciones.
