#!/bin/bash

clear
echo "========================================"
echo "      EMULADOR CHIP-8 de Mariano"
echo "========================================"
echo "Elige un juego para probar el emulador:"
echo ""
echo "1. Tetris"
echo "2. Fishie (Pecera virtual)"
echo "3. Conway's Life (Simulador de vida)"
echo "4. Jumping X and O (Juego)"
echo ""
read -p "Ingresa un numero (1-4): " choice

# Asignar la ruta de la ROM segun la opcion elegida
case $choice in
    1) ROM="tetris.ch8" ;;
    2) ROM="roms/programs/Fishie [Hap, 2005].ch8" ;;
    3) ROM="roms/programs/Life [GV Samways, 1980].ch8" ;;
    4) ROM="roms/programs/Jumping X and O [Harry Kleinberg, 1977].ch8" ;;
    *) 
       echo "Opcion invalida. Cargando Tetris por defecto..."
       ROM="tetris.ch8" 
       ;;
esac

echo ""
echo "Iniciando emulador con la ROM seleccionada..."

# Detiene el script si falla la compilacion
set -e

cargo run --release -- "$ROM"
