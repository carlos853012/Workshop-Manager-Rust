# Plantilla de Registro de Riesgos

## Tabla de riesgos

| ID | Riesgo | Categoría | Probabilidad | Impacto | Prioridad (Prob x Impacto) | Plan de mitigación | Plan de contingencia (si ocurre igual) | Dueño |
|---|---|---|---|---|---|---|---|---|
| R-01 | | Técnico/Recursos/Externo/Alcance | Alta/Media/Baja | Alta/Media/Baja | | | | |

## Categorías comunes de riesgo en proyectos de software

- **Técnico**: tecnología nueva sin experiencia del equipo, deuda técnica que puede complicar el desarrollo, dependencias de sistemas legacy.
- **Recursos**: dependencia de una sola persona (bus factor), disponibilidad del equipo (vacaciones, rotación), habilidades faltantes.
- **Externo**: dependencia de proveedores/terceros, dependencia de otros equipos internos, cambios regulatorios.
- **Alcance**: requisitos ambiguos o cambiantes, stakeholders con expectativas no alineadas, scope creep no gestionado.
- **Cronograma**: estimaciones optimistas, dependencias entre tareas no consideradas, fechas fijas impuestas sin validación técnica.

## Cómo priorizar riesgos

Prioridad = Probabilidad × Impacto. Enfoca el esfuerzo de mitigación en los riesgos de alta prioridad (alta probabilidad Y alto impacto), y al menos ten un plan de contingencia simple para riesgos de alto impacto aunque su probabilidad sea baja.

## Buenas prácticas

- Revisa el registro de riesgos periódicamente (no es un documento de "una sola vez al inicio del proyecto") — los riesgos cambian a medida que el proyecto avanza.
- Un riesgo sin dueño asignado tiende a no gestionarse — siempre asigna responsable de monitorear y ejecutar la mitigación.
- Diferencia un riesgo (algo que podría pasar) de un problema (algo que ya está pasando) — un problema ya materializado necesita gestión de incidente/issue, no solo un plan de mitigación preventivo.
