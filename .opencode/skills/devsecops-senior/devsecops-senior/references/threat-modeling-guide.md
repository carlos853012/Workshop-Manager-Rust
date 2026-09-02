# Guía de Threat Modeling

## Proceso recomendado

1. **Modela el sistema**: identifica componentes, flujos de datos, y límites de confianza (dónde cambia el nivel de privilegio o dominio de control — ej. entre el navegador del usuario y tu servidor, entre tu servidor y un servicio de terceros).
2. **Identifica activos**: qué datos o capacidades serían valiosas para un atacante (datos personales, credenciales, dinero, capacidad de ejecutar código).
3. **Identifica amenazas por componente/flujo usando STRIDE**:
   - **S**poofing: ¿alguien puede hacerse pasar por otro usuario o sistema?
   - **T**ampering: ¿alguien puede alterar datos en tránsito o en reposo sin autorización?
   - **R**epudiation: ¿alguien puede negar haber hecho una acción por falta de registro/evidencia?
   - **I**nformation disclosure: ¿alguien puede ver información que no debería?
   - **D**enial of service: ¿alguien puede hacer el sistema no disponible?
   - **E**levation of privilege: ¿alguien puede obtener más permisos de los que debería tener?
4. **Prioriza por probabilidad x impacto**: no todas las amenazas identificadas merecen el mismo esfuerzo de mitigación.
5. **Define mitigaciones concretas** para las amenazas priorizadas — cada amenaza debería tener una mitigación explícita o una decisión consciente de aceptar el riesgo (documentada, no implícita).

## Formato de salida sugerido

| Componente/Flujo | Amenaza (categoría STRIDE) | Descripción concreta | Probabilidad | Impacto | Mitigación propuesta |
|---|---|---|---|---|---|
| | | | Alta/Media/Baja | Alta/Media/Baja | |

## Errores comunes a evitar

- Hacer threat modeling demasiado abstracto ("alguien podría hackear el sistema") sin amenazas concretas y accionables.
- Enumerar amenazas sin priorizar — un threat model con 40 amenazas sin orden de prioridad no ayuda a decidir qué hacer primero.
- Olvidar amenazas internas (empleados con acceso legítimo que abusan de él) y centrarse solo en atacantes externos.
- No revisar el threat model cuando el sistema cambia significativamente — es un documento vivo, no un ejercicio de una sola vez.
