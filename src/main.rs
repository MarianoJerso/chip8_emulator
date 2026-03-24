mod cpu;
use cpu::Cpu;
use minifb::{Key, Window, WindowOptions, Scale};
use std::env;

// Usamos el mismo tamaño estándar universal del CHIP-8 original
const PANTALLA_ANCHO: usize = 64;
const PANTALLA_ALTO: usize = 32;

fn main() {
    // Le pedimos a Rust que capture el texto extra que le pasamos por la terminal (Ej. "cargo run -- tetris.ch8")
    let argumentos: Vec<String> = env::args().collect();
    if argumentos.len() < 2 {
        eprintln!("❌ ERROR FATAL: ¡Prendiste la consola sin meter ningún cartucho!");
        eprintln!("Uso correcto en la terminal: cargo run -- <nombre_del_juego.ch8>");
        return;
    }

    let ruta_cartucho = &argumentos[1];
    println!("Iniciando Emulador CHIP-8... 🚀");
    println!("Soplando el cartucho '{}' e insertándolo en la placa madre...", ruta_cartucho);

    let mut cpu = Cpu::new();

    // La lectora de discos real de Rust: lee los 1s y 0s puros del archivo desde tu disco duro
    let rom = std::fs::read(ruta_cartucho).unwrap_or_else(|error| {
        panic!("❌ El archivo del juego no existe o está corrupto: {}", error);
    });

    // "Insertamos" la ROM en el hardware empezando por su memoria 0x200
    cpu.cargar_rom(&rom);

    // =====================================================================
    // MOTOR GRÁFICO (CONEXIÓN CON WINDOWS VÍA MINIFB)
    // =====================================================================

    // Creamos la Ventana de Hardware del SO.
    // Usamos Scale::X16 para que la micro pantalla original de 64x32
    // sea visible y cómoda de ver estirando los pixeles x16 en un monitor 1080p
    let mut window = Window::new(
        "Emulador CHIP-8 - Hecho en Rust por Mariano",
        PANTALLA_ANCHO,
        PANTALLA_ALTO,
        WindowOptions {
            scale: Scale::X16,
            ..WindowOptions::default()
        },
    ).unwrap_or_else(|e| {
        panic!("La placa de video devolvió un error grave al abrir la ventana: {}", e);
    });

    // Limitamos el framerate del monitor para que dibuje 60 Cuadros por Segundo (60 FPS fijos)
    // 16600 microsegundos = ~16.6 ms (Que equivale exactamente a 60 Hz).
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    // Minifb requiere que le pasemos TODOS los 2048 pixeles de la pantalla como una sola línea recta,
    // en lugar de una matriz cuadrada bidimensional. Se lo pasamos como enteros (u32 hex color).
    let mut buffer: Vec<u32> = vec![0; PANTALLA_ANCHO * PANTALLA_ALTO];

    // =====================================================================
    // EL GAME LOOP PRINCIPAL
    // Este bucle infinito corre constantemente mientras la x roja de la ventana no se toque
    // o mientras el usuario no mantenga presionada la tecla ESCAPE
    // =====================================================================
    while window.is_open() && !window.is_key_down(Key::Escape) {
        
        // --- 0. CONTROLADOR DE INPUT (JOYSTICK) ---
        // Le preguntamos a la librería minifb (y por extensión, a Windows) qué teclas 
        // están físicamente pulsadas en este milisegundo exacto, y actualizamos nuestro Array interno simulado.
        cpu.teclas[0x1] = window.is_key_down(Key::Key1);
        cpu.teclas[0x2] = window.is_key_down(Key::Key2);
        cpu.teclas[0x3] = window.is_key_down(Key::Key3);
        cpu.teclas[0xC] = window.is_key_down(Key::Key4);
        
        cpu.teclas[0x4] = window.is_key_down(Key::Q);
        cpu.teclas[0x5] = window.is_key_down(Key::W);
        cpu.teclas[0x6] = window.is_key_down(Key::E);
        cpu.teclas[0xD] = window.is_key_down(Key::R);
        
        cpu.teclas[0x7] = window.is_key_down(Key::A);
        cpu.teclas[0x8] = window.is_key_down(Key::S);
        cpu.teclas[0x9] = window.is_key_down(Key::D);
        cpu.teclas[0xE] = window.is_key_down(Key::F);
        
        cpu.teclas[0xA] = window.is_key_down(Key::Z);
        cpu.teclas[0x0] = window.is_key_down(Key::X);
        cpu.teclas[0xB] = window.is_key_down(Key::C);
        cpu.teclas[0xF] = window.is_key_down(Key::V);


        // --- 1. PROCESAMIENTO CEREBRAL (CPU) ---
        // Nuestro monitor actualizará la pantalla física a 60 FPS (Hz).
        // Pero procesadores como el CHIP-8 corrían mucho más rápido, a 500 Hertz.
        // Simulamos esta velocidad haciendo que el procesador "lea y ejecute" 10 órdenes enteras
        // por cada milisegundo de cuadro que transcurre (10 ordenes * 60 fps = 600 Hz aprox)
        for _ in 0..10 {
            cpu.tick();
        }
        
        // --- 1.5. DESCUENTO DE RELOJES DE HARDWARE ---
        // Los CHIP-8 tenían dos cristales de frecuencia mecánicos soldados que latían a 60 Hertz.
        // Como este punto de nuestro código de ventana sabemos que corre a *exactamente* 60 fps (16.6ms),
        // descontamos un Tick de ambos temporizadores de la CPU en caso de estar activos.
        cpu.decrementar_temporizadores();

        // --- 2. RENDERIZADO VISUAL (Traduciendo al Hardware) ---
        // Extraemos la información técnica y silenciosa de la matriz cpu.pantalla (0 y 1s)
        // y la convertimos en píxeles RGB (Rojos, Verdes, Azules) para el monitor de Windows.
        for y in 0..PANTALLA_ALTO {
            for x in 0..PANTALLA_ANCHO {
                let pixel = cpu.pantalla[y][x];
                
                // Conversión manual: Si hay ceros pinta negro, si hay un '1' manda color FFFFFF (Blanco 100% RGB puro)
                let color_hex = if pixel == 1 { 0xFFFFFF } else { 0x000000 };
                
                // Fórmula de aplanado de Matrices: (Fila * Anchura_Total + Columna) 
                // Esto "aplasta" nuestra tabla 2D de [32][64] en una sola cinta VHS recta de 2048 posiciones.
                buffer[y * PANTALLA_ANCHO + x] = color_hex;
            }
        }

        // --- 3. DIBUJO FINAL ---
        // Le arrojamos la lista plana de colores al sistema operativo para que queme los fotones en tu pantalla real.
        // (Este proceso frena silenciosamente el hilo principal forzando los ~16.6ms que seteamos arriba)
        window.update_with_buffer(&buffer, PANTALLA_ANCHO, PANTALLA_ALTO).unwrap();
    }
}
