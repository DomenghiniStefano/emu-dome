# Runs scripts/check (the list of every check) from PowerShell on Windows.
#
# scripts/check is a POSIX sh script, the one list shared with Linux and the pre-commit hook.
# PowerShell has no `sh` on its PATH (Git for Windows puts only Git\cmd there), so this finds
# the sh.exe that ships with Git for Windows next to git.exe and runs the script with it.
# Exit code: the one of scripts/check (0 = all green).

$ErrorActionPreference = 'Stop'

$sh = Get-Command sh -ErrorAction SilentlyContinue
if ($null -ne $sh) {
    $shPath = $sh.Source
} else {
    # git.exe lives in <Git>\cmd (or <Git>\bin); sh.exe in <Git>\bin.
    $git = (Get-Command git).Source
    $gitRoot = Split-Path (Split-Path $git -Parent) -Parent
    $shPath = Join-Path $gitRoot 'bin\sh.exe'
    if (-not (Test-Path $shPath)) {
        Write-Error "sh.exe not found next to $git. Install Git for Windows, or run scripts/check from Git Bash."
    }
}

$check = Join-Path $PSScriptRoot 'check'
& $shPath $check
exit $LASTEXITCODE
