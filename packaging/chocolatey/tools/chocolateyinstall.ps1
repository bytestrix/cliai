$ErrorActionPreference = 'Stop'

$packageName = 'cliai'
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$url64 = 'https://github.com/cliai-team/cliai/releases/download/v0.1.0/cliai-windows-x86_64.zip'
$checksum64 = 'REPLACE_WITH_ACTUAL_CHECKSUM'

$packageArgs = @{
  packageName   = $packageName
  unzipLocation = $toolsDir
  url64bit      = $url64
  checksum64    = $checksum64
  checksumType64= 'sha256'
}

Install-ChocolateyZipPackage @packageArgs

# Add to PATH if not already there
$binPath = Join-Path $toolsDir 'cliai.exe'
if (Test-Path $binPath) {
    Write-Host "CLIAI installed successfully!"
    Write-Host "Note: You may need to restart your terminal for PATH changes to take effect."
    Write-Host ""
    Write-Host "To get started:"
    Write-Host "1. Install Ollama: https://ollama.ai/download"
    Write-Host "2. Run: ollama pull mistral"
    Write-Host "3. Try: cliai `"hello world`""
} else {
    throw "Installation failed: cliai.exe not found"
}