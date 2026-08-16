# 自动更新 easytier-core 嵌入式资源（联机虚拟组网）
#
# 对比 GitHub Releases 最新 stable 与 src-tauri/build_script/easytier.rs 记录的版本，
# 不一致则下载 6 个平台包解压替换 src-tauri/resources/easytier/{os}/{arch}/ 并更新版本常量。
# 由 .github/workflows/update-easytier.yml 每日调度调用；本地可手动执行（需网络）。

$ErrorActionPreference = "Stop"

$headers = @{ 'User-Agent' = 'MoLaunch-update-bot' }
if ($env:GITHUB_TOKEN) {
    $headers['Authorization'] = "Bearer $env:GITHUB_TOKEN"
}
$release = Invoke-RestMethod -Uri 'https://api.github.com/repos/EasyTier/EasyTier/releases/latest' -Headers $headers
$tag = $release.tag_name
$version = $tag.TrimStart('v')

$rsPath = 'src-tauri/build_script/easytier.rs'
$rsContent = Get-Content $rsPath -Raw
if ($rsContent -notmatch 'const EASYTIER_VERSION: &str = "([^"]+)"') {
    throw "无法从 $rsPath 解析 EASYTIER_VERSION 常量"
}
$current = $Matches[1]

function Set-Output {
    param([string]$Name, [string]$Value)
    if ($env:GITHUB_OUTPUT) {
        Add-Content -Path $env:GITHUB_OUTPUT -Value "$Name=$Value"
    }
}

if ($current -eq $version) {
    Write-Host "easytier-core 已是最新 v$version，无需更新"
    Set-Output -Name 'updated' -Value 'false'
    Set-Output -Name 'version' -Value $version
    exit 0
}

Write-Host "发现新版本 v$current -> v$version，开始下载替换..."

# 上游 release 资产 → 项目资源目录映射（Windows arm64 资产名为 arm64，与目录 aarch64 不同）
$platforms = @(
    'windows/x86_64/easytier-windows-x86_64'
    'windows/aarch64/easytier-windows-arm64'
    'linux/x86_64/easytier-linux-x86_64'
    'linux/aarch64/easytier-linux-aarch64'
    'macos/x86_64/easytier-macos-x86_64'
    'macos/aarch64/easytier-macos-aarch64'
)

$workDir = Join-Path $env:TEMP "easytier-update-$([guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $workDir | Out-Null
try {
    foreach ($entry in $platforms) {
        $os, $arch, $asset = $entry -split '/'
        $zipName = "$asset-v$version.zip"
        $zipPath = Join-Path $workDir $zipName
        $unzipDir = Join-Path $workDir "$os-$arch"
        $downloadUrl = "https://github.com/EasyTier/EasyTier/releases/download/$tag/$zipName"

        Write-Host "下载 $zipName ..."
        Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath -UseBasicParsing
        Expand-Archive -Path $zipPath -DestinationPath $unzipDir

        $core = Get-ChildItem -Path $unzipDir -Recurse -File |
            Where-Object { $_.Name -in @('easytier-core', 'easytier-core.exe') } |
            Select-Object -First 1
        if (-not $core) {
            throw "$zipName 中未找到 easytier-core 可执行文件"
        }

        $destDir = "src-tauri/resources/easytier/$os/$arch"
        New-Item -ItemType Directory -Force -Path $destDir | Out-Null
        Copy-Item $core.FullName (Join-Path $destDir $core.Name) -Force

        if ($os -eq 'windows') {
            foreach ($dll in @('Packet.dll', 'wintun.dll')) {
                $dllFile = Get-ChildItem -Path $unzipDir -Recurse -File -Filter $dll | Select-Object -First 1
                if ($dllFile) {
                    Copy-Item $dllFile.FullName (Join-Path $destDir $dll) -Force
                }
                else {
                    Write-Warning "$zipName 中未找到 $dll（上游可能调整了打包结构）"
                }
            }
        }
        Write-Host "  更新 $os/$arch 完成"
    }

    $newRs = $rsContent -replace 'const EASYTIER_VERSION: &str = "[^"]+";', "const EASYTIER_VERSION: &str = `"$version`";"
    [System.IO.File]::WriteAllText((Resolve-Path $rsPath), $newRs)
    Write-Host "版本常量已更新: $rsPath -> v$version"
}
finally {
    Remove-Item -Path $workDir -Recurse -Force -ErrorAction SilentlyContinue
}

Set-Output -Name 'updated' -Value 'true'
Set-Output -Name 'version' -Value $version
