# Self-Hosted Runners — Guía de configuración

> Guía paso a paso para que Workshop Manager compile con **tus propias máquinas** (en vez de las de GitHub) y guarde los instaladores en tu **servidor local**.

---

## ¿Qué es esto y para qué sirve?

Cuando subes código o un tag, GitHub normalmente usa **sus propias computadoras** para compilar. Eso consume los **minutos gratis** de tu cuenta (las de Windows cuestan el doble).

La solución es instalar un **runner** en tus máquinas: un programita que se queda despierto preguntándole a GitHub cada pocos segundos *"¿hay trabajo para mí?"*. Cuando hay, **tu máquina** compila, y no pagas nada.

## Glosario sencillo (para que nadie se pierda)

| Palabra rara | Qué es, en simple |
|---|---|
| **Runner** | Un **robot obrero** que vive en tu PC/server. GitHub le dice "compila esto" y él lo hace gratis. |
| **Token** | La **matrícula temporal** que le dice a GitHub "este robot es mío, trabaja en este repo". Caduca (~1h) y es de un solo uso. |
| **Samba / SMB** | La **puerta con llave** que abre la carpeta de tu server para que Windows pueda escribir ahí (así la PC Windows manda el `.msi`). |
| **nginx** | El **mesero**: cuando abres el navegador, busca el archivo en la carpeta y te lo sirve. |
| **`~/.theseus` cache** | La **mochila** con provisiones: bajar PostgreSQL una sola vez, para no volver a descargarlo en cada build. |
| **`labels` / `runs-on`** | La **etiqueta en la caja**: `wm-linux-01` dice "yo hago los trabajos de Linux x64". |
| **`force user`** | En Samba, "todo lo que se escriba aquí, que aparezca como si lo hubiera hecho `carlos`". |
| **systemd / servicio** | El **autoencendido**: el runner arranca solo cuando prendes la máquina, sin abrir terminal. |

---

## Estado actual del proyecto

| Paso | Estado |
|---|---|
| 1.3 Samba (compartir carpeta) | ✅ Hecho |
| 1.4 nginx (servir en navegador) | ✅ Hecho |
| 1.5 Runner Linux (`wm-linux-01`) | ✅ Hecho — Idle en GitHub |
| 1.6 Caché de PostgreSQL en `ghrunner` | ⏳ Pendiente |
| Workflow manual `Build Linux (manual)` | ✅ Creado (`.github/workflows/build-linux-manual.yml` + `deploy/index.html`) |
| 2. PC Windows (runner del MSI) | ⏳ Pendiente |
| 3. Secretos en GitHub | ⏳ Pendiente |
| 4. Cambios en los workflows | ⏳ Pendiente |

---

## Parte 1 — Server Linux (hecho ✅)

Los comandos de abajo son **los que ya se ejecutaron y funcionaron**. Vienen con los avisos de los errores que aparecieron en el camino.

### 1.1 Instalar paquetes

```bash
sudo apt update
sudo apt install -y nginx samba git curl rsync unzip build-essential \
  pkg-config libssl-dev jq
```

> Se instalan dos "ayudantes": **Samba** (compartir carpetas con Windows) y **nginx** (servir archivos en el navegador).

### 1.2 Crear la carpeta de descargas

```bash
sudo mkdir -p /srv/workshop-releases
sudo chown -R $USER:users /srv/workshop-releases
sudo chmod -R 775 /srv/workshop-releases
```

> Aquí van a parar todos los instaladores (`.msi`, `.deb`, binarios) y el `index.html`.

### 1.3 Samba (abrir la puerta para Windows)

**Paso 1 — editar el archivo de configuración** con `nano` (⚠️ **NO** pegar el bloque directamente en la terminal):

```bash
sudo nano /etc/samba/smb.conf
```

Bajar al **final del archivo** y pegar:

```ini
[workshop-release]
   path = /srv/workshop-releases
   browseable = yes
   read only = no
   valid users = release
   create mask = 0664
   directory mask = 0775
   force user = carlos
```

Guardar (`Ctrl+O`, `Enter`) y salir (`Ctrl+X`).

> ⚠️ `force user = carlos` lleva el **nombre literal** (tu usuario real); Samba **no** expande `$USER`.
> ⚠️ El bloque va **después** de las secciones `[global]`/`[printers]`, no en medio de otra.

**Paso 2 — crear el usuario de sistema ANTES de darle contraseña** (⚠️ si haces `smbpasswd` sin este paso, da `Failed to add entry`):

