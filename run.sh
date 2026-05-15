#!/bin/bash

clear
echo "========================================"
echo "      EMULADOR CHIP-8 de Mariano"
echo "========================================"
echo "Choose a game to test the emulator:"
echo ""
echo "1. Tetris"
echo "2. Pong"    
echo "3. Invaders"
echo "4. Breakout"
echo ""
read -p "Enter a number (1-4): " choice

# Asignar la ruta de la ROM segun la opcion elegida
case $choice in
    1) ROM="roms/tetris.ch8" ;;
    2) ROM="roms/pong.ch8" ;;
    3) ROM="roms/invadersch8" ;;
    4) ROM="roms/breakout.ch8" ;;
    *) 
       echo "Invalid option. Loading Tetris by default..."
       ROM="tetris.ch8" 
       ;;
esac

echo ""
echo "Starting emulator with the selected ROM..."

# Detiene el script si falla la compilacion
set -e

cargo run --release -- "$ROM"
