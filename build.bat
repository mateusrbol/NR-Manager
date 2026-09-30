@echo off
setlocal EnableExtensions
title NR Manager - Build
cd /d "%~dp0"

echo ============================================
echo   NR Manager - build do instalador
echo ============================================
echo.

where node >nul 2>nul
if errorlevel 1 (
  echo [ERRO] Node.js nao encontrado no PATH.
  echo        Instale em https://nodejs.org e rode novamente.
  goto :fail
)

where npm >nul 2>nul
if errorlevel 1 (
  echo [ERRO] npm nao encontrado no PATH.
  goto :fail
)

where cargo >nul 2>nul
if errorlevel 1 (
  echo [ERRO] Rust/cargo nao encontrado no PATH.
  echo        Instale em https://rustup.rs ^(rustup-init.exe^) e rode novamente.
  goto :fail
)

echo [1/3] Gerando icones (se necessario)...
if not exist "src-tauri\icons\icon.ico" (
  powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\gen-icons.ps1"
)

echo [2/3] Instalando dependencias do frontend...
call npm install
if errorlevel 1 goto :fail

echo [3/3] Compilando (Tauri build / NSIS)...
call npm run tauri:build
if errorlevel 1 goto :fail

echo.
echo ============================================
echo   BUILD CONCLUIDO
echo ============================================
echo Instalador : src-tauri\target\release\bundle\nsis\
echo Executavel : src-tauri\target\release\NR Manager.exe
echo.
pause
exit /b 0

:fail
echo.
echo [FALHA] O build nao foi concluido. Veja as mensagens acima.
pause
exit /b 1
