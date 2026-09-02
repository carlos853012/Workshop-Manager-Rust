# Framework de Evaluación de Alternativas Técnicas

## Dimensiones a evaluar (no todas aplican siempre)

- **Ajuste al requisito real**: ¿resuelve el problema que realmente existe, no un problema hipotético futuro?
- **Costo de aprendizaje/operación para el equipo actual**: ¿el equipo ya sabe usarlo o hay que invertir en aprenderlo?
- **Madurez y soporte**: ¿tiene comunidad activa, documentación, tiempo en el mercado? ¿qué pasa si la empresa detrás desaparece?
- **Costo de cambiarlo después** (reversibilidad): ¿es fácil migrar si la elección resulta equivocada, o queda "grabado en piedra"?
- **Costo de infraestructura/licencias** a la escala esperada.
- **Rendimiento** en el patrón de uso específico (no benchmarks genéricos de internet).
- **Ecosistema**: ¿se integra bien con el resto del stack ya elegido?

## Cómo presentar la comparación

Usa una tabla comparativa con las opciones como columnas y las dimensiones relevantes como filas. Evita presentar más de 3-4 opciones — más que eso diluye la decisión en vez de ayudar.

| Dimensión | Opción A | Opción B | Opción C |
|---|---|---|---|
| Ajuste al requisito | | | |
| Curva de aprendizaje del equipo | | | |
| Madurez/soporte | | | |
| Reversibilidad | | | |
| Costo a la escala esperada | | | |

## Errores comunes a señalar cuando aparezcan

- Elegir una tecnología por popularidad o "lo que usa Google/Netflix" sin considerar que el contexto (escala, equipo, presupuesto) es completamente distinto.
- Subestimar el costo de operar algo nuevo (no solo el costo de implementarlo la primera vez).
- Ignorar el costo de reversibilidad: elegir la opción "de moda" para algo que será muy caro de cambiar después sin haber validado que realmente encaja.
- No considerar el "boring technology" a propósito: a veces la opción aburrida y probada es la correcta, especialmente para componentes críticos del sistema.
