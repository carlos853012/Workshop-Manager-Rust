# Frameworks de Priorización

Elige el framework según cuántos datos hay disponibles y qué tan formal necesita ser la priorización.

## MoSCoW (rápido, poca data necesaria)

Clasifica cada item en:
- **Must have**: sin esto, el release/proyecto no tiene sentido o no cumple su objetivo mínimo.
- **Should have**: importante pero no bloqueante; puede quedar para una siguiente iteración si hay que recortar.
- **Could have**: deseable, bajo impacto si se omite.
- **Won't have (esta vez)**: explícitamente fuera de esta iteración — evita ambigüedad de "tal vez después".

Úsalo cuando: hay poco tiempo, pocos datos cuantitativos, o el objetivo es alinear rápido a un equipo/stakeholders.

## RICE (más riguroso, requiere estimaciones)

Score = (Reach × Impact × Confidence) / Effort

- **Reach**: a cuántos usuarios/eventos afecta en un período de tiempo.
- **Impact**: qué tan grande es el efecto por usuario/evento (escala típica: 3=masivo, 2=alto, 1=medio, 0.5=bajo, 0.25=mínimo).
- **Confidence**: qué tan seguros estamos de las estimaciones anteriores (100%, 80%, 50%).
- **Effort**: persona-tiempo estimado para implementarlo.

Úsalo cuando: hay varias iniciativas compitiendo por los mismos recursos y se necesita justificar la priorización con números, no solo intuición.

Si el usuario no tiene datos para todas las variables, no fuerces el cálculo — usa un framework más simple (MoSCoW o Value vs. Effort) o estima con rangos y transparenta la incertidumbre.

## Value vs. Effort (2x2, visual, poca data)

Grafica cada item en una matriz:
- **Alto valor / Bajo esfuerzo** → hacer primero ("quick wins").
- **Alto valor / Alto esfuerzo** → planificar (proyectos grandes que valen la pena).
- **Bajo valor / Bajo esfuerzo** → hacer si sobra tiempo ("fill-ins").
- **Bajo valor / Alto esfuerzo** → evitar o cuestionar por qué está en el backlog.

Úsalo cuando: se necesita una visualización rápida y el equipo entiende bien el contexto sin necesidad de números exactos.

## Al presentar una priorización

- Sé explícito sobre el framework elegido y por qué.
- Muestra el resultado en tabla ordenada, no como lista de justificaciones sueltas.
- Señala si algún item de "alta prioridad" según el framework en realidad no tiene un objetivo de negocio claro detrás — la priorización no reemplaza el juicio crítico sobre si algo debería estar en el backlog.
