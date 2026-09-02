# Checklist basado en OWASP Top 10 (marco de referencia)

Usa esto como marco de revisión, adaptando la profundidad al contexto (no todo aplica a toda app). Basado en las categorías de OWASP Top 10, sin asumir que la lista exacta es idéntica a la versión más reciente publicada — para un mapeo oficial actualizado, remite al usuario a owasp.org.

## Control de acceso roto
- [ ] ¿Se valida en el backend que el usuario autenticado tiene permiso sobre el recurso específico que solicita (no solo que está autenticado)?
- [ ] ¿Es posible acceder a recursos de otros usuarios cambiando un ID en la URL o request (IDOR)?
- [ ] ¿Las rutas/endpoints de administración están protegidas más allá de "no estar linkeadas" (seguridad por oscuridad no es suficiente)?

## Fallos criptográficos
- [ ] ¿Los datos sensibles se cifran en tránsito (HTTPS/TLS) y en reposo cuando corresponde?
- [ ] ¿Las contraseñas se almacenan con hashing apropiado (bcrypt/argon2/scrypt), nunca en texto plano ni con hashes rápidos como MD5/SHA1 sin salt?
- [ ] ¿Se evita implementar criptografía propia en vez de usar librerías probadas?

## Inyección
- [ ] ¿Las queries a bases de datos usan parámetros/prepared statements, no concatenación de strings con input del usuario?
- [ ] ¿Se validan/escapan inputs antes de usarlos en comandos de sistema, queries, o construcción de HTML (XSS)?

## Diseño inseguro
- [ ] ¿Se consideró el abuso del flujo de negocio, no solo su uso correcto (ej. ¿qué pasa si alguien aplica un cupón de descuento 1000 veces?)?

## Configuración de seguridad incorrecta
- [ ] ¿Hay configuraciones por defecto sin cambiar (credenciales default, puertos de administración expuestos)?
- [ ] ¿Los mensajes de error en producción exponen detalles internos (stack traces, versiones de software, estructura de base de datos)?
- [ ] ¿Los headers de seguridad HTTP están configurados (CSP, X-Content-Type-Options, etc.) cuando aplica a una app web?

## Componentes vulnerables/desactualizados
- [ ] ¿Hay un proceso de escaneo y actualización de dependencias, o se depende de revisión manual esporádica?
- [ ] Ver `dependency-security-checklist.md` para más detalle.

## Fallos de identificación y autenticación
- [ ] ¿Hay protección contra fuerza bruta en login (rate limiting, bloqueo temporal)?
- [ ] ¿Los tokens de sesión son suficientemente largos/aleatorios y expiran apropiadamente?
- [ ] ¿Se ofrece (o exige, según el contexto) autenticación multifactor para cuentas sensibles/administrativas?

## Fallos de integridad de software y datos
- [ ] ¿Se verifican firmas/checksums de dependencias y artefactos de build en el pipeline de CI/CD?
- [ ] ¿La deserialización de datos no confiables está controlada (evitar deserialización insegura que permita ejecución de código)?

## Fallos de logging y monitoreo de seguridad
- [ ] ¿Se registran eventos de seguridad relevantes (logins fallidos, cambios de permisos, accesos a datos sensibles)?
- [ ] ¿Hay alertas para patrones anómalos (muchos intentos fallidos, accesos desde ubicaciones inusuales)?

## Server-Side Request Forgery (SSRF)
- [ ] Si el sistema hace requests HTTP a URLs proporcionadas por el usuario, ¿se valida/restringe el destino para evitar que acceda a recursos internos (metadata de la nube, servicios internos)?

## Al reportar hallazgos

Prioriza por explotabilidad real en el contexto del sistema evaluado, no por posición en esta lista. Un hallazgo teórico sin vector de explotación práctico es menor prioridad que uno directamente explotable, aunque ambos pertenezcan a la misma categoría OWASP.
