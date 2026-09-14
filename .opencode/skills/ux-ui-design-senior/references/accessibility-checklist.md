# Checklist de Accesibilidad (basado en principios WCAG)

Organizado según los 4 principios POUR de WCAG: Perceptible, Operable, Comprensible, Robusto. No es un mapeo exhaustivo del estándar completo — para cumplimiento formal/legal, remite al usuario al documento oficial de WCAG y considerar una auditoría especializada.

## Perceptible

- [ ] Contraste de color suficiente entre texto y fondo (WCAG AA: mínimo 4.5:1 para texto normal, 3:1 para texto grande).
- [ ] La información no se comunica solo por color (ej. un error no debería indicarse solo con el campo en rojo — agregar ícono o texto).
- [ ] Imágenes con texto alternativo (`alt`) que describe su función/contenido, no solo su nombre de archivo. Imágenes puramente decorativas marcadas para que lectores de pantalla las ignoren.
- [ ] Contenido de audio/video con subtítulos o transcripción cuando aporta información esencial.

## Operable

- [ ] Toda funcionalidad accesible por teclado, no solo por mouse/touch (navegación con Tab, activación con Enter/Espacio).
- [ ] Foco visible claramente cuando se navega por teclado (outline/indicador visual, no eliminado por estilo sin reemplazo).
- [ ] Áreas táctiles/clickeables de tamaño suficiente (mínimo recomendado ~44x44px) para personas con precisión motora reducida.
- [ ] Sin contenido que parpadee de forma que pueda inducir convulsiones (parpadeos rápidos y de alto contraste).
- [ ] Suficiente tiempo para completar acciones con límite de tiempo, o forma de extenderlo (formularios que expiran, sesiones con timeout).

## Comprensible

- [ ] Lenguaje claro y directo, evitando jerga innecesaria.
- [ ] Comportamiento predecible: los elementos no cambian de contexto de forma inesperada al recibir foco o al interactuar (ej. un formulario no debería enviarse automáticamente al cambiar un campo sin acción explícita del usuario).
- [ ] Etiquetas claras en campos de formulario, asociadas correctamente al campo (no solo placeholder text que desaparece al escribir).
- [ ] Mensajes de error específicos que indican qué corregir, ubicados cerca del campo relevante.

## Robusto

- [ ] Estructura semántica correcta (encabezados jerárquicos `h1`-`h6` en orden, listas marcadas como listas, botones marcados como botones y no divs con onClick).
- [ ] Compatible con tecnología de asistencia (lectores de pantalla) — atributos ARIA usados cuando el HTML semántico nativo no es suficiente, sin abusar de ARIA cuando el elemento nativo ya resuelve el caso.

## Al reportar hallazgos de accesibilidad

- Prioriza por impacto: barreras que bloquean completamente una tarea para algún grupo de usuarios (ej. formulario no navegable por teclado) son críticas, independientemente de cuántos usuarios se estime que las encuentran.
- Conecta cada hallazgo con el impacto humano concreto (ej. "una persona que navega solo con teclado no puede completar el checkout") en vez de solo citar el criterio técnico incumplido — ayuda a priorizar frente a otros stakeholders.
- Recuerda que la accesibilidad bien implementada generalmente mejora la experiencia para todos los usuarios, no solo para quienes tienen una discapacidad diagnosticada.
