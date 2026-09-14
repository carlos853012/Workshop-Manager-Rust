# Manual de Usuario — WorkshopManager

**Versión:** 0.1.0
**Última actualización:** 2026-09-14
**Para:** Clientes y usuarios finales

---

## 1. Requisitos del Sistema

### Servidor (PC principal)

| Componente | Mínimo | Recomendado |
|------------|--------|-------------|
| Sistema operativo | Windows 10 (64-bit) | Windows 11 |
| RAM | 2 GB | 4 GB |
| Disco | 1 GB libre | 5 GB libre |
| Red | Solo LAN (opcional) | LAN + internet |

### Viewer (PCs secundarias)

| Componente | Mínimo | Recomendado |
|------------|--------|-------------|
| Sistema operativo | Windows 10 (64-bit) | Windows 11 |
| RAM | 1 GB | 2 GB |
| Disco | 100 MB libre | 500 MB libre |
| Red | Conexión al server via LAN | LAN |

---

## 2. Instalación

### 2.1 Instalar el Servidor

1. Ejecutar `WorkshopManagerServer-0.1.0-x86.msi`
2. Seguir el asistente de instalación
3. El servidor se instala en `C:\Program Files\WorkshopManager\`
4. Se crea un acceso directo en el Menú Inicio

### 2.2 Instalar el Viewer

1. Ejecutar `WorkshopManagerViewer-0.1.0-x86.msi`
2. Seguir el asistente de instalación
3. Se crea un acceso directo en el Menú Inicio

---

## 3. Primer Uso del Servidor

### 3.1 Activación de Licencia

Al iniciar el servidor por primera vez:

1. Se muestra un **código de activación** en la consola
2. Copia este código (las primeras 16 caracteres)
3. Envíamelo por WhatsApp o email para activar tu licencia
4. Recibirás un archivo `license.dat`
5. Copia `license.dat` a la carpeta de datos:
   ```
   %LOCALAPPDATA%\WorkshopManager\data\
   ```
6. Reinicia el servidor

### 3.2 Modo Trial

Si no tienes licencia aún, el servidor funciona en **modo trial por 7 días** con las siguientes limitaciones:
- 1 viewer conectado
- Sin migración a otro equipo

### 3.3 Registro de Admin

En el primer inicio, el servidor te pedirá crear el usuario administrador:
1. Ingresa tu email y contraseña
2. Esta cuenta tiene acceso total al sistema
3. Puedes crear más usuarios después (Mecánico, Vendedor)

---

## 4. El Servidor (Tray Icon)

El servidor se ejecuta en segundo plano. Su icono aparece en la bandeja del sistema (esquina inferior derecha).

### 4.1 Menú del Icono

Haz clic derecho en el icono del taller para ver las opciones:

| Opción | Descripción |
|--------|-------------|
| **Licencia: Base (2 viewers)** | Muestra el estado actual de tu licencia |
| **Copiar API Key** | Copia la clave de API al portapapeles |
| **Copiar Device Key** | Genera una nueva clave de dispositivo y la copia |
| **Ejecutar al inicio** | Activa/desactiva inicio automático con Windows |
| **Salir del servidor** | Detiene el servidor completamente |

### 4.2 Copiar API Key

Necesitas esta clave para conectar el viewer al servidor:
1. Clic derecho en el icono → **Copiar API Key**
2. Abre el viewer en otra PC
3. Clic en el ícono de engranaje ⚙️
4. Pega la API Key en el campo correspondiente

### 4.3 Copiar Device Key

Si el administrador activó la verificación de dispositivo:
1. Clic derecho → **Copiar Device Key**
2. La clave se genera y copia automáticamente
3. Pégala en el viewer (campo Device Key)

---

## 5. El Viewer (Conexión)

### 5.1 Conexión Local (misma PC)

Si el viewer y servidor están en la misma PC:
1. Abre el viewer
2. En la pantalla de login, haz clic en ⚙️
3. Configuración por defecto:
   - Dirección: `https://127.0.0.1:8443`
   - API Key: `dev-key-change-in-production`
   - Device Key: *(vacío)*
