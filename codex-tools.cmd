@echo off
setlocal EnableExtensions DisableDelayedExpansion
chcp 65001 >nul

set "PROJECT_DIR=%~dp0"
set "APP_DIR=%PROJECT_DIR%apps\Desktop"
pushd "%APP_DIR%" || (
  echo Nao foi possivel acessar a pasta do Codex Tools Desktop.
  exit /b 1
)

:menu
cls
echo.
echo  ========================================
echo               CODEX TOOLS
echo  ========================================
echo.
echo   [1] Iniciar em modo dev
echo   [2] Compilar a release
echo   [3] Sair
echo.
choice /c 123 /n /m "Selecione uma opcao [1-3]: "

if errorlevel 3 goto exit
if errorlevel 2 goto release
if errorlevel 1 goto dev
goto menu

:dev
set "ACTION_EXIT=0"
call :prepare_environment
if errorlevel 1 goto action_failed

echo.
echo Iniciando o Codex Tools em modo dev...
echo Use Ctrl+C para encerrar o servidor e voltar ao menu.
echo.
call npm.cmd run tauri dev
set "ACTION_EXIT=%errorlevel%"
goto action_finished

:release
set "ACTION_EXIT=0"
call :prepare_environment
if errorlevel 1 goto action_failed

echo.
echo Compilando a release do Codex Tools...
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
  echo Node.js nao foi encontrado ou nao pode ser executado.
  echo Instale a versao LTS em https://nodejs.org/en/download e abra um novo terminal.
  exit /b 1
)

call npm.cmd --version >nul 2>&1
if errorlevel 1 (
  echo.
  echo npm nao foi encontrado ou nao pode ser executado.
  echo Instale Node.js LTS com npm em https://nodejs.org/en/download.
  exit /b 1
)

cargo.exe --version >nul 2>&1
if errorlevel 1 (
  echo.
  echo Cargo nao foi encontrado ou nao pode ser executado.
  echo Instale Rust em https://rustup.rs/ e abra um novo terminal.
  exit /b 1
)

if exist "node_modules\.bin\tauri.cmd" if exist "node_modules\.bin\tsc.cmd" if exist "node_modules\.bin\vite.cmd" exit /b 0
echo.
echo Instalando dependencias do projeto a partir de package-lock.json...
call npm.cmd ci
exit /b %errorlevel%

:add_installed_node_to_path
if not defined ProgramFiles exit /b 0
if not exist "%ProgramFiles%\nodejs\node.exe" exit /b 0
if not exist "%ProgramFiles%\nodejs\npm.cmd" exit /b 0
set "PATH=%ProgramFiles%\nodejs;%PATH%"
echo Usando Node.js e npm instalados em %ProgramFiles%\nodejs.
exit /b 0

:add_user_npm_to_path
if not defined APPDATA exit /b 0
if not exist "%APPDATA%\npm\npm.cmd" exit /b 0
set "PATH=%APPDATA%\npm;%PATH%"
echo Usando npm instalado em %APPDATA%\npm.
exit /b 0

:action_failed
set "ACTION_EXIT=1"

:action_finished
echo.
if "%ACTION_EXIT%"=="0" (
  echo Operacao concluida com sucesso.
) else (
  echo A operacao terminou com erro ^(codigo %ACTION_EXIT%^).
)
echo Pressione qualquer tecla para voltar ao menu...
pause >nul
goto menu

:exit
popd
endlocal
exit /b 0
