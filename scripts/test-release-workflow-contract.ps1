[CmdletBinding()]
param([string]$ExpectedVersion)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$root = Split-Path -Parent $PSScriptRoot
$workflow = Get-Content -Raw -LiteralPath (Join-Path $root '.github\workflows\release.yml')
$ci = Get-Content -Raw -LiteralPath (Join-Path $root '.github\workflows\ci.yml')
$publisher = Get-Content -Raw -LiteralPath (Join-Path $root 'scripts\publish-github-release.ps1')

function Assert-True {
    param([bool]$Condition, [string]$Message)
    if (-not $Condition) { throw "FAIL: $Message" }
}

foreach ($token in @(
    'workflow_dispatch:',
    'tag:',
    'publish:',
    "default: false",
    "^v[0-9]+\.[0-9]+\.[0-9]+$",
    'permissions:',
    'contents: read',
    'runs-on: [self-hosted, Windows, X64, windowsbuild]',
    'ref: ${{ inputs.tag }}',
    '-UpdateRepository Terrieggwp/terrieflow-client `',
    'TerrieFlow-Client-windows-x64.zip',
    'TerrieFlow-Client-Setup-x64.exe',
    'RELEASE-SHA256SUMS.txt',
    'Xray-core-source.zip',
    'mihomo-source.zip',
    'smoke-test-windows-installed-update.ps1',
    'environment: terrieflow-release',
    'if: ${{ inputs.publish }}',
    'contents: write',
    '-Repository Terrieggwp/terrieflow-client `',
    'Point release.txt at the published version'
)) {
    Assert-True ($workflow.Contains($token)) "release workflow is missing: $token"
}

Assert-True (-not ($workflow -match '(?m)^\s*push:\s*$')) 'release must never run from a push or tag automatically'
Assert-True (-not $workflow.Contains("- 'v*'")) 'release must not have a wildcard tag trigger'
Assert-True (-not $workflow.Contains('pull_request_target')) 'release must not run in pull_request_target'
Assert-True (-not $workflow.Contains('fatyzzz/multicore')) 'official release must not target the upstream repository'
Assert-True (-not $workflow.Contains('multicore-releases')) 'official release must not target a legacy mirror'
Assert-True (-not $workflow.Contains('windows-x86.zip')) 'first beta must not publish Windows x86'
Assert-True (-not $workflow.Contains('windows-arm64.zip')) 'first beta must not publish Windows arm64'
Assert-True (-not $workflow.Contains('AppImage')) 'first beta must not publish Linux packages'
Assert-True (-not $workflow.Contains('GH_TOKEN: ${{ secrets.')) 'release must not require a personal access token'
Assert-True (-not $workflow.Contains('MULTICORE_DAEMON_TOKEN')) 'release must not embed runtime secrets'

$publishIndex = $workflow.IndexOf('Publish release to official update channel', [StringComparison]::Ordinal)
$channelIndex = $workflow.IndexOf('Point release.txt at the published version', [StringComparison]::Ordinal)
Assert-True ($publishIndex -ge 0 -and $channelIndex -gt $publishIndex) `
    'release.txt must only move after the GitHub release is published'

foreach ($action in @('actions/checkout@', 'actions/upload-artifact@', 'actions/download-artifact@')) {
    $matches = [regex]::Matches($workflow, [regex]::Escape($action) + '([0-9a-f]{40})')
    Assert-True ($matches.Count -gt 0) "$action must be used"
    foreach ($match in $matches) {
        Assert-True ($match.Groups[1].Value.Length -eq 40) "$action must be pinned to a commit"
    }
}

Assert-True ($publisher.Contains("`$OfficialRepository = 'Terrieggwp/terrieflow-client'")) `
    'publisher must be restricted to the official repository'
Assert-True ($publisher.Contains("'User-Agent'           = 'terrieflow-release-workflow'")) `
    'publisher must identify TerrieFlow'
Assert-True ($publisher.Contains('[Text.Encoding]::UTF8.GetBytes')) `
    'release metadata must be encoded as UTF-8 bytes'
Assert-True ($publisher.Contains('application/json; charset=utf-8')) `
    'release API requests must declare UTF-8 JSON'

Assert-True ($ci.Contains("- 'codex/**'")) 'CI must run for Codex development branches'
Assert-True (-not ([regex]::IsMatch($ci, '(?m)^  linux-packages:\s*$'))) `
    'normal CI must not create unsupported Linux release artifacts'
Assert-True ($ci.Contains('test-release-workflow-contract.ps1')) 'normal CI must verify the release contract'

if ($ExpectedVersion) {
    $manifest = Get-Content -Raw -LiteralPath (Join-Path $root 'apps\multicore-desktop\Cargo.toml')
    $installer = Get-Content -Raw -LiteralPath (Join-Path $root 'scripts\build-windows-installer.ps1')
    $ui = Get-Content -Raw -LiteralPath (Join-Path $root 'apps\multicore-desktop\ui\app.slint')
    Assert-True ([regex]::IsMatch($manifest, ('(?m)^version\s*=\s*"{0}"\s*$' -f [regex]::Escape($ExpectedVersion)))) `
        'desktop manifest must match the expected version'
    Assert-True ($installer.Contains(("[string]`$AppVersion = '{0}'" -f $ExpectedVersion))) `
        'installer default must match the expected version'
    Assert-True ($ui.Contains(('update-status: "TerrieFlow {0}";' -f $ExpectedVersion))) `
        'desktop fallback version must match the expected version'
}

Write-Output 'PASS: TerrieFlow Windows x64 release workflow contract'
