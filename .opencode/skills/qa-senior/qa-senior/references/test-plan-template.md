# Plantilla de Plan de Pruebas

Estructura recomendada para un plan de pruebas formal (documento Word/Markdown):

1. **Objetivo del plan** — Qué se va a probar y por qué (contexto del release/feature).
2. **Alcance**
   - Dentro de alcance: qué funcionalidades/módulos se cubren.
   - Fuera de alcance: qué explícitamente no se cubre y por qué (evita malentendidos después).
3. **Estrategia de pruebas**
   - Tipos de prueba a aplicar (funcional, regresión, performance, seguridad básica, usabilidad, compatibilidad).
   - Niveles (unitaria, integración, sistema, aceptación) y quién es responsable de cada uno.
4. **Análisis de riesgo**
   - Tabla de riesgos: área funcional, probabilidad de falla, impacto en negocio, prioridad de testing resultante.
5. **Criterios de entrada** — Qué debe cumplirse antes de empezar a probar (ej. build estable en ambiente de QA, datos de prueba disponibles).
6. **Criterios de salida / Definición de "listo para producción"**
   - % de casos ejecutados, % de casos pasados, bugs críticos/altos abiertos permitidos (idealmente 0 críticos).
7. **Entorno y datos de prueba** — Ambientes usados, necesidad de datos sintéticos o anonimizados (especialmente si hay PII).
8. **Cronograma** — Si aplica, fechas estimadas de ciclos de prueba.
9. **Roles y responsabilidades**
10. **Herramientas** — Gestión de casos, bug tracking, automatización si aplica.
11. **Supuestos y dependencias**

## Notas de uso

- No incluyas secciones que no aportan valor al caso concreto (ej. no fuerces "cronograma" si el usuario solo quiere probar una feature chica). Adapta el nivel de formalidad al tamaño del proyecto.
- La sección de análisis de riesgo es la más valiosa y no debe omitirse — es lo que diferencia un plan de pruebas senior de una lista de tareas.
