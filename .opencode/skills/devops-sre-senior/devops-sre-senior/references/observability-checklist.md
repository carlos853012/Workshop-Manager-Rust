# Checklist de Observabilidad

## Los tres pilares

- **Logs**: eventos detallados con contexto, para investigar qué pasó exactamente en un caso específico.
- **Métricas**: series temporales agregadas (latencia, tasa de error, throughput, saturación de recursos), para dashboards y alertas.
- **Traces**: seguimiento de una request individual a través de múltiples servicios, para entender dónde se pierde tiempo o falla en sistemas distribuidos.

## Logs

- [ ] Logs estructurados (JSON u otro formato parseable), no solo texto libre.
- [ ] Incluyen un identificador de correlación (trace ID/request ID) que permite seguir una request a través de múltiples servicios/componentes.
- [ ] Niveles de log usados consistentemente (debug/info/warn/error) — no todo es "info" ni todo es "error".
- [ ] No se loggean datos sensibles (contraseñas, tokens completos, PII innecesaria).

## Métricas — las "señales doradas" (Golden Signals)

- [ ] **Latencia**: tiempo de respuesta, idealmente con percentiles (p50, p95, p99), no solo promedio (el promedio esconde outliers importantes).
- [ ] **Tráfico**: volumen de requests/eventos procesados.
- [ ] **Errores**: tasa de errores, idealmente separada por tipo (4xx vs 5xx, errores de negocio vs de sistema).
- [ ] **Saturación**: qué tan cerca está el sistema de sus límites de recursos (CPU, memoria, conexiones de DB, colas).

## Alertas

- [ ] Cada alerta es accionable — si dispara y no requiere que un humano haga algo, no debería ser una alerta (considerar downgrade a dashboard/log en vez de alerta).
- [ ] Alertas basadas en síntomas de usuario (latencia alta, tasa de error alta) más que en causas internas específicas cuando sea posible — permite detectar problemas no anticipados.
- [ ] Umbrales calibrados para evitar alert fatigue (demasiadas alertas de baja severidad entrenan al equipo a ignorarlas).
- [ ] Runbook o al menos una guía mínima de qué hacer cuando dispara cada alerta crítica.

## Dashboards

- [ ] Dashboard de alto nivel con el estado general del sistema (golden signals) visible para todo el equipo.
- [ ] Dashboards específicos por servicio/componente para investigación más profunda.

## Al revisar observabilidad existente

Señala si detectas:
- Sistemas sin ninguna métrica de negocio (solo métricas técnicas) — dificulta saber si un problema técnico realmente afecta al negocio.
- Alertas silenciadas permanentemente porque "siempre dan falso positivo" — es una señal de que la alerta está mal calibrada, no de que se pueda ignorar con seguridad.
- Ausencia de trace ID compartido entre servicios, lo que hace muy difícil debuggear problemas en sistemas distribuidos.
