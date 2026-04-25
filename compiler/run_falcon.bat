@echo off
setlocal EnableExtensions

REM Usage: run_falcon.bat <path_to_fc_or_fpy_file> [--verbose]

if "%~1"=="" (
    echo Usage: run_falcon.bat examples\simple_add.fc
    exit /b 1
)

set "INPUT_FILE=%~f1"
set "SCRIPT_DIR=%~dp0"
set "CARGO_EXE=cargo"
set "VERBOSE=0"

if "%~2"=="--verbose" set "VERBOSE=1"
if "%~2"=="-v" set "VERBOSE=1"

if not exist "%INPUT_FILE%" (
    echo Error: Input file not found: %INPUT_FILE%
    exit /b 1
)

where /Q cargo
if errorlevel 1 (
    if exist "%USERPROFILE%\.cargo\bin\cargo.exe" (
        set "CARGO_EXE=%USERPROFILE%\.cargo\bin\cargo.exe"
        set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
    )
)

where /Q "%CARGO_EXE%"
if errorlevel 1 (
    echo Error: cargo not found. Install Rust cargo first.
    exit /b 1
)

if %VERBOSE%==1 echo [1/1] Building and running with Falcon...

pushd "%SCRIPT_DIR%"
if %VERBOSE%==1 (
    "%CARGO_EXE%" run --features llvm --bin falcon -- build "%INPUT_FILE%" --run
) else (
    "%CARGO_EXE%" run -q --features llvm --bin falcon -- build "%INPUT_FILE%" --run
)
set "FALCON_EXIT=%ERRORLEVEL%"
popd

exit /b %FALCON_EXIT%
