# asscify installer — Windows (PowerShell)
# Usage:
#   irm https://raw.githubusercontent.com/flaxodotdev/asscify/master/install.ps1 | iex
#   $env:ASSCIFY_VERSION='v0.1.0'; irm https://raw.githubusercontent.com/flaxodotdev/asscify/master/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = "flaxodotdev/asscify"
$Bin = "asscify.exe"
$InstallDir = if ($env:ASSCIFY_INSTALL_DIR) { $env:ASSCIFY_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA "asscify\bin" }
$Version = if ($env:ASSCIFY_VERSION) { $env:ASSCIFY_VERSION } else { "latest" }

Write-Host "==> asscify installer (Windows)"
Write-Host "    version: $Version"
Write-Host "    dir    : $InstallDir"

$Arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
switch ($Arch) {
    "X64" { $Target = "windows-x86_64" }
    "Arm64" { $Target = "windows-aarch64" }
    default { throw "unsupported arch: $Arch" }
}

$Asset = "asscify-$Target.zip"

if ($Version -eq "latest") {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    $Version = $release.tag_name
    if (-not $Version) { throw "could not resolve latest release" }
}

$Base = "https://github.com/$Repo/releases/download/$Version"
$Url = "$Base/$Asset"

$Tmp = Join-Path $env:TEMP ("asscify-install-" + [guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $Tmp | Out-Null

try {
    Write-Host "==> downloading $Asset"
    $ZipPath = Join-Path $Tmp $Asset
    Invoke-WebRequest -Uri $Url -OutFile $ZipPath

    Expand-Archive -Path $ZipPath -DestinationPath $Tmp -Force
    $Exe = Get-ChildItem -Path $Tmp -Recurse -Filter $Bin | Select-Object -First 1
    if (-not $Exe) { throw "binary not found in archive" }

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Copy-Item -Force $Exe.FullName (Join-Path $InstallDir $Bin)

    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($UserPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
        $env:Path = "$env:Path;$InstallDir"
        Write-Host "==> added $InstallDir to your user PATH (new shells pick this up)"
    }

    Write-Host ""
    Write-Host "==> installed $(Join-Path $InstallDir $Bin)"
    Write-Host "Run:  asscify"
    Write-Host "Help: asscify  then press ?"
}
finally {
    Remove-Item -Recurse -Force $Tmp -ErrorAction SilentlyContinue
}
