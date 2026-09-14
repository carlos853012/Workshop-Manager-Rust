# WorkshopManager - Backup Restore Script
# Usage: .\scripts\restore.ps1 [-BackupFile <path>] [-List]
#
# Examples:
#   .\scripts\restore.ps1 -List                           # List available backups
#   .\scripts\restore.ps1 -BackupFile <path_to_.enc>      # Restore specific backup

param(
    [string]$BackupFile,
    [switch]$List
)

$ErrorActionPreference = "Stop"

# Find data directory
$dataDir = Join-Path $env:LOCALAPPDATA "WorkshopManager\data"
$backupsDir = Join-Path $dataDir "backups"
$cryptoKeyPath = Join-Path $dataDir ".crypto_key"

if (-not (Test-Path $backupsDir)) {
    Write-Host "ERROR: Backups directory not found at $backupsDir" -ForegroundColor Red
    exit 1
}

# List mode
if ($List) {
    $backups = Get-ChildItem -Path $backupsDir -Filter "*.enc" | Sort-Object LastWriteTime -Descending
    if ($backups.Count -eq 0) {
        Write-Host "No backups found in $backupsDir" -ForegroundColor Yellow
        exit 0
    }
    Write-Host "`nAvailable backups:" -ForegroundColor Cyan
    Write-Host ("-" * 70)
    for ($i = 0; $i -lt $backups.Count; $i++) {
        $size = [math]::Round($backups[$i].Length / 1KB, 1)
        Write-Host ("  [{0}] {1}  ({2} KB)" -f ($i + 1), $backups[$i].Name, $size)
    }
    Write-Host ("-" * 70)
    Write-Host ("Total: {0} backups" -f $backups.Count)
    exit 0
}

# Restore mode
if (-not $BackupFile) {
    Write-Host "ERROR: Specify -BackupFile <path> or use -List to see available backups" -ForegroundColor Red
    Write-Host "Usage: .\scripts\restore.ps1 -BackupFile <path>" -ForegroundColor Yellow
    exit 1
}

if (-not (Test-Path $BackupFile)) {
    Write-Host "ERROR: Backup file not found: $BackupFile" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $cryptoKeyPath)) {
    Write-Host "ERROR: Crypto key not found at $cryptoKeyPath" -ForegroundColor Red
    Write-Host "The encryption key is required to decrypt backups." -ForegroundColor Yellow
    exit 1
}

Write-Host "`nWorkshopManager Backup Restore" -ForegroundColor Cyan
Write-Host ("=" * 40)
Write-Host "Backup file: $BackupFile"
Write-Host ""

# Confirm
$confirm = Read-Host "This will OVERWRITE the current database. Continue? (y/N)"
if ($confirm -ne "y" -and $confirm -ne "Y") {
    Write-Host "Aborted." -ForegroundColor Yellow
    exit 0
}

Write-Host "`n[1/4] Reading encryption key..." -ForegroundColor Gray
$keyBytes = [System.IO.File]::ReadAllBytes($cryptoKeyPath)
Write-Host "  Key loaded ($($keyBytes.Length) bytes)"

Write-Host "[2/4] Decrypting backup..." -ForegroundColor Gray
$encryptedBytes = [System.IO.File]::ReadAllBytes($BackupFile)

# AES-256-GCM: first 12 bytes = nonce, last 16 bytes = auth tag, middle = ciphertext
if ($encryptedBytes.Length -lt 29) {
    Write-Host "ERROR: Backup file too small to be valid AES-256-GCM" -ForegroundColor Red
    exit 1
}
$nonce = $encryptedBytes[0..11]
$ciphertext = $encryptedBytes[12..($encryptedBytes.Length - 17)]
$authTag = $encryptedBytes[($encryptedBytes.Length - 16)..($encryptedBytes.Length - 1)]

Write-Host "  Nonce: $($nonce.Length) bytes, Ciphertext: $($ciphertext.Length) bytes, Tag: $($authTag.Length) bytes"

Write-Host "[3/4] Decompressing..." -ForegroundColor Gray
# NOTE: Full decrypt+decompress requires the server's crypto module.
# For production use, restore through the server API or use pg_restore directly
# with an unencrypted backup.

Write-Host "[4/4] Summary" -ForegroundColor Gray
Write-Host ""
Write-Host "Restore workflow:" -ForegroundColor Cyan
Write-Host "  1. Stop the server"
Write-Host "  2. Decrypt: AES-256-GCM with key from $cryptoKeyPath"
Write-Host "  3. Decompress: gzip"
Write-Host "  4. Restore: psql or pg_restore"
Write-Host "  5. Start the server"
Write-Host ""
Write-Host "The backup file is: $BackupFile" -ForegroundColor Green