```bash
sudo useradd -r -M -s /usr/sbin/nologin release
sudo smbpasswd -a release
sudo smbpasswd -e release
sudo pdbedit -L
```

**Paso 3 — reiniciar y probar:**

```bash
sudo systemctl restart smbd
sudo apt install -y smbclient
smbclient -L localhost -U release
```

Debe aparecer `workshop-release` como `Disk`. Prueba de escritura:

```bash
smbclient //localhost/workshop-release -U release -c "put /etc/hostname prueba.txt"
ls -la /srv/workshop-releases
```

> El archivo `prueba.txt` debe aparecer con dueño `carlos` (gracias a `force user`). Borrarlo con:
> `smbclient //localhost/workshop-release -U release -c "del prueba.txt"`

### 1.4 nginx (el mesero)

**Paso 1 — crear el sitio:**

```bash
sudo nano /etc/nginx/sites-available/workshop-releases
```

Pegar:

```nginx
server {
    listen 80 default_server;
    server_name _;
    root /srv/workshop-releases;
    index index.html;
    autoindex off;
    charset utf-8;

    location / {
        try_files $uri $uri/ =404;
    }
}
```

**Paso 2 — habilitarlo y recargar:**

```bash
sudo rm -f /etc/nginx/sites-enabled/default
sudo ln -sf /etc/nginx/sites-available/workshop-releases /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl enable --now nginx
sudo systemctl reload nginx
```

> ⚠️ **Ojo con dos cosas que ya nos pasaron:**
> 1. `systemctl enable --now` **no recarga** un nginx que ya estaba corriendo. Después de tocar la config, siempre `sudo systemctl reload nginx`.
> 2. Si sigue saliendo "Welcome to nginx!", es que el proceso viejo sigue vivo; el `reload` lo arregla.

**Paso 3 — probar:**

```bash
sudo sh -c 'echo ok > /srv/workshop-releases/index.html'
curl -I http://localhost/
sudo rm /srv/workshop-releases/index.html
```

- Con `index.html` presente → `HTTP/1.1 200 OK` (Content-Length: 3).
- Sin `index.html` → `403 Forbidden` (directorio vacío con `autoindex off`). **Es lo esperado**, no un error.

### 1.5 El runner de GitHub (el robot)

**Paso 1 — crear el usuario `ghrunner`** (no tiene contraseña ni sudo; solo trabaja):

```bash
sudo adduser --disabled-password --gecos "" ghrunner
sudo chmod g+s /srv/workshop-releases
```

> `adduser` lo agrega automáticamente al grupo `users`, así que puede escribir en `/srv/workshop-releases`.

**Paso 2 — generar el token** (navegador):
1. https://github.com/carlos853012/Workshop-Manager-Rust/settings/actions
2. **Runners → New self-hosted runner → Linux → x64**
3. Copiar el bloque `./config.sh --url ... --token XXXXX` (caduca en ~1h).

**Paso 3 — entrar como `ghrunner` y descargar:**

```bash
sudo -iu ghrunner
mkdir actions-runner && cd actions-runner

curl -o actions-runner-linux-x64-2.337.0.tar.gz -L \
  https://github.com/actions/runner/releases/download/v2.337.0/actions-runner-linux-x64-2.337.0.tar.gz

echo "70920811a4f8ad4328818682bca5c6469c1c942fab52448868071d0063816613  actions-runner-linux-x64-2.337.0.tar.gz" | shasum -a 256 -c

tar xzf ./actions-runner-linux-x64-2.337.0.tar.gz
```

> Usar la **versión que muestre GitHub** en el momento (la `2.337.0` es la que apareció; puede haber una más nueva).

**Paso 4 — registrar el runner** (con el token, y agregando `--labels` y `--name`, que GitHub no pone por defecto):

```bash
./config.sh --url https://github.com/carlos853012/Workshop-Manager-Rust \
  --token <TU_TOKEN> \
  --labels self-hosted,linux,x64,wm-linux --name wm-linux-01 --unattended
```

Esperado: `Runner successfully added` + `Settings Saved`.

**Paso 5 — instalar como servicio.** ⚠️ **No** uses `./run.sh` (queda pegado a la terminal y se muere al cerrar sesión). Y **no** corras `sudo` desde `ghrunner` (no tiene permiso). Sal de la sesión de `ghrunner` y hazlo desde `carlos`:

```bash
exit
sudo bash -c 'cd /home/ghrunner/actions-runner && ./svc.sh install ghrunner'
sudo bash -c 'cd /home/ghrunner/actions-runner && ./svc.sh start'
sudo bash -c 'cd /home/ghrunner/actions-runner && ./svc.sh status'
```

