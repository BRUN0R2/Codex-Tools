param(
    [ValidateSet('Open', 'RunAs', 'AppsFolderRunAs', 'AppsFolderRunAsArgs')]
    [string]$LaunchMode = 'RunAs',
    [switch]$InspectOnly,
    [switch]$IsolatedProfile,
    [string]$UserDataDirectory,
    [string]$OutputPath,
    [int]$ObserveSeconds = 8,
    [int]$PollMilliseconds = 20
)

$ErrorActionPreference = 'Stop'
$package = Get-AppxPackage -Name 'OpenAI.Codex'
if ($null -eq $package) {
    throw 'OpenAI.Codex is not installed for the current user.'
}

$executablePath = Join-Path $package.InstallLocation 'app\ChatGPT.exe'
if (-not (Test-Path -LiteralPath $executablePath)) {
    throw "Codex desktop executable not found: $executablePath"
}

$existingProcessIds = @(
    Get-Process -Name ChatGPT -ErrorAction SilentlyContinue |
        Select-Object -ExpandProperty Id
)
$profilePath = $UserDataDirectory
if ($IsolatedProfile -and $UserDataDirectory) {
    throw 'Specify either IsolatedProfile or UserDataDirectory.'
}
if ($IsolatedProfile) {
    if ($LaunchMode -eq 'AppsFolderRunAs') {
        throw 'The AppsFolder verb does not accept a user data directory.'
    }
    $profilePath = Join-Path $env:TEMP ("codex-elevation-profile-{0}" -f [guid]::NewGuid())
}
if ($profilePath -and $LaunchMode -eq 'AppsFolderRunAs') {
    throw 'The AppsFolder verb does not accept a user data directory.'
}
$readyPath = Join-Path $env:TEMP ("codex-elevation-probe-{0}.ready" -f [guid]::NewGuid())
$nativeSource = @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;

public static class CodexElevationProbeNative
{
    private const uint ProcessQueryLimitedInformation = 0x1000;
    private const uint TokenQuery = 0x0008;
    private const int TokenElevation = 20;
    private const int ErrorInsufficientBuffer = 122;
    private const int AppModelErrorNoPackage = 15700;

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern IntPtr OpenProcess(uint access, bool inheritHandle, int processId);

    [DllImport("advapi32.dll", SetLastError = true)]
    private static extern bool OpenProcessToken(IntPtr process, uint access, out IntPtr token);

    [DllImport("advapi32.dll", SetLastError = true)]
    private static extern bool GetTokenInformation(
        IntPtr token, int informationClass, out int information, int length, out int returnedLength);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool CloseHandle(IntPtr handle);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern int GetPackageFullName(IntPtr process, ref uint length, IntPtr name);

    public static string PackageIdentity(int processId)
    {
        IntPtr process = OpenProcess(ProcessQueryLimitedInformation, false, processId);
        if (process == IntPtr.Zero)
            throw new Win32Exception(Marshal.GetLastWin32Error());

        try
        {
            uint length = 0;
            int result = GetPackageFullName(process, ref length, IntPtr.Zero);
            if (result == AppModelErrorNoPackage)
                return "none";
            if (result != ErrorInsufficientBuffer)
                throw new Win32Exception(result);

            IntPtr name = Marshal.AllocHGlobal(checked((int)length * sizeof(char)));
            try
            {
                result = GetPackageFullName(process, ref length, name);
                if (result != 0)
                    throw new Win32Exception(result);
                return Marshal.PtrToStringUni(name);
            }
            finally
            {
                Marshal.FreeHGlobal(name);
            }
        }
        finally
        {
            CloseHandle(process);
        }
    }

    public static bool IsElevated(int processId)
    {
        IntPtr process = OpenProcess(ProcessQueryLimitedInformation, false, processId);
        if (process == IntPtr.Zero)
            throw new Win32Exception(Marshal.GetLastWin32Error());

        try
        {
            IntPtr token;
            if (!OpenProcessToken(process, TokenQuery, out token))
                throw new Win32Exception(Marshal.GetLastWin32Error());

            try
            {
                int elevation;
                int returnedLength;
                if (!GetTokenInformation(token, TokenElevation, out elevation,
                    sizeof(int), out returnedLength))
                    throw new Win32Exception(Marshal.GetLastWin32Error());

                return elevation != 0;
            }
            finally
            {
                CloseHandle(token);
            }
        }
        finally
        {
            CloseHandle(process);
        }
    }
}
'@

if ($InspectOnly) {
    Add-Type -TypeDefinition $nativeSource
    Get-CimInstance Win32_Process -Filter "name = 'ChatGPT.exe' or name = 'codex.exe'" |
        ForEach-Object {
            $process = $_
            try {
                $elevated = [CodexElevationProbeNative]::IsElevated($process.ProcessId)
                $elevation = if ($elevated) { 'administrator' } else { 'normal' }
                $packageIdentity = [CodexElevationProbeNative]::PackageIdentity($process.ProcessId)
            }
            catch {
                $elevation = "unavailable: $($_.Exception.Message)"
                $packageIdentity = 'unavailable'
            }

            [pscustomobject]@{
                ProcessId = $process.ProcessId
                ParentProcessId = $process.ParentProcessId
                Name = $process.Name
                Elevation = $elevation
                PackageIdentity = $packageIdentity
            }
        }
    return
}

