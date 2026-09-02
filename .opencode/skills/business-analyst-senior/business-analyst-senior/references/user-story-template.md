# Plantilla de Historia de Usuario

## Formato

**Como** [rol/tipo de usuario]
**Quiero** [acción o funcionalidad]
**Para** [beneficio u objetivo de negocio real — no relleno]

## Criterios de aceptación (formato Given/When/Then)

- **Given** [contexto/precondición]
- **When** [acción del usuario o evento del sistema]
- **Then** [resultado esperado]

Incluye al menos:
1. Un escenario de camino feliz.
2. Un escenario negativo o de error relevante.
3. Un escenario de límite/borde si aplica (ej. lista vacía, permisos insuficientes).

## Ejemplo

**Como** cliente registrado
**Quiero** recibir una notificación cuando mi pedido cambie de estado
**Para** saber en qué momento llegará mi compra sin tener que consultar manualmente

Criterios de aceptación:
- Given un pedido en estado "En preparación", When el estado cambia a "Enviado", Then el cliente recibe una notificación push y un email dentro de los siguientes 5 minutos.
- Given un cliente que desactivó las notificaciones, When su pedido cambia de estado, Then no recibe push pero sí puede ver el estado actualizado en la app.
- Given un pedido cancelado, When el estado cambia a "Cancelado", Then el cliente recibe una notificación explicando el motivo si está disponible.

## Señales de alerta al redactar historias

- **Beneficio genérico o ausente** ("para que funcione mejor") → indaga el objetivo real de negocio.
- **Historia demasiado grande** (mezcla varias funcionalidades independientes) → sugiere dividirla en historias más chicas, cada una entregable de forma independiente.
- **Criterios de aceptación que solo cubren el camino feliz** → agrega al menos un caso negativo/de error.
- **Solución disfrazada de historia** ("Como admin quiero un checkbox que...") → el checkbox es una solución, no una necesidad; anota cuál es la necesidad real detrás si es posible.
