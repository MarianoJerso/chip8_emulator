mod cpu;
use cpu::Cpu;

fn main() {
    println!("Iniciando Emulador CHIP-8...");

    let mut mi_procesador = Cpu::new();

    // Set V0 = 5 (0x6005)
    mi_procesador.cargar_dummy_rom(0x60, 0x05);
    // Add 3 to V0 (0x7003) -> V0 will be 8
    mi_procesador.cargar_dummy_rom_extra(0x70, 0x03, 514);
    // Jump down to 0x200 (infinite loop 0x1200)
    mi_procesador.cargar_dummy_rom_extra(0x12, 0x00, 516);

    println!("ROM de prueba inyectada. ¡Arrancando el reloj!\n");

    mi_procesador.tick();
    mi_procesador.tick();
    mi_procesador.tick();

    println!("\n¡El procesador completó sus primeros ciclos con éxito!");
}
