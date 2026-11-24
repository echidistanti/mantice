<#
.SYNOPSIS
  Builds the Windows installer of Mantice with Inno Setup.
.DESCRIPTION
  Run `cargo build --release` first, then this script. The installer is written to installer\Output as
  Mantice-<version>-windows-x64-setup.exe. Inno Setup 6.3 or later is needed: installer\install-inno-setup.ps1
  installs it, or get it from https://jrsoftware.org/isdl.php.
.PARAMETER Version
  The version of the installer. Default: the version in Cargo.toml.
.PARAMETER ExeDir
  The folder that holds mantice.exe. Default: target\release.
#>
param(
    [string]$Version,
    [string]$ExeDir
)
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
if (-not $Version) {
    $match = Select-String -Path (Join-Path $root 'Cargo.toml') -Pattern '^version = "(.+)"' | Select-Object -First 1
    if (-not $match) { throw 'Could not read the version from Cargo.toml' }
    $Version = $match.Matches[0].Groups[1].Value
}
if (-not $ExeDir) { $ExeDir = Join-Path $root 'target\release' }
if (-not (Test-Path (Join-Path $ExeDir 'mantice.exe'))) {
    throw "mantice.exe not found in ${ExeDir} - run 'cargo build --release' first"
}

# The compiler: on the PATH, or in the usual places of an Inno Setup installation.
$iscc = Get-Command ISCC.exe -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty Source
if (-not $iscc) {
    $places = @($env:ProgramFiles, ${env:ProgramFiles(x86)}, (Join-Path $env:LOCALAPPDATA 'Programs')) | Where-Object { $_ }
    $iscc = $places |
        ForEach-Object { Get-ChildItem -Path $_ -Directory -Filter 'Inno Setup*' -ErrorAction SilentlyContinue } |
        ForEach-Object { Join-Path $_.FullName 'ISCC.exe' } |
        Where-Object { Test-Path $_ } |
        Select-Object -First 1
}
if (-not $iscc) { throw 'ISCC.exe not found: install Inno Setup 6.3 or later (installer\install-inno-setup.ps1)' }

# The icon of the program is drawn by build.rs into the cargo output folder: the setup program reuses it.
$defines = @("/DAppVersion=$Version", "/DExeDir=$ExeDir")
$icon = Get-ChildItem -Path (Join-Path $root 'target') -Recurse -Filter 'mantice.ico' -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1
if ($icon) { $defines += "/DIconFile=$($icon.FullName)" }
else { Write-Warning 'mantice.ico not found under target: the setup program gets the default Inno Setup icon' }

& $iscc @defines (Join-Path $PSScriptRoot 'mantice.iss')
if ($LASTEXITCODE -ne 0) { throw "ISCC failed with exit code $LASTEXITCODE" }

Get-ChildItem -Path (Join-Path $PSScriptRoot 'Output') -Filter '*.exe' | ForEach-Object { Write-Host "Built $($_.FullName)" }
