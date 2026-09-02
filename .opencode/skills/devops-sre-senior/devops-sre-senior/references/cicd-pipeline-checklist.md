# Checklist de Pipeline de CI/CD

## Build

- [ ] El build es reproducible (mismas dependencias exactas, idealmente con lockfiles: `Cargo.lock`, `poetry.lock`/`requirements.txt` con hashes, `package-lock.json`/`pnpm-lock.yaml`).
- [ ] Caché de dependencias configurado para acelerar builds repetidos.
- [ ] Artefactos versionados de forma inmutable (tag/hash específico, no solo `latest`).

## Test

- [ ] Tests rápidos (unitarias) corren primero — fail fast antes de gastar tiempo en tests lentos.
- [ ] Tests de integración/E2E corren en un entorno aislado y reproducible (containers efímeros, no un ambiente compartido que puede tener estado sucio).
- [ ] El pipeline falla si la cobertura de tests cae por debajo de un umbral acordado (si el equipo usa ese criterio) o si tests críticos fallan — no se permite mergear con tests rotos.

## Seguridad en el pipeline

- [ ] Escaneo de dependencias (ver skill de seguridad) integrado, no manual.
- [ ] Secretos gestionados por el sistema de secrets del CI (nunca hardcodeados en el archivo de pipeline ni en variables de entorno visibles en logs).
- [ ] Permisos del pipeline con el mínimo scope necesario (ej. token de despliegue que solo puede desplegar, no administrar toda la cuenta de nube).

## Despliegue

- [ ] Despliegue a staging automático tras pasar tests, con smoke tests post-deploy automáticos.
- [ ] Despliegue a producción requiere aprobación explícita o gate automático (ej. solo si staging pasó smoke tests), según el nivel de riesgo del proyecto.
- [ ] Rollback automatizado o al menos documentado como un paso rápido y conocido por el equipo, no improvisado en el momento de un incidente.
- [ ] Migraciones de base de datos son compatibles hacia atrás durante el despliegue (para permitir rollback de la aplicación sin romper por incompatibilidad de esquema).

## Observabilidad del propio pipeline

- [ ] Notificaciones claras de éxito/fallo al equipo (no solo un log que nadie revisa).
- [ ] Tiempo de ejecución del pipeline monitoreado — pipelines que se vuelven cada vez más lentos afectan la velocidad de todo el equipo.

## Señales de alerta a comunicar si aparecen

- Despliegues manuales frecuentes fuera del pipeline ("por esta vez lo hago a mano") — indica que el pipeline no cubre un caso real y necesita ajustarse.
- Tests flaky (fallan intermitentemente sin relación con el cambio) que el equipo ignora o re-ejecuta sin investigar — erosionan la confianza en el pipeline con el tiempo.
