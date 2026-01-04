# Windows用バグレポート自動収集スクリプト
# 使い方: .\bug-report.ps1 "バグの説明"

param(
    [Parameter(Mandatory=$true)]
    [string]$Description
)

$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$reportDir = "$env:TEMP\idle_factory_bug_$timestamp"
New-Item -ItemType Directory -Path $reportDir -Force | Out-Null

# 1. スクリーンショット取得
Add-Type -AssemblyName System.Windows.Forms
$screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bitmap = New-Object System.Drawing.Bitmap($screen.Width, $screen.Height)
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.CopyFromScreen($screen.Location, [System.Drawing.Point]::Empty, $screen.Size)
$bitmap.Save("$reportDir\screenshot.png")
$graphics.Dispose()
$bitmap.Dispose()

# 2. ログ収集
$logPath = "$env:APPDATA\idle_factory\logs"
if (Test-Path $logPath) {
    $latestLog = Get-ChildItem $logPath -Filter "*.log" | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if ($latestLog) {
        Copy-Item $latestLog.FullName "$reportDir\game.log"
    }
}

# 3. システム情報
$sysinfo = @{
    os = [System.Environment]::OSVersion.VersionString
    timestamp = $timestamp
    description = $Description
}
$sysinfo | ConvertTo-Json | Out-File "$reportDir\info.json"

# 4. ZIP作成
$zipPath = "$env:USERPROFILE\Desktop\bug_report_$timestamp.zip"
Compress-Archive -Path "$reportDir\*" -DestinationPath $zipPath -Force

Write-Host "Bug report created: $zipPath"
Write-Host "Please share this file or upload to a shared location."

# クリップボードにパスをコピー
Set-Clipboard -Value $zipPath
Write-Host "Path copied to clipboard."
