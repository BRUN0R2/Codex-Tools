@echo off
setlocal EnableExtensions
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
call :require_tool npm
if errorlevel 1 goto action_failed
call :require_tool cargo
if errorlevel 1 goto action_failed

echo.
echo Iniciando o Codex Tools em modo dev...
echo Use Ctrl+C para encerrar o servidor e voltar ao menu.
echo.
call npm run tauri dev
set "ACTION_EXIT=%errorlevel%"
goto action_finished

:release
set "ACTION_EXIT=0"
call :require_tool npm
if errorlevel 1 goto action_failed
call :require_tool cargo
if errorlevel 1 goto action_failed

echo.
echo Compilando a release do Codex Tools...
echo.
call npm run tauri build
set "ACTION_EXIT=%errorlevel%"
goto action_finished

:require_tool
where %~1 >nul 2>&1
if errorlevel 1 (
  echo.
  echo Ferramenta obrigatoria nao encontrada: %~1
  exit /b 1
)
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
