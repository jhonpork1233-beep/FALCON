@echo off
setlocal EnableExtensions

REM Usage:
REM   falcon factorial.fc
REM   falcon build factorial.fc --profile=kernel

set "SCRIPT_DIR=%~dp0"
set "FALCON_BIN="
set "CARGO_EXE=cargo"
set "USE_FILE_ARG=0"

if /I "%~x1"==".fc" set "USE_FILE_ARG=1"
if /I "%~x1"==".fpy" set "USE_FILE_ARG=1"

where /Q cargo
if errorlevel 1 (
    if exist "%USERPROFILE%\.cargo\bin\cargo.exe" (
        set "CARGO_EXE=%USERPROFILE%\.cargo\bin\cargo.exe"
        set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
    )
)

if exist "%SCRIPT_DIR%target\debug\falcon.exe" (
    set "FALCON_BIN=%SCRIPT_DIR%target\debug\falcon.exe"
) else if exist "%SCRIPT_DIR%target\release\falcon.exe" (
    set "FALCON_BIN=%SCRIPT_DIR%target\release\falcon.exe"
)

if defined FALCON_BIN (
    if "%USE_FILE_ARG%"=="1" (
        "%FALCON_BIN%" "%~f1" %2 %3 %4 %5 %6 %7 %8 %9
    ) else (
        "%FALCON_BIN%" %*
    )
    exit /b %ERRORLEVEL%
)

where /Q "%CARGO_EXE%"
if errorlevel 1 (
    echo Error: Falcon binary not found and cargo is unavailable.
    echo Install Rust or run install-falcon.ps1 first.
    exit /b 1
)

pushd "%SCRIPT_DIR%"
if "%USE_FILE_ARG%"=="1" (
    "%CARGO_EXE%" run -q --features llvm --bin falcon -- "%~f1" %2 %3 %4 %5 %6 %7 %8 %9
) else (
    "%CARGO_EXE%" run -q --features llvm --bin falcon -- %*
)
set "FALCON_EXIT=%ERRORLEVEL%"
popd

exit /b %FALCON_EXIT%