4. Haz clic en **Guardar**
5. Ingresa con tu email y contraseña

### 5.2 Conexión LAN (otra PC)

Para conectar desde otra PC en la red:
1. Abre el viewer en la PC secundaria
2. Clic en ⚙️ en la pantalla de login
3. Ingresa:
   - **Dirección del servidor:** `https://192.168.1.X:8443` (la IP del PC con el server)
   - **API Key:** La misma que tiene el servidor (pedírsela al administrador)
   - **Device Key:** *(vacío)* si no está activada la verificación de dispositivo
4. Clic en **Guardar**
5. Ingresa con tu usuario y contraseña

### 5.3 Configuración de Conexión

El ícono de engranaje ⚙️ te permite cambiar:

| Campo | Descripción |
|-------|-------------|
| Dirección del servidor | URL del server (https://IP:8443) |
| API Key | Clave compartida entre server y viewer |
| Device Key | Clave de dispositivo (opcional) |

---

## 6. Límites de Viewers

Tu licencia define cuántos viewers puedes conectar simultáneamente:

| Licencia | Viewers máximos | Migraciones |
|----------|----------------|-------------|
| Trial | 1 | 0 |
| Base | 2 | 3 |
| Reports | 5 | 5 |
| Advanced | 10 | 10 |
| API | Ilimitado | Ilimitado |

### ¿Qué pasa si alcanzo el límite?

Si intentas conectar un viewer adicional:
1. Se muestra un error: "Límite de viewers alcanzado"
2. Debes desconectar un viewer existente o actualizar tu licencia

### ¿Cómo actualizo mi licencia?

Contacta al proveedor para obtener una licencia de mayor tier.

---

## 7. Migración a Nuevo Equipo

Si tu PC principal se daña y necesitas mover el servidor a otro equipo:

1. Contacta al proveedor
2. Instala el servidor en el nuevo equipo
3. Ejecuta `license-tool hardware-id` para obtener el código
4. Envía el código al proveedor
5. Recibirás una nueva licencia para el nuevo equipo
6. Copia `license.dat` a `%LOCALAPPDATA%\WorkshopManager\data\`
7. Reinicia el servidor

**Nota:** La licencia anterior se desactiva automáticamente.

---

## 8. Preguntas Frecuentes

### ¿Necesito internet para usar el sistema?

No. Solo necesitas internet para la activación inicial. Después, el sistema funciona completamente offline.

### ¿Puedo usar el server en otra red?

Sí. Cambia `host = "0.0.0.0"` en `config/server.toml` para que escuche en todas las interfaces de red.

### ¿Cómo cambio la contraseña de admin?

Ingresa al viewer como admin → Usuarios → Seleccionar usuario → Editar contraseña.

### ¿Dónde se guardan los datos?

En Windows: `%LOCALAPPDATA%\WorkshopManager\data\`

### ¿Cómo hago un backup?

Los backups se crean automáticamente cada 24 horas. Para un backup manual, contacta al administrador.

### ¿El sistema es seguro?

Sí. El sistema utiliza:
- Cifrado TLS 1.3 para todas las comunicaciones
- Contraseñas hasheadas con Argon2id
- Cifrado AES-256-GCM para datos sensibles
- Registro de auditoría completo

---

## 9. Soporte

Si tienes problemas:
1. Revisa este manual
2. Contacta al proveedor por WhatsApp o email
3. Incluye el mensaje de error completo

---

## 10. Glosario

| Término | Definición |
|---------|-----------|
| **Server** | PC donde se ejecuta el servidor (base de datos) |
| **Viewer** | PC donde se ejecuta la aplicación (interfaz) |
| **API Key** | Clave compartida para autenticar la conexión |
| **Device Key** | Clave de dispositivo para vincular el viewer |
| **Licencia** | Archivo que autoriza el uso del software |
| **Trial** | Modo de prueba gratuito por 7 días |
| **Tier** | Nivel de licencia (Base, Reports, Advanced, API) |
