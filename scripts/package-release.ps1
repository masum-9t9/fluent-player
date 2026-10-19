param([switch]$SkipBuild)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$manifest = Get-Content (Join-Path $root 'Cargo.toml') -Raw
if ($manifest -notmatch '(?m)^version\s*=\s*"([^"]+)"') { throw 'Could not read package version from Cargo.toml' }
$version = $Matches[1]
$target = Join-Path $root 'target\release\fluent_player.exe'
$mpv = Join-Path $root 'third_party\mpv\libmpv-2.dll'
$notices = Join-Path $root 'third_party\mpv\README.md'
$release = Join-Path $root 'release'
$portable = Join-Path $release 'portable'
if (-not $SkipBuild) {
    Push-Location $root
    try { cargo build --release; if ($LASTEXITCODE -ne 0) { throw "cargo build failed ($LASTEXITCODE)" } }
    finally { Pop-Location }
}
foreach ($path in @($target, $mpv, $notices)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Required release input missing: $path" }
}
New-Item -ItemType Directory -Force -Path $release, $portable | Out-Null
$appName = "FluentPlayer-v$version.exe"
Copy-Item -LiteralPath $target -Destination (Join-Path $release $appName) -Force
Copy-Item -LiteralPath $mpv -Destination (Join-Path $release 'libmpv-2.dll') -Force
Copy-Item -LiteralPath $notices -Destination (Join-Path $release 'THIRD-PARTY-NOTICES.md') -Force
Copy-Item -LiteralPath $target -Destination (Join-Path $portable 'FluentPlayer.exe') -Force
Copy-Item -LiteralPath $mpv -Destination (Join-Path $portable 'libmpv-2.dll') -Force
Copy-Item -LiteralPath $notices -Destination (Join-Path $portable 'THIRD-PARTY-NOTICES.md') -Force
@"
Fluent Player portable build

Run FluentPlayer.exe. Keep libmpv-2.dll beside it for media playback.
This x64 build targets Windows 10/11. No installer is required.
See THIRD-PARTY-NOTICES.md for mpv runtime provenance and licensing notes.
"@ | Set-Content -Encoding utf8 (Join-Path $portable 'README.txt')
$archive = Join-Path $release "FluentPlayer-v$version-portable.zip"
if (Test-Path -LiteralPath $archive) { Remove-Item -LiteralPath $archive -Force }
Compress-Archive -Path (Join-Path $portable '*') -DestinationPath $archive -CompressionLevel NoCompression
Write-Host "Created $appName, portable folder, and $(Split-Path -Leaf $archive) under $release"
