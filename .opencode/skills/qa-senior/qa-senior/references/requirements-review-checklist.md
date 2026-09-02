# Checklist de Revisión de Requisitos / Historias de Usuario

Al revisar un requisito o historia de usuario, evalúa cada punto y reporta solo los hallazgos relevantes (no marques todo como "OK" innecesariamente; enfócate en lo que falta o es riesgoso).

## Claridad y completitud

- [ ] ¿El requisito tiene un criterio de aceptación medible (no solo descriptivo)?
- [ ] ¿Se evitan términos subjetivos sin definición ("rápido", "intuitivo", "fácil")?
- [ ] ¿Están definidos los roles/permisos involucrados (quién puede hacer qué)?
- [ ] ¿Se especifica el comportamiento ante errores (input inválido, fallo de red, timeout)?
- [ ] ¿Se especifica el comportamiento en casos límite (listas vacías, valores máximos, primer/último elemento)?

## Reglas de negocio

- [ ] ¿Existen reglas de negocio implícitas no documentadas? (ej. "un usuario no puede tener dos cuentas con el mismo email" — ¿está dicho o asumido?)
- [ ] ¿Qué pasa con datos duplicados, concurrentes o en conflicto?
- [ ] ¿Hay reglas de negocio que dependen de otro sistema o equipo y no están confirmadas?

## Dependencias y riesgos técnicos

- [ ] ¿Depende de APIs o servicios de terceros? ¿Qué pasa si fallan?
- [ ] ¿Involucra datos sensibles o PII? ¿Se especifica cómo se protegen?
- [ ] ¿Hay riesgo de problemas de concurrencia (dos usuarios editando lo mismo, doble submit)?
- [ ] ¿Requiere migración o cambio de datos existentes? ¿Qué pasa con datos legacy que no cumplen el nuevo formato?

## Priorización de hallazgos

Al reportar, clasifica cada hallazgo:
- **Crítico**: puede causar pérdida de datos, problemas de seguridad, o bloquear el flujo de negocio principal.
- **Alto**: puede causar comportamiento incorrecto visible al usuario en escenarios comunes.
- **Medio**: afecta casos poco frecuentes o experiencia de usuario, no bloquea funcionalidad core.
- **Bajo**: mejora o clarificación menor, no bloqueante.

Presenta los hallazgos ordenados por severidad, no en el orden en que aparecen en el documento original.