$observer = Start-Job -ArgumentList $existingProcessIds, $nativeSource, $ObserveSeconds, $PollMilliseconds, $readyPath -ScriptBlock {
    param($baseline, $source, $durationSeconds, $intervalMilliseconds, $signalPath)

    Add-Type -TypeDefinition $source
    $seen = @{}
    $stopwatch = [Diagnostics.Stopwatch]::StartNew()
    Set-Content -LiteralPath $signalPath -Value 'ready'

    while ($stopwatch.Elapsed.TotalSeconds -lt $durationSeconds) {
        foreach ($process in @(Get-Process -Name ChatGPT -ErrorAction SilentlyContinue)) {
            if ($baseline -contains $process.Id -or $seen.ContainsKey($process.Id)) {
                continue
            }

            $seen[$process.Id] = $true
            try {
                $elevated = [CodexElevationProbeNative]::IsElevated($process.Id)
                $result = if ($elevated) { 'administrator' } else { 'normal' }
                $packageIdentity = [CodexElevationProbeNative]::PackageIdentity($process.Id)
            }
            catch {
                $result = "unavailable: $($_.Exception.Message)"
                $packageIdentity = 'unavailable'
            }

            [pscustomobject]@{
                ProcessId = $process.Id
                Elevation = $result
                PackageIdentity = $packageIdentity
                ObservedAfterMilliseconds = [math]::Round($stopwatch.Elapsed.TotalMilliseconds)
            }
        }

        Start-Sleep -Milliseconds $intervalMilliseconds
    }
}

try {
    $readyDeadline = [Diagnostics.Stopwatch]::StartNew()
    while (-not (Test-Path -LiteralPath $readyPath)) {
        if ($readyDeadline.Elapsed.TotalSeconds -ge 5) {
            throw 'The process observer did not become ready.'
        }
        Start-Sleep -Milliseconds 50
    }

    try {
        if ($LaunchMode -eq 'AppsFolderRunAs') {
            $item = (New-Object -ComObject Shell.Application).NameSpace('shell:AppsFolder').Items() |
                Where-Object { $_.Path -eq "$($package.PackageFamilyName)!App" }
            if ($null -eq $item) {
                throw 'The Codex AppsFolder item was not found.'
            }
            $verb = $item.Verbs() |
                Where-Object { $_.Name -match 'administrator|administrador' } |
                Select-Object -First 1
            if ($null -eq $verb) {
                throw 'The AppsFolder item has no administrator verb.'
            }
            $verb.DoIt()
            $launchResult = "Invoked AppsFolder verb: $($verb.Name)"
        }
        elseif ($LaunchMode -eq 'AppsFolderRunAsArgs') {
            $arguments = '--do-not-de-elevate'
            if ($profilePath) {
                $arguments += " --user-data-dir=`"$profilePath`""
            }
            $applicationId = "$($package.PackageFamilyName)!App"
            Start-Process -FilePath "shell:AppsFolder\$applicationId" -Verb RunAs -ArgumentList $arguments
            $launchResult = 'Requested AppsFolder launch with arguments'
        }
        else {
            $arguments = '--do-not-de-elevate'
            if ($profilePath) {
                $arguments += " --user-data-dir=`"$profilePath`""
            }

            $launchParameters = @{
                FilePath = $executablePath
                ArgumentList = $arguments
                PassThru = $true
            }
            if ($LaunchMode -eq 'RunAs') {
                $launchParameters.Verb = 'RunAs'
            }

            $launchedProcess = Start-Process @launchParameters
            $launchResult = "Started PID $($launchedProcess.Id)"
        }
    }
    catch {
        $launchResult = "Launch failed: $($_.Exception.Message)"
    }

    Wait-Job -Job $observer | Out-Null
    $observedProcesses = @(Receive-Job -Job $observer)
    $remainingProcessIds = @(
        Get-Process -Name ChatGPT -ErrorAction SilentlyContinue |
            Where-Object { $existingProcessIds -notcontains $_.Id } |
            Select-Object -ExpandProperty Id
    )

    $report = [pscustomobject]@{
        PackageVersion = $package.Version.ToString()
        LaunchMode = $LaunchMode
        ProfilePath = $profilePath
        LaunchResult = $launchResult
        ExistingProcessIds = ($existingProcessIds -join ', ')
        ObservedProcesses = $observedProcesses
        RemainingProcessIds = ($remainingProcessIds -join ', ')
    }
    if ($OutputPath) {
        $report | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $OutputPath -Encoding UTF8
    }
    else {
        $report
    }
}
finally {
    if ($observer.State -eq 'Running') {
        Stop-Job -Job $observer
    }
    Remove-Job -Job $observer -Force
    Remove-Item -LiteralPath $readyPath -ErrorAction SilentlyContinue
}
