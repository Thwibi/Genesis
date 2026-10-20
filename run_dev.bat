@echo off
REM Genesis SOC Auth - dev launcher
REM Initializes the MSVC x64 environment (for SQLCipher/OpenSSL native builds)
REM then runs the Tauri dev server.

set "VSDIR=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools"
call "%VSDIR%\VC\Auxiliary\Build\vcvars64.bat" >nul
if errorlevel 1 (
  echo [ERROR] Failed to initialize MSVC environment.
  exit /b 1
)

REM Toolchain paths: cargo, cmake (SQLCipher), strawberry perl (OpenSSL configure)
set "PATH=%USERPROFILE%\.cargo\bin;C:\Program Files\CMake\bin;C:\Strawberry\perl\bin;C:\Strawberry\c\bin;%PATH%"

cd /d "%~dp0"
echo [Genesis] Starting Tauri dev server...
call npm run tauri dev
