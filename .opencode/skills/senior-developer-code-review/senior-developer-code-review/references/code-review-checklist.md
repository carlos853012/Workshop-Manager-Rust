# Checklist de Code Review

Usa este checklist como guía de cobertura, no como lista mecánica a marcar completa en cada review — enfócate en lo que aplica al código específico.

## Correctitud

- [ ] ¿El código hace lo que el título/descripción del cambio dice que hace?
- [ ] ¿Se manejan los casos borde (colecciones vacías, valores nulos/None, límites numéricos)?
- [ ] ¿Los errores se manejan explícitamente (no se silencian con catch/except vacíos, no se ignoran `Result`/`Option` en Rust, no hay promesas sin `.catch` en JS)?
- [ ] ¿Hay condiciones de carrera evidentes en código concurrente/async?
- [ ] ¿Hay validación de inputs externos (datos de usuario, respuestas de APIs externas) antes de usarlos?

## Seguridad (nivel básico, no auditoría completa)

- [ ] ¿Hay secretos, API keys o credenciales hardcodeadas?
- [ ] ¿Hay concatenación de strings para construir queries SQL (riesgo de inyección) en vez de queries parametrizadas?
- [ ] ¿Se exponen datos sensibles en logs o mensajes de error?
- [ ] ¿Hay validación de permisos/autorización antes de operaciones sensibles, no solo autenticación?

## Diseño y mantenibilidad

- [ ] ¿Cada función/clase tiene una responsabilidad clara (principio de responsabilidad única)?
- [ ] ¿Hay duplicación de lógica que debería extraerse a una función/módulo compartido?
- [ ] ¿Los nombres de variables/funciones comunican intención sin necesidad de leer la implementación?
- [ ] ¿El anidamiento de condicionales/loops es razonable, o se beneficiaría de early returns / extracción de funciones?
- [ ] ¿Las dependencias externas (DB, APIs, reloj del sistema) están inyectadas de forma que el código sea testeable?
- [ ] ¿Los "magic numbers" o strings literales repetidos deberían ser constantes nombradas?

## Idiomático al lenguaje

- [ ] ¿El código aprovecha las características idiomáticas del lenguaje (ver `language-idioms.md`) en vez de traducir literalmente patrones de otro lenguaje?

## Performance (solo si relevante)

- [ ] ¿Hay queries dentro de loops que deberían batchearse (problema N+1)?
- [ ] ¿Hay allocaciones o copias innecesarias en rutas de código que se ejecutan frecuentemente?
- [ ] ¿La complejidad algorítmica es razonable para el tamaño de datos esperado?

## Tests (si el cambio los incluye o debería incluirlos)

- [ ] ¿El cambio incluye tests para el comportamiento nuevo, especialmente casos borde y de error?
- [ ] ¿Los tests verifican comportamiento, no implementación (evitar tests tan acoplados que se rompan con cualquier refactor)?

## Cómo priorizar los hallazgos al reportar

1. Bloqueante: bugs, problemas de seguridad, pérdida de datos potencial.
2. Recomendado fuertemente: problemas de diseño que causarán dolor de mantenimiento real.
3. Opcional/sugerencia: mejoras de legibilidad o estilo que no son críticas.
