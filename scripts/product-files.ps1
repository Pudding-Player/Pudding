# Shared payload whitelist for development assembly and the installer.
function Get-ProductFiles {
    $projectRoot = Split-Path -Parent $PSScriptRoot
    [pscustomobject]@{ Relative = 'pudding.exe'; Source = (Join-Path $projectRoot 'target/release/pudding.exe') }
    $bridgeJson = & node (Join-Path $projectRoot 'scripts/bridges.mjs') files
    if ($LASTEXITCODE -ne 0) { throw 'Cannot resolve bridge product files' }
    $bridgeJson | ConvertFrom-Json | ForEach-Object { $_ }
    foreach ($pluginName in @('windows-scheduler', 'web-search')) {
        $directory = (Resolve-Path (Join-Path $projectRoot "plugins/$pluginName")).Path
        foreach ($file in Get-ChildItem -LiteralPath $directory -Recurse -File) {
            if ($file.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Redirected plugin file: $($file.FullName)" }
            $name = $file.FullName.Substring($directory.Length + 1).Replace('\', '/')
            if ($name -notin @('.pudding-plugin/plugin.json', 'README.md') -and $name -notmatch '^skills/.+\.md$') { throw "Unexpected core plugin file: $name" }
            [pscustomobject]@{ Relative = "plugins/$pluginName/$name"; Source = $file.FullName }
        }
    }
}
