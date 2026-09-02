# Guía de Estrategia de Automatización

## Marco de decisión: qué automatizar

Prioriza automatizar cuando se cumplen varios de estos factores:
- Se ejecuta con frecuencia (cada release, cada PR, diariamente).
- El flujo es estable (no cambia constantemente su UI/lógica).
- Es un flujo crítico de negocio (login, checkout, pagos, flujos con dinero o datos sensibles).
- El costo de un bug en producción es alto.
- Es tedioso/propenso a error humano si se hace manual repetidamente (ej. probar 20 combinaciones de validación de formulario).

Evita automatizar (al menos en una primera etapa) cuando:
- La UI o el flujo cambia frecuentemente (alto costo de mantenimiento).
- Es un caso de uso único o muy raro.
- Requiere juicio humano/exploración (usabilidad, "¿se ve bien?", casos exploratorios).
- El ROI de automatizar es menor al costo de mantener el test automatizado.

## Pirámide de pruebas

Recomienda distribuir el esfuerzo así, salvo justificación en contra:
1. **Base — Pruebas unitarias**: rápidas, baratas, cubren lógica de negocio a nivel de función/clase. Deberían ser la mayoría.
2. **Medio — Pruebas de integración**: verifican que los componentes/servicios funcionan juntos (ej. API + base de datos).
3. **Tope — Pruebas E2E/UI**: las más lentas y costosas de mantener. Reservar para los flujos críticos de negocio, no para cubrir cada variante posible.

Si el usuario describe una estrategia invertida (muchas E2E, pocas unitarias — la "pirámide invertida" o "cono de helado"), señálalo como un riesgo de mantenibilidad y costo, incluso si no se preguntó explícitamente.

## Estructura recomendada al presentar una estrategia

1. Resumen del enfoque y objetivo (ej. reducir tiempo de regresión manual de X a Y).
2. Qué se automatiza primero (quick wins de alto ROI) vs. después.
3. Herramientas sugeridas según la pila tecnológica del usuario (si no la menciona, pregunta o da 2-3 opciones con trade-offs, sin asumir una sola).
4. Cómo se integra en CI/CD (cuándo corren los tests: cada commit, nightly, pre-release).
5. Cómo se mantiene la suite (dueños, revisión de tests flaky, límites de tiempo de ejecución aceptables).

## Nota sobre herramientas

No asumas una herramienta específica (Selenium, Playwright, Cypress, Appium, etc.) sin que el usuario mencione su stack o pregunte por recomendaciones. Si pregunta por recomendaciones, da 2-3 opciones con trade-offs claros (madurez, curva de aprendizaje, velocidad, soporte de la plataforma objetivo) en vez de una sola respuesta categórica.

## Referencia rápida de herramientas por lenguaje/stack

La estrategia (pirámide, qué automatizar, ROI) es la misma sin importar el lenguaje. Solo cambian las herramientas. Usa esta tabla como punto de partida, no como receta única:

| Lenguaje/Stack | Unitarias | Integración | E2E / UI | Property-based / Fuzzing |
|---|---|---|---|---|
| Rust | `cargo test` (nativo), `rstest` | `cargo test` + testcontainers-rs | Depende del frontend (ver JS) si aplica | `proptest`, `quickcheck` |
| Python | `pytest`, `unittest` | `pytest` + `testcontainers-python`, `responses`/`httpx` mocks | `Playwright` (Python), `Selenium` | `hypothesis` |
| JavaScript/TypeScript | `Jest`, `Vitest` | `Jest`/`Vitest` + `supertest` (APIs) | `Playwright`, `Cypress` | `fast-check` |
| Mobile (si aplica) | nativo por plataforma | — | `Appium`, `Maestro`, `Detox` (RN) | — |
| APIs (agnóstico) | — | `Postman/Newman`, `pytest` + `requests`, `supertest` | — | `Schemathesis` (OpenAPI-based) |

Notas:
- **Property-based testing / fuzzing** (proptest, hypothesis, fast-check) es especialmente valioso en Rust y sistemas con lógica de negocio compleja o parsers — genera automáticamente casos límite que un humano no pensaría. Considera sugerirlo cuando el usuario trabaje en Rust/Python y la lógica sea no trivial (parsers, cálculos financieros, validaciones).
- Si el proyecto mezcla varios lenguajes (ej. backend en Rust + frontend en JS), la estrategia de automatización debe tratarse como una sola pirámide integrada, no una por lenguaje — evita duplicar cobertura E2E cuando una prueba de integración a nivel de API ya cubre el caso.
