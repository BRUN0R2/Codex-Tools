param(
    [Parameter(Mandatory = $true)]
    [int]$TargetProcessId,
    [Parameter(Mandatory = $true)]
    [string]$OutputPath,
    [switch]$InvokeAdminLaunch
)

$ErrorActionPreference = 'Stop'

try {
    Add-Type -AssemblyName UIAutomationClient
    $root = [System.Windows.Automation.AutomationElement]::RootElement
    $condition = New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::ProcessIdProperty,
        $TargetProcessId
    )
    $window = $root.FindFirst([System.Windows.Automation.TreeScope]::Children, $condition)
    if ($null -eq $window) {
        throw 'The Codex Tools window was not found.'
    }
    $elements = $window.FindAll(
        [System.Windows.Automation.TreeScope]::Descendants,
        [System.Windows.Automation.Condition]::TrueCondition
    )
    $controls = @($elements | ForEach-Object {
        [pscustomobject]@{
            Name = $_.Current.Name
            ControlType = $_.Current.ControlType.ProgrammaticName
        }
    })

    $invoked = $false
    if ($InvokeAdminLaunch) {
        $button = @($elements | Where-Object {
            $_.Current.ControlType -eq [System.Windows.Automation.ControlType]::Button -and
            $_.Current.Name -eq 'Abrir como administrador'
        }) | Select-Object -First 1
        if ($null -eq $button) {
            throw 'The administrator launch button was not found.'
        }
        $pattern = $button.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)
        $pattern.Invoke()
        $invoked = $true
    }

    [pscustomobject]@{
        TargetProcessId = $TargetProcessId
        Invoked = $invoked
        Controls = $controls
        Error = $null
    } | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $OutputPath -Encoding UTF8
}
catch {
    [pscustomobject]@{
        TargetProcessId = $TargetProcessId
        Invoked = $false
        Controls = @()
        Error = $_.Exception.Message
    } | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $OutputPath -Encoding UTF8
    throw
}
