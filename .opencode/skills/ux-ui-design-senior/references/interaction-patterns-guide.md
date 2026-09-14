# Guía de Patrones de Interacción y Microcopy

## Formularios

- Un campo por línea para formularios largos (más fácil de escanear que múltiples columnas), salvo campos naturalmente relacionados (ej. ciudad/código postal).
- Validación en tiempo real cuando ayuda a prevenir el error antes de enviar (ej. formato de email), pero sin marcar error mientras el usuario todavía está escribiendo activamente un campo válido a medio completar.
- Etiquetas siempre visibles (no solo placeholder que desaparece al escribir — el usuario pierde el contexto de qué campo es al momento de revisarlo).
- Indica claramente qué campos son obligatorios vs. opcionales (generalmente es más claro marcar los opcionales, si la mayoría son obligatorios).
- Mensajes de error específicos junto al campo relevante, no solo un mensaje genérico al principio o final del formulario.

## Mensajes de error

- Explican qué pasó en lenguaje claro, no un código técnico sin contexto ("no pudimos procesar tu pago" en vez de "Error 402").
- Indican cómo resolverlo cuando es posible ("verifica el número de tarjeta e intenta de nuevo").
- No culpan al usuario ni usan tono negativo/alarmante innecesariamente.
- Nunca pierden el trabajo del usuario sin advertencia (ej. un formulario largo que falla no debería borrar todo lo ya completado).

## Estados vacíos

- Explican por qué está vacío (¿es la primera vez, o se filtró/buscó algo sin resultados?) — cada causa merece un mensaje distinto.
- Cuando aplica, sugieren la próxima acción concreta (ej. "Todavía no tienes proyectos — crea el primero" con un botón de acción directo), no solo "No hay datos".
- Diferencia estado vacío por primera vez (oportunidad de guiar/onboard) de estado vacío por filtro sin resultados (sugerir ajustar el filtro).

## Confirmaciones y acciones destructivas

- Para acciones irreversibles o de alto impacto (eliminar, cancelar una suscripción), confirma explícitamente y describe la consecuencia concreta ("esto eliminará permanentemente 12 archivos"), no solo "¿estás seguro?".
- El botón de confirmación describe la acción específica ("Eliminar", no "Aceptar" genérico), consistente con el título/contexto del diálogo.

## Onboarding

- Prioriza mostrar valor rápido sobre explicar exhaustivamente cada funcionalidad de entrada — el mejor onboarding a menudo es contextual (aparece cuando el usuario llega a esa parte del producto), no un tour completo al inicio que se olvida.
- No bloquees al usuario con pasos de configuración opcionales antes de dejarlo experimentar el valor central del producto.

## Estados de carga

- Para operaciones cortas (<1 segundo aprox.), un indicador simple es suficiente.
- Para operaciones más largas, comunica progreso si es posible (no solo un spinner indefinido) y qué está pasando, especialmente si involucra múltiples pasos.
- Skeleton screens (placeholders con la forma del contenido que va a cargar) suelen percibirse como más rápidos que un spinner genérico, para cargas de contenido de pantalla completa.

## Principio general de todo microcopy

Nombra las cosas por lo que la persona controla y reconoce, no por cómo funciona el sistema internamente. Usa voz activa: un botón dice exactamente qué pasa al usarlo ("Guardar cambios", no "Enviar"), y el resultado de esa acción usa la misma palabra ("Guardado", no "Enviado con éxito").
