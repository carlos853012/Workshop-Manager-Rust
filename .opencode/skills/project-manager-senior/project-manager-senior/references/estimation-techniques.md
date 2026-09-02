# Técnicas de Estimación

## Story Points (relativo, dentro de un equipo estable)

- Se estima el tamaño relativo del trabajo (esfuerzo + complejidad + incertidumbre combinados), no tiempo directo.
- Requiere historial del equipo (velocity) para traducir puntos a tiempo real — no tiene sentido comunicar story points a alguien fuera del equipo que no tiene ese contexto.
- Útil para: planning interno de un equipo con cierta madurez trabajando junto.
- Escalas comunes: Fibonacci (1, 2, 3, 5, 8, 13...) — los saltos más grandes en números altos reflejan que la incertidumbre crece con el tamaño de la tarea.

## Estimación en tiempo con rangos (para comunicación externa/stakeholders)

En vez de un número único, comunica un rango:
- **Optimista**: si todo sale bien, sin sorpresas.
- **Probable**: la estimación más realista considerando fricción típica.
- **Pesimista**: si aparecen los problemas comunes (dependencias externas lentas, bugs inesperados, ausencias del equipo).

Cuanto más lejos en el tiempo o menos definido esté el trabajo, más ancho debería ser el rango — no fuerces precisión falsa en estimaciones de trabajo poco refinado.

## Técnicas de estimación colaborativa

- **Planning Poker**: cada miembro del equipo estima en privado (evita anclaje a la primera opinión expresada), se revela simultáneamente, se discuten las diferencias grandes.
- **T-shirt sizing (XS/S/M/L/XL)**: útil para estimaciones muy tempranas de alto nivel, cuando el detalle no está definido todavía.

## Señales de una estimación poco confiable

- El trabajo no está desglosado (una sola tarea gigante "implementar la feature X" sin desglosar en partes entendibles).
- Se estima sin el equipo que va a ejecutar el trabajo (estimaciones impuestas desde arriba sin validación técnica).
- La fecha se decidió primero y la estimación se ajustó para que "cuadre" — esto no es una estimación, es una negociación de alcance disfrazada.
- No se considera el historial real del equipo (velocity pasada) al proyectar cuánto se puede lograr en un período.

## Cómo comunicar estimaciones con honestidad

- Sé explícito sobre qué se asumió al estimar (ej. "asumiendo que el equipo actual, sin ausencias significativas, y sin bloqueos externos").
- Si la estimación cambia con nueva información, comunica el cambio y por qué, en vez de mantener silenciosamente una fecha que ya se sabe poco realista.
