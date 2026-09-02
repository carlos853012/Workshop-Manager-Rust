# Checklist de Seguridad de Dependencias

## Herramientas de escaneo por lenguaje

| Lenguaje | Herramienta de auditoría |
|---|---|
| Rust | `cargo audit`, `cargo deny` |
| Python | `pip-audit`, `safety` |
| JavaScript/TypeScript | `npm audit`, `pnpm audit`, `yarn audit`, Snyk |
| Multi-lenguaje/contenedores | Trivy, Grype, Snyk, Dependabot/Renovate para actualización automática de PRs |

## Proceso recomendado

- [ ] Escaneo de dependencias integrado en CI/CD (no solo ejecutado manualmente de vez en cuando).
- [ ] Proceso para actualizar dependencias regularmente (ej. Dependabot/Renovate con PRs automáticos), no solo cuando se descubre una vulnerabilidad.
- [ ] Política clara de cuánto tiempo se tolera una vulnerabilidad conocida sin parchear, según severidad (ej. crítica: días; baja: próximo ciclo de mantenimiento regular).

## Cómo priorizar una vulnerabilidad reportada

Evalúa, en este orden:
1. **¿Es explotable en el contexto real de uso?** Si el proyecto no usa la función/módulo vulnerable, el riesgo práctico es mucho menor que el score CVSS sugiere.
2. **¿Está expuesta externamente?** Una vulnerabilidad en una dependencia usada solo en scripts internos de build es menor prioridad que una en el path de requests de un servidor público.
3. **¿Hay un exploit público conocido?** Aumenta la urgencia significativamente.
4. **¿Qué tan grande es el esfuerzo de actualizar?** Un major version bump con breaking changes requiere más planificación que un patch menor.

No trates el score CVSS como la única fuente de verdad — es un punto de partida, no una decisión automática de prioridad.

## Al actualizar dependencias

- Revisa el changelog de la nueva versión, no solo actualices ciegamente — especialmente para major versions.
- Ten tests que den confianza de que la actualización no rompió nada (si no hay tests suficientes, dilo como riesgo antes de actualizar dependencias core).