> ⚠️ El `svc.sh` exige correr **dentro** de la carpeta del runner (`Must run from runner root`). Por eso el `cd` va dentro del `bash -c`. El `cd` directo de `carlos` da "Permiso denegado" (el home de `ghrunner` no es legible para otros).

### 1.6 Verificación

```bash
sudo systemctl status actions.runner.* --no-pager
```

- Debe decir `active (running)`.
- En GitHub → `Settings → Actions → Runners` → **`wm-linux-01`** con estado **Idle** y labels `self-hosted, linux, x64, wm-linux`.

### 1.7 Errores comunes (los que ya nos pasaron)

| Error | Causa | Solución |
|---|---|---|
| `[workshop-release]: no se encontró la orden` | Pegaste el bloque de `smb.conf` en la terminal | Editar `/etc/samba/smb.conf` con `nano` |
| `Failed to add entry for user release` | El usuario `release` no existe en el sistema | `sudo useradd -r -M -s /usr/sbin/nologin release` **antes** de `smbpasswd` |
| `Welcome to nginx!` tras cambiar la config | nginx no se recargó | `sudo systemctl reload nginx` (no `enable --now`) |
| `cd: Permiso denegado` al entrar a `/home/ghrunner` | Home de `ghrunner` no legible para `carlos` | Usar `sudo bash -c 'cd ... && ./svc.sh ...'` |
| `Failed: Must run from runner root` | `svc.sh` fuera de su carpeta | El `cd` dentro del `bash -c`, antes del `./svc.sh` |

---

## Parte 2 — PC Windows (pendiente ⏳)

Runner que compila los `.msi` (necesita Windows + WiX).

1. Instalar **WiX v3.14** (`choco install wixtoolset` o `wix314.exe /quiet`) — debe quedar `C:\Program Files (x86)\WiX Toolset v3.14\bin\candle.exe`.
2. GitHub → `New self-hosted runner` → **Windows x64** → descargar `actions-runner-win-x64`.
3. `config.cmd --url ... --token ... --labels self-hosted,windows,x64,wm-win --name wm-win-01 --unattended`
4. `svc.cmd install` + `svc.cmd start` **como `carlos`** (⚠️ obligatorio: `release.yml` usa `C:\Users\carlos\.theseus`).
5. Credencial SMB: `cmdkey /add:SERVER /user:release /pass:<PASSWORD>` y probar `dir \\SERVER\workshop-release`.
6. Pre-cachear PG: `%USERPROFILE%\.theseus\postgresql\postgresql-18.3.0-x86_64-pc-windows-msvc.tar.gz`.

---

## Parte 3 — Secretos en GitHub (pendiente ⏳)

`Settings → Secrets and variables → Actions`:

| Secreto | Valor |
|---|---|
| `SMB_HOST` | IP o nombre del server (`SERVER`) |
| `SMB_SHARE` | `workshop-release` |
| `SMB_USER` | `release` |
| `SMB_PASS` | `<PASSWORD>` de Samba |

---

## Parte 4 — Cambios en los workflows (pendiente ⏳)

> **Prueba manual disponible:** `Actions → Build Linux (manual)` compila en `wm-linux-01` y publica en
> `/srv/workshop-releases/<version>/` (index.html + versions.json + symlink `latest`), sin tocar
> `release.yml` ni gastar minutos de GitHub. Útil mientras se migran los jobs definitivos.

- `ci.yml`: `check` → `[self-hosted, windows, x64]`; `check-linux-x64` → `[self-hosted, linux, x64]`; `check-linux-arm64` sigue en GitHub.
- `release.yml`: jobs Linux → self-hosted; MSI → self-hosted Windows + copia al share por `robocopy`; reemplazar `softprops/action-gh-release` por un job `publish` que genera `manifest.json` + `index.html` + symlink `latest` en `/srv/workshop-releases`.
- Nuevo `deploy/index.html` (página de descargas LAN).

---

## Chequeo final

1. Push a `main` → CI verde en ambos runners.
2. Tag de prueba → archivos en `/srv/workshop-releases/<versión>/`.
3. `http://SERVER/` lista y permite descargar desde otra máquina de la LAN.

---

## Cross-References

| Document | Description |
|----------|-------------|
| [Maintenance](./maintenance.md) | Routine procedures |
| [Monitoring](./monitoring.md) | Observability and alerting |
| [Incident Response](./incident-response.md) | Incident playbook |
| [Installation](../05-deployment/installation.md) | Server setup |
