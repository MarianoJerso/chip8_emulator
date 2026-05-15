@echo off
cls
echo ========================================
echo       EMULADOR CHIP-8 de Mariano
echo ========================================
echo Choose a game to test the emulator:
echo.
echo 1. Tetris
echo 2. Pong
echo 3. Invaders
echo 4. Breakout
echo.
set /p choice="Enter a number (1-4): "

:: Inicializar la variable vacia
set "ROM="

:: Asignar usando sintaxis segura de comillas para evitar errores con espacios y corchetes
if "%choice%"=="1" set "ROM=roms\tetris.ch8"
if "%choice%"=="2" set "ROM=roms\pong.ch8"
if "%choice%"=="3" set "ROM=roms\invaders.ch8"
if "%choice%"=="4" set "ROM=roms\breakout.ch8"

:: Validar si esta vacia (opcion incorrecta)
if not defined ROM (
    echo Invalid option. Loading Tetris by default...
    set "ROM=tetris.ch8"
)

echo.
echo Starting emulator with the selected ROM...
cargo run --release -- "%ROM%"

:: Pausa si hay un error
if %errorlevel% neq 0 (
    echo.
    echo An error occurred while running the emulator.
    pause
)
