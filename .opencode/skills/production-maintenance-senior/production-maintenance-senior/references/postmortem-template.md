# Plantilla de Postmortem (Blameless)

## Resumen ejecutivo
Qué pasó, cuánto duró, y qué impacto tuvo (usuarios afectados, pérdida de datos si hubo, impacto de negocio) — en 2-3 frases, para alguien que no leerá el resto del documento.

## Línea de tiempo
Cronología detallada de eventos, con timestamps: cuándo empezó el problema (no cuándo se detectó, si son distintos), cuándo se detectó, acciones tomadas, cuándo se resolvió. Incluye tanto lo que funcionó como lo que no durante la respuesta.

## Impacto
- Duración total del incidente.
- Usuarios/sistemas afectados y en qué medida (todos, un porcentaje, una región específica).
- Impacto de negocio si es cuantificable (transacciones perdidas, SLA incumplido, etc.).

## Causa raíz
Usa una técnica como "5 por qués" para llegar más allá del síntoma inmediato hasta una causa sistémica accionable. Ejemplo:
1. ¿Por qué falló el servicio? → Se quedó sin memoria.
2. ¿Por qué se quedó sin memoria? → Un memory leak en el manejo de conexiones.
3. ¿Por qué no se detectó antes? → No había alerta de tendencia de uso de memoria, solo de uso instantáneo alto.
4. ¿Por qué no había esa alerta? → No se consideró en el diseño de observabilidad original.
5. Causa raíz sistémica: el proceso de diseño de nuevos servicios no incluye una revisión de observabilidad como paso obligatorio.

Evita detenerte en "alguien cometió un error" como causa raíz final — pregunta qué en el sistema/proceso permitió que ese error tuviera el impacto que tuvo.

## Qué salió bien
Reconoce explícitamente qué funcionó durante la respuesta (detección rápida, comunicación clara, rollback efectivo) — un postmortem no es solo una lista de fallas.

## Acciones de seguimiento

| Acción | Dueño | Fecha límite | Prioridad |
|---|---|---|---|
| | | | |

Cada acción debe ser concreta y verificable (no "mejorar el monitoreo" sino "agregar alerta de tendencia de memoria al servicio X para el DD/MM").

## Notas de uso

- Comparte el postmortem ampliamente dentro de la organización (no solo con el equipo directamente involucrado) — el valor de aprendizaje se pierde si queda aislado.
- No uses el postmortem para evaluar desempeño individual — si se percibe como una herramienta punitiva, la gente dejará de reportar/documentar honestamente.
