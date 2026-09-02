# Checklist de Preparación y Respuesta a Incidentes de Seguridad

## Fases de respuesta

### 1. Detección e identificación
- [ ] ¿Cómo se detectó el incidente (alerta automática, reporte de usuario, hallazgo manual)?
- [ ] ¿Qué sistemas/datos están potencialmente afectados?
- [ ] ¿El incidente está confirmado o es una sospecha que requiere investigación?

### 2. Contención
- [ ] Contención a corto plazo: aislar el sistema afectado sin perder evidencia (ej. desconectar de la red en vez de apagar, para preservar memoria/logs si es relevante).
- [ ] ¿Es necesario rotar credenciales/secretos potencialmente comprometidos inmediatamente?
- [ ] ¿Es necesario revocar sesiones activas de usuarios potencialmente afectados?

### 3. Erradicación
- [ ] Identificar y eliminar la causa raíz (vulnerabilidad explotada, credencial filtrada, configuración incorrecta).
- [ ] Verificar que no queden puntos de persistencia del atacante (cuentas creadas, backdoors, tareas programadas maliciosas).

### 4. Recuperación
- [ ] Restaurar sistemas afectados desde un estado conocido como limpio.
- [ ] Monitoreo reforzado post-incidente para detectar reincidencia.

### 5. Lecciones aprendidas (post-mortem)
- [ ] Documentar línea de tiempo del incidente sin buscar culpables individuales (blameless post-mortem).
- [ ] Identificar qué controles hubieran prevenido o detectado el incidente antes.
- [ ] Convertir los aprendizajes en acciones concretas con dueño y fecha, no solo observaciones.

## Consideraciones adicionales

- [ ] **Notificación**: dependiendo de la jurisdicción y tipo de datos afectados, puede existir obligación legal de notificar a usuarios afectados o autoridades (ej. regulaciones de protección de datos). Esto requiere involucrar a legal/compliance — esta skill no reemplaza asesoría legal.
- [ ] **Comunicación**: definir quién comunica qué y a quién (interno, usuarios afectados, público) — evitar comunicación improvisada durante el incidente.

## Nota

Esta guía es un marco general de referencia, no un plan de respuesta a incidentes completo y verificado para una organización específica — para sistemas críticos, se recomienda un plan de respuesta a incidentes formal, revisado periódicamente y practicado (tabletop exercises), no solo este checklist.
