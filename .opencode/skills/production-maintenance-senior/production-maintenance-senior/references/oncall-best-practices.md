# Buenas Prácticas de On-Call

## Diagnóstico de la salud del on-call actual

Antes de proponer cambios, entiende:
- ¿Cuántas alertas dispara un turno típico? ¿Cuántas requirieron acción real vs. fueron ruido/falsos positivos?
- ¿Cuántas veces se interrumpe el sueño de la persona de guardia en un turno típico?
- ¿Hay "héroes" que resuelven todo porque son los únicos que entienden ciertos sistemas (bus factor bajo)?

## Principios

- **Reducir ruido antes que agregar gente**: si la mayoría de las alertas no son accionables, agregar más personas a la rotación no resuelve el problema de fondo — solo lo distribuye.
- **Alertas accionables únicamente fuera de horario laboral**: alertas de baja severidad o que no requieren acción inmediata deberían esperar al horario laboral (dashboard/ticket), no despertar a nadie.
- **Runbooks para incidentes comunes**: reduce el tiempo de respuesta y el estrés de decidir qué hacer bajo presión a las 3am.
- **Rotación justa y sostenible**: evitar que la misma persona esté de guardia desproporcionadamente más que el resto del equipo.
- **Compensación/tiempo libre por incidentes fuera de horario**: reconocer el costo real de ser interrumpido, no tratarlo como gratis.

## Reducir el "bus factor" en el on-call

- Documentar el conocimiento tribal (cómo diagnosticar/resolver problemas comunes) en vez de depender de que una persona específica esté disponible.
- Rotar activamente para que más de una persona gane experiencia con cada sistema, no dejar que el mismo experto resuelva todo indefinidamente (esto además lo agota a él/ella específicamente).

## Métricas útiles para evaluar el on-call a lo largo del tiempo

- Número de alertas por turno (tendencia, no solo el número absoluto de un turno).
- % de alertas que resultaron en acción real vs. ruido.
- Tiempo promedio de resolución (MTTR) y su tendencia.
- Interrupciones de sueño por turno (proxy de sostenibilidad humana, no solo técnica).

## Señal de alerta a comunicar si aparece

Si el equipo reporta "estamos acostumbrados, así es el trabajo" ante una carga de on-call insostenible, vale la pena señalar explícitamente que la normalización de una carga alta no la vuelve sostenible — suele preceder a burnout o rotación de personal, que termina costando más que arreglar el problema de raíz.
