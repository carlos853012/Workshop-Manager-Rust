# Guía de Arquitectura de Información

## Principios centrales

- Organiza según el modelo mental del usuario (cómo espera encontrar algo), no según cómo está estructurado internamente el sistema o la organización.
- Usa el lenguaje del usuario en etiquetas de navegación/categorías, no jerga interna del negocio o del equipo técnico.
- Prioriza lo más usado/importante con mayor visibilidad y menor profundidad de clics, sin ocultar lo menos frecuente pero necesario.

## Técnicas útiles

### Card sorting
Pide a usuarios reales que agrupen contenido/funcionalidades en categorías que tengan sentido para ellos. Útil para validar o descubrir una estructura de navegación antes de definirla desde la intuición del equipo. Abierto (el usuario crea sus propias categorías) revela modelos mentales; cerrado (categorías predefinidas) valida una estructura ya propuesta.

### Tree testing
Con la estructura de navegación ya definida (sin el diseño visual final), pide a usuarios que encuentren dónde estaría cierta información/funcionalidad. Mide qué tan encontrable es algo con la estructura actual, independiente del diseño visual.

### Mapas de sitio / jerarquía
Documenta la estructura completa de navegación con niveles y relaciones — útil para detectar profundidad excesiva o categorías con demasiadas opciones al mismo nivel.

## Cómo evaluar la profundidad de la jerarquía

- Muy plana (todo al mismo nivel con demasiadas opciones): sobrecarga cognitiva al elegir, difícil de escanear.
- Muy profunda (muchos niveles para llegar a algo frecuente): fricción y abandono, especialmente para tareas repetidas.
- No hay un número "correcto" universal — depende del volumen de contenido/funcionalidades y de qué tan frecuentemente se accede a cada rama. Prioriza que las tareas más frecuentes tengan el camino más corto.

## Errores comunes a señalar

- Categorías que reflejan la estructura organizacional interna (ej. "Departamento de Finanzas") en vez de lo que el usuario busca (ej. "Facturación y pagos").
- Navegación duplicada o inconsistente entre secciones similares del producto.
- Etiquetas ambiguas que podrían significar varias cosas distintas para distintos usuarios (ej. "Recursos" sin más contexto).
- Agregar una nueva categoría de navegación cada vez que se agrega una feature, sin revisar si encaja en la estructura existente — con el tiempo esto degrada la arquitectura completa.
