# Guía de Estrategias de Despliegue

## Comparación rápida

| Estrategia | Riesgo de exposición | Complejidad operacional | Costo de infraestructura | Velocidad de rollback |
|---|---|---|---|---|
| Rolling | Medio (afecta % creciente de tráfico) | Baja | Bajo (usa la misma infra) | Media (hay que revertir el rollout) |
| Blue-green | Bajo (todo o nada, pero detectado tarde si no hay monitoreo) | Media | Alto (doble infraestructura temporalmente) | Muy rápida (cambiar el switch de vuelta) |
| Canary | Muy bajo (exposición mínima inicial) | Alta (requiere buena observabilidad y automatización de decisión) | Medio | Rápida si se detecta a tiempo |
| Feature flags | Controlable por segmento de usuario | Media (requiere gestión del ciclo de vida de los flags) | Bajo | Instantánea (desactivar el flag) |

## Cuándo recomendar cada una

- **Rolling**: buen default para equipos pequeños/medianos sin necesidad de canary sofisticado, con downtime tolerable de segundos durante el rollout.
- **Blue-green**: cuando se necesita rollback instantáneo y el presupuesto permite infraestructura duplicada temporalmente, o cuando el cambio no se puede desplegar gradualmente (ej. cambios de esquema no compatibles).
- **Canary**: sistemas críticos con alto tráfico donde detectar un problema con el 1% de usuarios antes que con el 100% justifica la complejidad adicional. Requiere métricas confiables y, idealmente, decisión automática de avanzar/revertir basada en esas métricas.
- **Feature flags**: cuando se quiere separar el riesgo de desplegar código del riesgo de activar una funcionalidad, o cuando se necesita activar funcionalidad para segmentos específicos (beta testers, un cliente específico, etc.).

Estas estrategias no son mutuamente excluyentes — es común combinar feature flags con rolling/canary deployments.

## Prerequisitos antes de adoptar canary o blue-green

- Observabilidad confiable (métricas de error/latencia por versión desplegada, no solo agregadas).
- Automatización del pipeline suficientemente madura para no depender de pasos manuales bajo presión de tiempo.
- Si estos prerequisitos no están, señala que adoptar rolling deployment simple + buen plan de rollback manual es más realista que forzar canary sin las bases necesarias.
