param(
    [string]$binary,
    [Parameter(ValueFromRemainingArguments=$true)]
    [string[]]$args
)

# Prepend repository ffmpeg bin to PATH so the test binary finds FFmpeg DLLs
$ffmpegBin = Join-Path $PSScriptRoot '..\ffmpeg\windows-x86_64\bin'
try {
    $ffmpegBin = (Resolve-Path $ffmpegBin -ErrorAction Stop).Path
    $env:PATH = $ffmpegBin + ';' + $env:PATH
} catch {
    Write-Host "Warning: ffmpeg bin directory not found at $ffmpegBin"
}

# Invoke the test/run binary with forwarded arguments
& $binary @args
exit $LASTEXITCODE
