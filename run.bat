@echo off
cls
echo ========================================
echo       EMULADOR CHIP-8 de Mariano
echo ========================================
echo Elige un juego para probar el emulador:
echo.
echo 1. Tetris
echo 2. Fishie (Pecera virtual)
echo 3. Conway's Life (Simulador de vida)
echo 4. Jumping X and O (Juego)
echo.
set /p choice="Ingresa un numero (1-4): "

:: Inicializar la variable vacia
set "ROM="

:: Asignar usando sintaxis segura de comillas para evitar errores con espacios y corchetes
if "%choice%"=="1" set "ROM=roms\programs\tetris.ch8"
if "%choice%"=="2" set "ROM=roms\programs\Fishie [Hap, 2005].ch8"
if "%choice%"=="3" set "ROM=roms\programs\Life [GV Samways, 1980].ch8"
if "%choice%"=="4" set "ROM=roms\programs\Jumping X and O [Harry Kleinberg, 1977].ch8"

:: Validar si esta vacia (opcion incorrecta)
if not defined ROM (
    echo Opcion invalida. Cargando Tetris por defecto...
    set "ROM=tetris.ch8"
)

echo.
echo Iniciando emulador con la ROM seleccionada...
cargo run --release -- "%ROM%"

:: Pausa si hay un error
if %errorlevel% neq 0 (
    echo.
    echo Ocurrio un error al ejecutar el emulador.
    pause
)
