@echo off
setlocal EnableExtensions DisableDelayedExpansion
chcp 65001 >nul

set "PROJECT_DIR=%~dp0"
set "APP_DIR=%PROJECT_DIR%apps\Desktop"
pushd "%APP_DIR%" || (
  echo Could not access the Codex Tools Desktop folder.
  exit /b 1
)

:menu
cls
echo.
echo  ========================================
echo               CODEX TOOLS
echo  ========================================
echo.
echo   [1] Start in development mode
echo   [2] Build release
echo   [3] Exit
echo.
choice /c 123 /n /m "Select an option [1-3]: "

if errorlevel 3 goto exit
if errorlevel 2 goto release
if errorlevel 1 goto dev
goto menu

:dev
set "ACTION_EXIT=0"
call :prepare_environment
if errorlevel 1 goto action_failed

echo.
echo Starting Codex Tools in development mode...
echo Press Ctrl+C to stop the server and return to the menu.
echo.
call npm.cmd run tauri dev
set "ACTION_EXIT=%errorlevel%"
goto action_finished

:release
set "ACTION_EXIT=0"
call :prepare_environment
if errorlevel 1 goto action_failed

echo.
echo Building the Codex Tools release...
echo.
call npm.cmd run tauri build
set "ACTION_EXIT=%errorlevel%"
goto action_finished

:prepare_environment
where node.exe >nul 2>&1
if errorlevel 1 call :add_installed_node_to_path
where npm.cmd >nul 2>&1
if errorlevel 1 call :add_installed_node_to_path
where npm.cmd >nul 2>&1
if errorlevel 1 call :add_user_npm_to_path

node.exe --version >nul 2>&1
if errorlevel 1 (
  echo.
  echo Node.js was not found or could not be run.
  echo Install the LTS version from https://nodejs.org/en/download and open a new terminal.
  exit /b 1
)

call npm.cmd --version >nul 2>&1
if errorlevel 1 (
  echo.
  echo npm was not found or could not be run.
  echo Install Node.js LTS with npm from https://nodejs.org/en/download.
  exit /b 1
)

cargo.exe --version >nul 2>&1
if errorlevel 1 (
  echo.
  echo Cargo was not found or could not be run.
  echo Install Rust from https://rustup.rs/ and open a new terminal.
  exit /b 1
)

if exist "node_modules\.bin\tauri.cmd" if exist "node_modules\.bin\tsc.cmd" if exist "node_modules\.bin\vite.cmd" exit /b 0
echo.
echo Installing project dependencies from package-lock.json...
call npm.cmd ci
exit /b %errorlevel%

:add_installed_node_to_path
if not defined ProgramFiles exit /b 0
if not exist "%ProgramFiles%\nodejs\node.exe" exit /b 0
if not exist "%ProgramFiles%\nodejs\npm.cmd" exit /b 0
set "PATH=%ProgramFiles%\nodejs;%PATH%"
echo Using Node.js and npm installed at %ProgramFiles%\nodejs.
exit /b 0

:add_user_npm_to_path
if not defined APPDATA exit /b 0
if not exist "%APPDATA%\npm\npm.cmd" exit /b 0
set "PATH=%APPDATA%\npm;%PATH%"
echo Using npm installed at %APPDATA%\npm.
exit /b 0

:action_failed
set "ACTION_EXIT=1"

:action_finished
echo.
if "%ACTION_EXIT%"=="0" (
  echo Operation completed successfully.
) else (
  echo Operation failed with exit code %ACTION_EXIT%.
)
echo Press any key to return to the menu...
pause >nul
goto menu

:exit
popd
endlocal
exit /b 0
