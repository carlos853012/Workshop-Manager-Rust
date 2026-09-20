# Creates a desktop shortcut for WorkshopManager Viewer
# Run this script once to create the shortcut

$WshShell = New-Object -ComObject WScript.Shell
$Desktop = [System.Environment]::GetFolderPath("Desktop")
$Shortcut = $WshShell.CreateShortcut("$Desktop\WorkshopManager Viewer.lnk")

# Point to the viewer executable
$ViewerPath = Join-Path $PSScriptRoot "..\crates\workshop-viewer\target\release\workshop-viewer.exe"
$IconPath = Join-Path $PSScriptRoot "..\crates\workshop-viewer\icon.ico"

# Resolve full paths
$ViewerPath = Resolve-Path $ViewerPath -ErrorAction SilentlyContinue
$IconPath = Resolve-Path $IconPath -ErrorAction SilentlyContinue

if ($ViewerPath) {
    $Shortcut.TargetPath = $ViewerPath.Path
} else {
    Write-Host "WARNING: Viewer executable not found at $ViewerPath"
    Write-Host "Please build the viewer first: cargo build -p workshop-viewer --release"
    $Shortcut.TargetPath = "path\to\workshop-viewer.exe"
}

if ($IconPath) {
    $Shortcut.IconLocation = "$($IconPath.Path),0"
} else {
    Write-Host "WARNING: Icon not found at $IconPath"
}

$Shortcut.WorkingDirectory = Split-Path $Shortcut.TargetPath
$Shortcut.Description = "WorkshopManager - Sistema de Gestión de Taller"
$Shortcut.Save()

Write-Host "Desktop shortcut created: $Desktop\WorkshopManager Viewer.lnk"
Write-Host "Icon: $($IconPath.Path)"
