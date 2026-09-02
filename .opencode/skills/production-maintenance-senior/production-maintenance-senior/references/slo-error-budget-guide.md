# Guía de SLIs, SLOs y Error Budgets

## Definiciones

- **SLI (Service Level Indicator)**: métrica concreta y medible de comportamiento del sistema desde la perspectiva del usuario. Ejemplos: % de requests HTTP con código 2xx/3xx, latencia p99 de un endpoint, % de jobs batch completados a tiempo.
- **SLO (Service Level Objective)**: el objetivo interno para ese SLI durante una ventana de tiempo (ej. "99.9% de requests exitosas en ventana móvil de 30 días").
- **SLA (Service Level Agreement)**: compromiso externo/contractual, típicamente con consecuencias (créditos, penalidades) si no se cumple. Suele ser más laxo que el SLO interno, para dejar margen de maniobra.
- **Error budget**: 100% - SLO. Si el SLO es 99.9%, el error budget es 0.1% de fallos permitidos en la ventana. Se usa como semáforo: budget disponible → se puede seguir lanzando cambios con velocidad normal; budget agotado → congelar features nuevas y priorizar estabilidad hasta recuperar margen.

## Cómo elegir buenos SLIs

- Deben reflejar la experiencia real del usuario, no solo lo que es fácil de medir internamente (ej. "el servidor está up" no es lo mismo que "el usuario puede completar su compra").
- Prefiere medir en el punto más cercano posible al usuario (ej. desde el load balancer o el cliente, no solo desde dentro del servicio).
- No definas demasiados SLIs — 2-4 por servicio suele ser suficiente para capturar lo que realmente importa (disponibilidad, latencia, y a veces corrección/completitud).

## Cómo elegir un SLO razonable

- Empieza por lo que el negocio/usuarios realmente necesitan, no por "cuanto más alto mejor". 99.99% de disponibilidad es mucho más caro y complejo de lograr que 99.9%, y no siempre vale la pena el costo adicional.
- Si no hay datos históricos, empieza con un SLO conservador basado en el comportamiento actual del sistema y ajusta con el tiempo, en vez de inventar un número aspiracional sin base.
- Revisa los SLOs periódicamente — no son inmutables, deben reflejar las expectativas actuales del negocio y usuarios.

## Uso del error budget en la práctica

- Cuando el error budget está disponible: el equipo puede priorizar velocidad (lanzar features, experimentar).
- Cuando el error budget se agota: se prioriza estabilidad (fix de bugs, reducción de riesgo) sobre nuevas features, hasta recuperar margen.
- Esto da un criterio objetivo para la eterna tensión "velocidad vs. estabilidad", en vez de decidirlo solo por intuición o política interna.

## Errores comunes

- Definir SLOs sin conectarlos a ninguna decisión real (si nada cambia cuando se agota el error budget, el SLO es solo un número decorativo).
- Copiar SLOs de otra empresa/servicio sin considerar el contexto propio (usuarios, criticidad, presupuesto).
- Medir demasiadas cosas sin priorizar cuáles importan realmente para la experiencia del usuario.
