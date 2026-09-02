# Plantilla de Architecture Decision Record (ADR)

## ADR-XXX: [Título corto de la decisión]

**Estado**: Propuesta / Aceptada / Rechazada / Reemplazada por ADR-YYY

**Contexto**
¿Qué situación o problema motiva esta decisión? Incluye restricciones relevantes (técnicas, de negocio, de equipo, de tiempo). Debe poder entenderse sin conocer ya la decisión tomada.

**Opciones consideradas**

Para cada opción:
- Descripción breve.
- Ventajas.
- Desventajas.
- En qué contexto sería la mejor opción (aunque no se elija aquí).

**Decisión**
Qué se decidió, en una o dos frases claras.

**Justificación**
Por qué esta opción sobre las demás, ligado a los requisitos/contexto específico (no una justificación genérica de "es mejor práctica").

**Consecuencias**

- Positivas: qué se gana con esta decisión.
- Negativas / trade-offs aceptados: qué se sacrifica o qué riesgo se asume conscientemente.
- Qué queda pendiente o requiere revisión futura (ej. "revisar si el volumen de datos supera X").

## Notas de uso

- Un ADR no es un documento para justificar una decisión ya tomada de forma sesgada — debe reflejar honestamente las opciones que se consideraron, incluso las que no se eligieron.
- Los ADRs son inmutables una vez aceptados: si la decisión cambia después, se crea un nuevo ADR que referencia y reemplaza al anterior, no se edita el original.
- Mantén el ADR corto (idealmente una página) — el objetivo es que alguien lo lea en 5 minutos dentro de un año y entienda el "por qué".
