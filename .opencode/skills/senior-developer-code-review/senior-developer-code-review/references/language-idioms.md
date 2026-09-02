# Convenciones Idiomáticas por Lenguaje

Usa esto para evaluar si el código "se siente nativo" del lenguaje o está traduciendo patrones de otro lenguaje de forma forzada.

## Rust

- Preferir `Result<T, E>` y `Option<T>` con manejo explícito (`match`, `?`, combinadores como `.map()`/`.and_then()`) en vez de `unwrap()`/`expect()` fuera de tests o prototipos.
- Aprovechar el sistema de ownership en vez de clonar (`.clone()`) reflexivamente para evitar lidiar con lifetimes — señalar clones innecesarios en rutas calientes.
- Preferir iteradores (`.iter()`, `.map()`, `.filter()`, `.collect()`) sobre loops imperativos con índices manuales, cuando mejora la claridad.
- Usar pattern matching exhaustivo (`match`) en vez de cadenas de `if/else` para enums.
- Evitar `unsafe` salvo justificación clara y documentada de por qué es necesario y por qué es seguro en ese contexto.
- Preferir tipos que hagan estados inválidos irrepresentables (newtype pattern, enums bien diseñados) en vez de validar con `if` en cada uso.

## Python

- Preferir comprensions (list/dict/set comprehensions) sobre loops manuales para transformaciones simples — pero no forzarlas si reducen la legibilidad en casos complejos.
- Usar context managers (`with`) para manejo de recursos (archivos, conexiones) en vez de try/finally manual.
- Preferir `dataclasses` o `pydantic` sobre diccionarios sueltos para estructuras de datos con forma conocida.
- Usar type hints en código no trivial, especialmente en funciones públicas/APIs internas.
- Evitar mutable default arguments (`def f(x=[])`) — es un error clásico con efectos secundarios inesperados.
- Preferir `pathlib` sobre manipulación de strings para rutas de archivos.
- Manejo de excepciones específico (`except ValueError:`) en vez de `except:` genérico que oculta bugs.

## JavaScript / TypeScript

- Preferir `const`/`let` sobre `var` (scope de bloque vs. function scope, evita bugs de hoisting).
- Usar `async/await` sobre cadenas de `.then()` anidadas para legibilidad, con manejo de errores explícito (`try/catch` o `.catch()`).
- En TypeScript, evitar `any` salvo justificación explícita — preferir tipos específicos o `unknown` con type narrowing.
- Preferir optional chaining (`?.`) y nullish coalescing (`??`) sobre validaciones manuales verbosas de null/undefined.
- Evitar mutar props/argumentos directamente en frameworks como React — preferir estructuras inmutables y actualización explícita del estado.
- Preferir funciones puras y evitar efectos secundarios ocultos dentro de funciones que parecen ser solo cálculos.

## Al revisar código que mezcla estilos

Si el código parece escrito por alguien acostumbrado a otro lenguaje (ej. mucho `try/except` genérico estilo Java en Python, o loops con índices manuales en Rust donde un iterador sería más idiomático), señálalo explícitamente como oportunidad de mejora, no solo como preferencia — el código idiomático es generalmente más mantenible para otros devs del mismo ecosistema.
