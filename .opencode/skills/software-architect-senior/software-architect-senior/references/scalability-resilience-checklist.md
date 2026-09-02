# Checklist de Escalabilidad y Resiliencia

Usa este checklist al diseñar o revisar un sistema. No todo aplica a todos los sistemas — prioriza según la criticidad y escala esperada.

## Puntos únicos de fallo (SPOF)

- [ ] ¿Hay algún componente sin redundancia cuya caída tumba todo el sistema (base de datos única, servidor único, cola única)?
- [ ] ¿El sistema puede degradarse parcialmente (algunas funciones siguen funcionando) en vez de caer completamente si un componente falla?

## Escalabilidad

- [ ] ¿El sistema escala horizontalmente (agregar más instancias) o solo verticalmente (servidor más grande, con límite físico)?
- [ ] ¿Hay estado guardado en memoria de una instancia específica que impida escalar horizontalmente sin problemas (sticky sessions no gestionadas, cachés locales no compartidas)?
- [ ] ¿La base de datos es previsiblemente el cuello de botella a la escala esperada? ¿Hay plan de caching, réplicas de lectura, o sharding si se necesita?

## Manejo de fallos y resiliencia

- [ ] Llamadas a servicios externos tienen timeout definido (nunca esperar indefinidamente).
- [ ] Hay reintentos con backoff para fallos transitorios, pero con límite (evitar reintentos infinitos que agraven una caída).
- [ ] Circuit breakers o mecanismos similares para evitar que un servicio caído tumbe en cascada a los que dependen de él.
- [ ] Colas o buffers para absorber picos de carga en vez de rechazar o perder requests.

## Datos

- [ ] Estrategia de backup y de recuperación ante desastres definida (no solo "asumimos que la nube no falla").
- [ ] Consistencia de datos: ¿el sistema requiere consistencia fuerte (transaccional) o puede tolerar consistencia eventual? ¿el diseño refleja esa decisión conscientemente?

## Observabilidad (prerequisito para operar a escala)

- [ ] Logs estructurados y correlacionables entre servicios (trace ID compartido).
- [ ] Métricas de las señales clave (latencia, tasa de error, saturación) disponibles antes de que el sistema esté en producción con carga real.

## Al reportar hallazgos

Prioriza por: (1) probabilidad de que ocurra dado el contexto real del sistema, y (2) impacto si ocurre. No trates un SPOF en un componente crítico de pagos igual que uno en una página informativa poco visitada.
