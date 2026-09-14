# Checklist de Heurísticas de Usabilidad (Nielsen)

Usa estas 10 heurísticas como marco sistemático al evaluar una interfaz. Para cada una, busca ejemplos concretos en la interfaz evaluada, no una evaluación abstracta.

## 1. Visibilidad del estado del sistema
¿El usuario siempre sabe qué está pasando? (indicadores de carga, confirmaciones de acciones, estado actual claro — ej. en qué paso de un formulario multi-paso está).

## 2. Coincidencia entre el sistema y el mundo real
¿El lenguaje, orden de información e íconos usan conceptos familiares al usuario, no jerga técnica interna del sistema?

## 3. Control y libertad del usuario
¿Hay una "salida de emergencia" clara para acciones no deseadas (deshacer, cancelar, volver atrás) sin tener que pasar por un proceso extendido?

## 4. Consistencia y estándares
¿Los mismos elementos/acciones se comportan igual en todo el producto? ¿Se siguen convenciones estándar de la plataforma (web/iOS/Android) que el usuario ya conoce de otros productos?

## 5. Prevención de errores
¿El diseño previene errores antes de que ocurran (validación en tiempo real, confirmaciones para acciones destructivas, restricciones de formato claras) en vez de solo mostrar un error después?

## 6. Reconocer antes que recordar
¿La información y opciones necesarias están visibles cuando se necesitan, en vez de requerir que el usuario recuerde algo de una pantalla anterior?

## 7. Flexibilidad y eficiencia de uso
¿Hay atajos o formas más eficientes para usuarios experimentados, sin complicar la experiencia para usuarios nuevos (ej. atajos de teclado, autocompletar, valores por defecto inteligentes)?

## 8. Diseño estético y minimalista
¿Cada elemento en pantalla tiene un propósito? ¿Hay información o elementos visuales que compiten innecesariamente por la atención con lo realmente importante?

## 9. Ayudar a reconocer, diagnosticar y recuperarse de errores
¿Los mensajes de error se expresan en lenguaje claro (no códigos técnicos), indican exactamente qué pasó, y sugieren una solución concreta?

## 10. Ayuda y documentación
Si la tarea es compleja, ¿hay ayuda contextual disponible en el momento que se necesita, en vez de requerir buscar en documentación separada?

## Cómo reportar hallazgos

- Agrupa por heurística violada, con un ejemplo concreto de la interfaz para cada hallazgo (no una afirmación abstracta).
- Prioriza por severidad: **Crítico** (bloquea o hace fallar la tarea), **Medio** (genera fricción/confusión pero se puede completar la tarea), **Menor** (cosmético, molesto pero no bloqueante).
- Para cada hallazgo relevante, sugiere una dirección de solución concreta, no solo el problema.
