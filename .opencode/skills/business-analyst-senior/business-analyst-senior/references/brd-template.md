# Plantilla de Documento de Requisitos (BRD/PRD)

Adapta el nivel de detalle al tamaño del proyecto — no fuerces todas las secciones si el proyecto es chico.

1. **Contexto y problema de negocio** — Qué problema u oportunidad motiva este proyecto/feature. Debe poder explicarse sin mencionar la solución técnica.
2. **Objetivos y métricas de éxito** — Cómo se sabrá que el proyecto fue exitoso (idealmente medible).
3. **Alcance**
   - Dentro de alcance.
   - Fuera de alcance (explícito — evita interpretaciones distintas después).
4. **Stakeholders** — Quién decide, quién opina, quién ejecuta, quién se ve afectado. Ver `stakeholder-analysis-template.md`.
5. **Requisitos funcionales** — Qué debe hacer el sistema, agrupado por área/módulo. Idealmente numerados para trazabilidad.
6. **Requisitos no funcionales** — Performance, seguridad, disponibilidad, compatibilidad, escalabilidad, cumplimiento normativo si aplica.
7. **Supuestos** — Qué se está asumiendo como verdadero sin haberlo confirmado.
8. **Riesgos** — Qué podría hacer fallar el proyecto y qué tan probable/grave es.
9. **Dependencias** — De qué otros equipos, sistemas o proyectos depende esto.
10. **Glosario** — Si hay términos de negocio específicos que no todos los lectores conocerán.

## Notas de uso

- Los requisitos funcionales deben ser verificables — evita frases como "el sistema debe ser intuitivo" sin un criterio medible.
- Si el usuario ya tiene una solución técnica en mente, igual documenta el problema de negocio por separado — permite validar que la solución propuesta realmente lo resuelve.
- La sección de "fuera de alcance" es la que más previene conflictos después — no la omitas aunque parezca redundante.
