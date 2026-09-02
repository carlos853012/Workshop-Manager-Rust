param(
    [Parameter(Mandatory = $true)]
    [string]$NewVersion
)

if ($NewVersion -notmatch '^\d+\.\d+\.\d+$') {
    Write-Error "Formato invalido: use X.Y.Z (ej: 0.4.0)"
    exit 1
}

$root = Split-Path -Parent $PSScriptRoot
$cargo = Join-Path $root "Cargo.toml"
$enc = New-Object System.Text.UTF8Encoding($false)

$content = [System.IO.File]::ReadAllText($cargo)

if ($content -notmatch '(?m)^\[workspace\.package\]\s*^version = "\d+\.\d+\.\d+"') {
    Write-Error "No se encontro [workspace.package] version en $cargo"
    exit 1
}

$old = [regex]::Match($content, '(?m)^version = "\d+\.\d+\.\d+"').Value
$new = "version = `"$NewVersion`""
$content = $content.Replace($old, $new)
[System.IO.File]::WriteAllText($cargo, $content, $enc)
Write-Host "Version actualizada: $old -> $new"

Write-Host "Verificando build..."
& cargo build 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) {
    git checkout -- $cargo
    Write-Error "Build fallo; se revirtio Cargo.toml"
    exit 1
}

git add $cargo
git commit -m "v${NewVersion}: bump version"
if ($LASTEXITCODE -ne 0) {
    Write-Error "Commit fallo (¿cambios no staged en otros archivos?)"
    exit 1
}

git tag "v$NewVersion"
Write-Host ""
Write-Host "Listo. Para publicar el release:"
Write-Host "  git push && git push --tags"