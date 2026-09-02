# Guía de Refactorización

## Principio central

Refactorizar significa cambiar la estructura interna del código sin cambiar su comportamiento observable. Si el usuario quiere cambiar comportamiento, eso no es refactor — es una modificación funcional, y debe tratarse (y probarse) como tal.

## Antes de refactorizar

- Si existe una suite de tests que cubre el comportamiento actual, úsala como red de seguridad — corre los tests antes y después.
- Si NO existen tests, dilo explícitamente: refactorizar sin tests es más riesgoso, porque no hay forma automática de verificar que el comportamiento no cambió. Sugiere agregar al menos tests de caracterización (que capturen el comportamiento actual) antes de refactors grandes.

## Señales comunes de que algo necesita refactor (code smells)

- **Función/método muy largo** que hace muchas cosas → extraer funciones más pequeñas con nombres que expliquen qué hace cada parte.
- **Duplicación de lógica** en varios lugares → extraer a una función/módulo compartido.
- **Anidamiento profundo de condicionales** → usar early returns, extraer condiciones a funciones con nombre descriptivo.
- **Clase que hace demasiado** (viola responsabilidad única) → dividir en clases/módulos más enfocados.
- **Parámetros excesivos en una función** → agrupar en un objeto/struct con nombre, o dividir la función.
- **Comentarios que explican "qué hace" el código** en vez de "por qué" → generalmente señal de que el código debería ser más autoexplicativo (mejor nombrado, funciones más pequeñas), no que necesita más comentarios.
- **Números o strings mágicos repetidos** → extraer a constantes nombradas.

## Cómo proponer un refactor

1. Explica qué problema tiene la estructura actual (mantenibilidad, testeabilidad, legibilidad) — no refactorices "porque sí" o por preferencia estética sin justificación.
2. Propón el cambio en pasos incrementales si es grande, cada uno verificable por separado.
3. Muestra el resultado final, pero si el cambio es grande, considera mostrar también los pasos intermedios para que el usuario pueda seguir el razonamiento.
4. No introduzcas abstracciones especulativas ("por si en el futuro necesitamos X") sin una necesidad concreta actual — el over-engineering es tan problemático como el código desordenado.

## Cuándo NO recomendar refactor

- Código que funciona, es código "feo" pero no se toca casi nunca, y no está causando problemas reales de mantenimiento — el costo/beneficio puede no justificarlo. Dilo explícitamente en vez de refactorizar por inercia.
- Justo antes de un deadline crítico, salvo que el refactor sea necesario para resolver un bug bloqueante.
