# Synoema — Build for Windows x64
#
# Usage:
#   .\build.ps1           Build synoema.exe + synoema-mcp.exe
#   .\build.ps1 -Clean    Remove built binaries
#   .\build.ps1 -Install  Copy to C:\Tools\ and add to PATH suggestion

param(
    [switch]$Clean,
    [switch]$Install
)

$Target   = "x86_64-pc-windows-msvc"
$LangDir  = Join-Path $PSScriptRoot "..\..\..\lang"
$McpDir   = Join-Path $PSScriptRoot "..\..\..\mcp"

if ($Clean) {
    Remove-Item -Force -ErrorAction SilentlyContinue `
        (Join-Path $PSScriptRoot "synoema.exe"),
        (Join-Path $PSScriptRoot "synoema-mcp.exe")
    Write-Host "Cleaned."
    exit 0
}

if ($Install) {
    $InstallDir = "C:\Tools"
    if (-not (Test-Path $InstallDir)) { New-Item -ItemType Directory -Path $InstallDir | Out-Null }
    Copy-Item (Join-Path $PSScriptRoot "synoema.exe") $InstallDir -Force
    Copy-Item (Join-Path $PSScriptRoot "synoema-mcp.exe") $InstallDir -Force
    Write-Host "Installed to $InstallDir"
    Write-Host "Add $InstallDir to your PATH if not already there:"
    Write-Host '  [Environment]::SetEnvironmentVariable("PATH", $env:PATH + ";C:\Tools", "User")'
    exit 0
}

# Build synoema (repl)
Write-Host "Building synoema..."
cargo build --release --target $Target -p synoema-repl --manifest-path (Join-Path $LangDir "Cargo.toml")
if ($LASTEXITCODE -ne 0) { Write-Error "Build failed"; exit 1 }
Copy-Item (Join-Path $LangDir "target\$Target\release\synoema.exe") $PSScriptRoot -Force
Write-Host "Built: synoema.exe"

# Build synoema-mcp
Write-Host "Building synoema-mcp..."
cargo build --release --target $Target --manifest-path (Join-Path $McpDir "Cargo.toml")
if ($LASTEXITCODE -ne 0) { Write-Error "Build failed"; exit 1 }
Copy-Item (Join-Path $McpDir "target\$Target\release\synoema-mcp.exe") $PSScriptRoot -Force
Write-Host "Built: synoema-mcp.exe"

Write-Host "`nDone. Binaries are in: $PSScriptRoot"
