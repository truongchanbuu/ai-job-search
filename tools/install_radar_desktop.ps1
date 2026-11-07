param(
    [Parameter(Mandatory=$true)][string]$Distro,
    [Parameter(Mandatory=$true)][string]$LinuxRoot
)
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$desktop = [Environment]::GetFolderPath('Desktop')
$appDir = Join-Path $env:LOCALAPPDATA 'FreelanceRadar'
New-Item -ItemType Directory -Force -Path $appDir | Out-Null
Add-Type -AssemblyName System.Drawing
$uncRoot = '\\wsl.localhost\' + $Distro + $LinuxRoot.Replace('/', '\')
$assetDir = Join-Path $uncRoot 'assets\radar'
$image = [System.Drawing.Image]::FromFile((Join-Path $assetDir 'radar.png'))
$bitmap = New-Object System.Drawing.Bitmap($image, 256, 256)
$memory = New-Object System.IO.MemoryStream
$bitmap.Save($memory, [System.Drawing.Imaging.ImageFormat]::Png)
$bytes = $memory.ToArray()
$iconPath = Join-Path $appDir 'radar.ico'
$writer = New-Object System.IO.BinaryWriter([System.IO.File]::Create($iconPath))
$writer.Write([UInt16]0); $writer.Write([UInt16]1); $writer.Write([UInt16]1)
$writer.Write([byte]0); $writer.Write([byte]0); $writer.Write([byte]0); $writer.Write([byte]0)
$writer.Write([UInt16]1); $writer.Write([UInt16]32)
$writer.Write([UInt32]$bytes.Length); $writer.Write([UInt32]22); $writer.Write($bytes)
$writer.Dispose(); $memory.Dispose(); $bitmap.Dispose(); $image.Dispose()
Copy-Item -LiteralPath $iconPath -Destination (Join-Path $assetDir 'radar.ico') -Force
$shortcutPath = Join-Path $desktop 'Freelance Radar.lnk'
if (Test-Path -LiteralPath $shortcutPath) {
    $existing = (New-Object -ComObject WScript.Shell).CreateShortcut($shortcutPath)
    if ($existing.Description -ne 'Open Personal Freelance Radar (local backend + terminal)') {
        throw 'An unrelated shortcut already uses this name; refusing to overwrite it.'
    }
}
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = Join-Path $env:LOCALAPPDATA 'Microsoft\WindowsApps\wt.exe'
$shortcut.Arguments = 'new-tab --title "Freelance Radar" wsl.exe -d "' + $Distro + '" --cd "' + $LinuxRoot + '" --exec "' + $LinuxRoot + '/rust/target/debug/radar-launcher" --web'
$shortcut.IconLocation = (Join-Path $appDir 'radar.ico') + ',0'
$shortcut.Description = 'Open Personal Freelance Radar (local backend + terminal)'
$shortcut.WorkingDirectory = $env:USERPROFILE
$shortcut.Save()
Write-Output ('Created: ' + $shortcutPath)
Write-Output ('Target exists: ' + (Test-Path -LiteralPath $shortcut.TargetPath))
Write-Output ('Icon exists: ' + (Test-Path -LiteralPath (Join-Path $appDir 'radar.ico')))
