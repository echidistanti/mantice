<#
.SYNOPSIS
  Installs Inno Setup, the tool that builds the Mantice installer.
.DESCRIPTION
  Downloads a pinned version of Inno Setup from its GitHub releases, checks the SHA-256 of the file and installs
  it silently. The GitHub workflows run it before installer\build.ps1; it is also handy on a fresh machine.
  Installing into Program Files needs administrator rights (the GitHub runners have them).
#>
$ErrorActionPreference = 'Stop'

$version = '6.7.4'
$sha256 = 'a549929319f9120716677c685535ca4671e4b937b5fdc87e2585f32d8eb30e8c'
$url = "https://github.com/jrsoftware/issrc/releases/download/is-6_7_4/innosetup-$version.exe"
$setup = Join-Path ([IO.Path]::GetTempPath()) "innosetup-$version.exe"

Write-Host "Downloading Inno Setup $version"
Invoke-WebRequest -Uri $url -OutFile $setup

$actual = (Get-FileHash -Path $setup -Algorithm SHA256).Hash
if ($actual -ne $sha256) {
    Remove-Item $setup -Force
    throw "The SHA-256 of the Inno Setup installer is $actual, expected $sha256"
}

Write-Host "Installing Inno Setup $version"
$process = Start-Process -FilePath $setup -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/SP-' -Wait -PassThru
if ($process.ExitCode -ne 0) {
    throw "The Inno Setup installer failed with exit code $($process.ExitCode)"
}
Remove-Item $setup -Force
