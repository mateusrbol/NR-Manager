@echo off
setlocal EnableExtensions
title NR Manager - Dev
cd /d "%~dp0"

where node >nul 2>nul || (echo [ERRO] Node.js nao encontrado. & pause & exit /b 1)
where cargo >nul 2>nul || (echo [ERRO] Rust/cargo nao encontrado. & pause & exit /b 1)

if not exist "node_modules" (
  echo Instalando dependencias...
  call npm install || (pause & exit /b 1)
)

if not exist "src-tauri\icons\icon.ico" (
  powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\gen-icons.ps1"
)

echo Iniciando NR Manager em modo desenvolvimento...
call npm run tauri:dev
