# Plantilla de Caso de Prueba

Usa esta estructura como tabla cuando generes casos de prueba (en Word o Markdown).

| Campo | Descripción |
|---|---|
| ID | Identificador único, ej. TC-001 |
| Título | Descripción corta y específica de qué se prueba |
| Módulo/Feature | A qué parte del sistema pertenece |
| Tipo | Funcional / No funcional / Regresión / Exploratoria |
| Prioridad | Alta / Media / Baja (según riesgo de negocio) |
| Precondiciones | Estado necesario antes de ejecutar (ej. usuario logueado, datos de prueba cargados) |
| Pasos | Numerados, uno por acción, verificables |
| Datos de prueba | Valores específicos a usar (incluye casos límite si aplica) |
| Resultado esperado | Qué debería pasar exactamente, sin ambigüedad |
| Tipo de caso | Happy path / Negativo / Límite / Error de sistema |

## Ejemplo

| Campo | Valor |
|---|---|
| ID | TC-014 |
| Título | Rechazar registro con email ya existente |
| Módulo | Registro de usuario |
| Tipo | Funcional |
| Prioridad | Alta |
| Precondiciones | Existe un usuario registrado con email `test@example.com` |
| Pasos | 1. Ir a pantalla de registro. 2. Completar formulario con email `test@example.com`. 3. Enviar formulario. |
| Datos de prueba | email: test@example.com, password: válido |
| Resultado esperado | El sistema muestra el mensaje "Este correo ya está registrado" y no crea una cuenta duplicada |
| Tipo de caso | Negativo |

## Buenas prácticas al generar sets de casos

- Agrupa los casos por módulo o funcionalidad, no como una lista plana gigante.
- Incluye siempre una mezcla de happy path, negativos, límites y errores de sistema — nunca solo happy path salvo pedido explícito.
- Si el número de combinaciones posibles es muy alto (ej. múltiples filtros combinables), usa una tabla de decisión o indica explícitamente qué combinaciones se priorizaron y por qué (riesgo, uso real, etc.), en vez de listar todas las combinaciones posibles.
- Numera los IDs de forma consecutiva y sin huecos dentro de un mismo documento.
