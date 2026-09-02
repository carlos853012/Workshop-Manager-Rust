# Guía de Diseño de APIs

## Principios generales

- Diseña pensando en el consumidor de la API, no en la estructura interna de la base de datos.
- Sé consistente: mismos patrones de nombrado, paginación, manejo de errores en todos los endpoints de un mismo servicio.
- Versiona desde el inicio (aunque sea `/v1/`) — agregar versionado después de tener consumidores es mucho más costoso.

## REST — checklist de diseño

- [ ] Recursos como sustantivos, no verbos (`/orders`, no `/getOrders`).
- [ ] Uso correcto de verbos HTTP (GET no debe tener efectos secundarios; POST para crear; PUT/PATCH para actualizar; DELETE para eliminar).
- [ ] Códigos de estado HTTP correctos y consistentes (200/201/204 éxito; 400 input inválido; 401/403 auth; 404 no encontrado; 409 conflicto; 429 rate limit; 5xx error de servidor).
- [ ] Formato de error consistente en todos los endpoints (ej. `{ "error": { "code": "...", "message": "..." } }`), no un formato distinto por endpoint.
- [ ] Paginación definida para listas (cursor-based para datasets grandes/cambiantes; offset-based aceptable para datasets chicos y estables).
- [ ] Filtros y ordenamiento vía query params, consistentes entre endpoints similares.
- [ ] Idempotencia en operaciones que pueden reintentarse (ej. usar `Idempotency-Key` en creación de pagos/pedidos).
- [ ] Autenticación/autorización explícita por endpoint, no asumida.
- [ ] Rate limiting documentado si aplica, con headers informativos (`X-RateLimit-*`).

## GraphQL — consideraciones

- Diseña el schema pensando en los casos de uso del consumidor, evitando el problema de "over-fetching" que GraphQL busca resolver, pero sin caer en queries N+1 en el resolver (usar dataloaders/batching).
- Define límites de profundidad/complejidad de queries para evitar abuso (una query maliciosa puede ser muy costosa).
- Versionado en GraphQL es distinto a REST: se prefiere evolucionar el schema (deprecar campos) en vez de versionar toda la API.

## gRPC — consideraciones

- Adecuado para comunicación interna entre servicios de alto rendimiento, no ideal como API pública de cara a clientes externos sin gateway intermedio.
- Define los `.proto` pensando en compatibilidad hacia adelante (campos opcionales, no reordenar/reusar números de campo).

## Al revisar una API existente

Señala explícitamente si encuentras:
- Inconsistencia de convenciones entre endpoints (algunos en camelCase, otros en snake_case; algunos paginan, otros no).
- Ausencia de manejo de errores estandarizado.
- Endpoints que exponen más datos de los necesarios (over-fetching que puede filtrar datos sensibles sin querer).
- Falta de versionado que hará imposible evolucionar la API sin romper consumidores existentes.
