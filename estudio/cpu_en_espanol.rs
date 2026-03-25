// ============================================================
// COPIA DE ESTUDIO - VERSION EN ESPAÑOL
// Este archivo NO es parte del proyecto compilable.
// Es tu copia personal con todos los comentarios en español
// para estudiar la arquitectura del emulador.
// ============================================================

//creamos una constante global para el tamaño de la ram que siempre va a ser el mismo. Es una constante porque simula las limitaciones físicas de una placa de silicio real soldada en una consola.
const MEMORY_SIZE: usize = 4096;
//creamos una constante global para la direccion en memoria en la que siempre vamos a arrancar cuando inicializamos la cpu. Los primeros 512 bytes (de la 0x000 a la 0x1FF) estaban físicamente reservados para que la consola CHIP-8 original guardara su propio sistema operativo básico (y más adelante, las fuentes de las letras). Por eso nuestro Program Counter siempre debe nacer apuntando a la 0x200. Si arranca en la 0, ¡el procesador intentaría "jugar" con el código de su propio sistema operativo y colapsaría!
const START_ADDRESS: u16 = 0x200;

// Constantes globales para el tamaño universal de la pantalla de CHIP-8
const PANTALLA_ANCHO: usize = 64;
const PANTALLA_ALTO: usize = 32;

// Tipografía oficial de la consola de 80 bytes (Representación gráfica de los números del 0 al F)
// Cada número ocupa 5 bytes (5 filas de 8 pixels). Los bits en 1 son pixels blancos.
const FUNTES_SISTEMA: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

// Creamos una estructura publica para simular nuestro procesador de 8-bit
pub struct Cpu {
    // La RAM: un array de integers de 8 bits que puede almacenar hasta 4096 posiciones
    ram: [u8; MEMORY_SIZE],
    // Registros de propósito general V0 al VF (16 cajitas de 8 bits para operaciones aritméticas)
    v: [u8; 16],
    // Registro especial I (El "dedo pintor"): guarda direcciones de memoria grandes (u16) que apuntan a dibujos o fuentes
    i: u16,
    // Program Counter (El "dedo lector"): puntero que marca en qué dirección de la RAM está la siguiente instrucción
    pc: u16,
    // Stack: array de 16 posiciones para recordar direcciones de retorno al hacer saltos a funciones
    stack: [u16; 16],
    // Stack Pointer: índice que apunta a la cima de la pila (el próximo post-it libre)
    sp: u8,
    // VRAM (Video RAM): matriz bidimensional que representa los pixels de la pantalla
    // 0 = pixel negro (apagado), 1 = pixel blanco (prendido)
    pub pantalla: [[u8; PANTALLA_ANCHO]; PANTALLA_ALTO],
    // Joystick Hexadecimal: array de 16 booleanos (true = presionado, false = suelto)
    // Mapeado a las teclas QWERTY físicas del teclado real en main.rs
    pub teclas: [bool; 16],
    // Temporizador general: decrece automáticamente a 60 Hz. Usado para efectos, cooldowns, etc.
    pub delay_timer: u8,
    // Temporizador de sonido: mientras sea > 0, el buzzer físico suena
    pub sound_timer: u8,
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Self {
            ram: [0; MEMORY_SIZE],
            v: [0; 16],
            i: 0,
            pc: START_ADDRESS,
            stack: [0; 16],
            sp: 0,
            pantalla: [[0; PANTALLA_ANCHO]; PANTALLA_ALTO],
            teclas: [false; 16],
            delay_timer: 0,
            sound_timer: 0,
        };

        // Cargamos las fuentes del sistema en la RAM al arrancar (como el BIOS de una PC)
        // La convención oficial del CHIP-8 ubica la fuente en 0x050
        for f in 0..80 {
            cpu.ram[0x050 + f] = FUNTES_SISTEMA[f];
        }

        cpu
    }

    // Cargar_rom: "Insertar el cartucho". Copia los bytes de la ROM en la RAM empezando en 0x200
    pub fn cargar_rom(&mut self, datos: &[u8]) {
        for (i, &byte) in datos.iter().enumerate() {
            let direccion = START_ADDRESS as usize + i;
            if direccion < MEMORY_SIZE {
                self.ram[direccion] = byte;
            }
        }
    }

    // Decrementar_temporizadores: debe ejecutarse exactamente a 60 Hz desde el Game Loop
    // Simula los cristales de cuarzo osciladores soldados en la placa madre original
    pub fn decrementar_temporizadores(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // Aquí iría la señal al buzzer físico. Por ahora solo un comentario.
                // println!("*BEEP*");
            }
            self.sound_timer -= 1;
        }
    }

    // Tick: Un ciclo completo Fetch-Decode-Execute del procesador
    pub fn tick(&mut self) {
        // FETCH: Leer los 2 bytes de la instrucción actual de la RAM
        let byte_alto = self.ram[self.pc as usize] as u16;
        let byte_bajo = self.ram[(self.pc + 1) as usize] as u16;

        // Unimos los dos bytes en un solo Opcode de 16 bits usando operadores de bits
        let opcode = (byte_alto << 8) | byte_bajo;

        // Avanzar el Program Counter a la próxima instrucción (cada instrucción ocupa 2 bytes)
        self.pc += 2;

        // DECODE: Aislamos el primer dígito hexadecimal para identificar la familia de instrucción
        let primer_digito = (opcode & 0xF000) >> 12;

        // EXECUTE: Ejecutamos la instrucción correspondiente
        match primer_digito {
            0x0 => {
                // Familia 0: Instrucciones del sistema
                if opcode == 0x00E0 {
                    // 00E0: Limpiar pantalla. Llena toda la VRAM con ceros (negro)
                    self.pantalla = [[0; PANTALLA_ANCHO]; PANTALLA_ALTO];
                } else if opcode == 0x00EE {
                    // 00EE: Retornar de subrutina.
                    // Bajamos en el "ascensor" (Stack Pointer) y recuperamos la dirección de retorno
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                }
            },

            0x1 => {
                // 1NNN: Jump. Salto incondicional a la dirección NNN
                let direccion = opcode & 0x0FFF;
                self.pc = direccion;
            },

            0x2 => {
                // 2NNN: Call Subroutine. Llamar a una función.
                // Guardamos la posición actual en la pila antes de saltar
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                let direccion = opcode & 0x0FFF;
                self.pc = direccion;
            },

            0x3 => {
                // 3XNN: Skip if VX == NN (if/else mediante salto de renglón)
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                if self.v[x] == nn {
                    self.pc += 2; // Saltamos la siguiente instrucción
                }
            },

            0x4 => {
                // 4XNN: Skip if VX != NN
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                if self.v[x] != nn {
                    self.pc += 2;
                }
            },

            0x5 => {
                // 5XY0: Skip if VX == VY (comparación entre dos registros)
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            },

            0x6 => {
                // 6XNN: Set VX = NN. Cargar un valor directo en un registro
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                self.v[x] = nn;
            },

            0x7 => {
                // 7XNN: Add VX += NN. Suma inmediata al registro.
                // wrapping_add: si pasa de 255 vuelve a 0 sin romper el programa
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                self.v[x] = self.v[x].wrapping_add(nn);
            },

            0x8 => {
                // Familia 8: ALU (Unidad Aritmético-Lógica). El último dígito define la operación.
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                let operacion = opcode & 0x000F;

                match operacion {
                    0x0 => self.v[x] = self.v[y],          // Set: VX = VY
                    0x1 => self.v[x] |= self.v[y],          // OR lógico a nivel de bits
                    0x2 => self.v[x] &= self.v[y],          // AND lógico a nivel de bits
                    0x3 => self.v[x] ^= self.v[y],          // XOR lógico a nivel de bits

                    0x4 => {
                        // Suma con Carry Flag (Acarreo)
                        // overflowing_add devuelve el resultado truncado + booleano de overflow
                        let (suma, se_paso) = self.v[x].overflowing_add(self.v[y]);
                        self.v[x] = suma;
                        self.v[0xF] = if se_paso { 1 } else { 0 };
                    },

                    0x5 => {
                        // Resta con Borrow Flag (Préstamo / Underflow)
                        // Regla CHIP-8 "al revés": VF=1 si NO hubo underflow, VF=0 si sí hubo
                        let (resta, cayo_abajo_de_cero) = self.v[x].overflowing_sub(self.v[y]);
                        self.v[x] = resta;
                        self.v[0xF] = if cayo_abajo_de_cero { 0 } else { 1 };
                    },

                    0x6 => {
                        // Shift Right: desplaza bits a la derecha (divide por 2)
                        // VF guarda el bit que se "cayó" por el borde derecho
                        self.v[0xF] = self.v[x] & 1;
                        self.v[x] >>= 1;
                    },

                    0x7 => {
                        // Resta Inversa: VX = VY - VX
                        let (resta, cayo_abajo_de_cero) = self.v[y].overflowing_sub(self.v[x]);
                        self.v[x] = resta;
                        self.v[0xF] = if cayo_abajo_de_cero { 0 } else { 1 };
                    },

                    0xE => {
                        // Shift Left: desplaza bits a la izquierda (multiplica por 2)
                        // VF guarda el bit más significativo que se "cayó" por la izquierda
                        self.v[0xF] = (self.v[x] >> 7) & 1;
                        self.v[x] <<= 1;
                    },

                    _ => {} // Instrucción desconocida, ignorar
                }
            },

            0x9 => {
                // 9XY0: Skip if VX != VY (inverso de 5XY0)
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            },

            0xA => {
                // ANNN: Set I = NNN. Apuntar el "dedo pintor" a una dirección de memoria
                let direccion = opcode & 0x0FFF;
                self.i = direccion;
            },

            0xC => {
                // CXNN: Random. Genera un byte aleatorio y lo enmascara con NN usando AND
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                let numero_random = rand::random::<u8>();
                self.v[x] = numero_random & nn;
            },

            0xD => {
                // DXYN: Draw Sprite. El corazón gráfico del emulador.
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                // N = cantidad de filas del sprite (altura en pixels)
                let altura = (opcode & 0x000F) as u16;

                // Coordenadas de origen. Módulo para que envuelva la pantalla si se pasa del borde
                let coord_x = self.v[x] as usize % PANTALLA_ANCHO;
                let coord_y = self.v[y] as usize % PANTALLA_ALTO;

                // Reset del flag de colisión antes de dibujar
                self.v[0xF] = 0;

                // Para cada fila del sprite (empezando en self.i de la RAM)
                for fila in 0..altura {
                    let byte_sprite = self.ram[(self.i + fila) as usize];

                    // Para cada bit (pixel) del byte (8 pixels de ancho fijo)
                    for columna in 0..8 {
                        // Extraemos el bit individual usando máscara y desplazamiento
                        let pixel = (byte_sprite >> (7 - columna)) & 1;

                        if pixel == 1 {
                            let pixel_x = coord_x + columna;
                            let pixel_y = coord_y + fila as usize;

                            // Clipping: no dibujar fuera de los límites de la pantalla
                            if pixel_x < PANTALLA_ANCHO && pixel_y < PANTALLA_ALTO {
                                // Si el pixel ya estaba prendido, registramos colisión en VF
                                if self.pantalla[pixel_y][pixel_x] == 1 {
                                    self.v[0xF] = 1;
                                }
                                // XOR: invierte el estado del pixel (así es como funciona el CHIP-8 original)
                                self.pantalla[pixel_y][pixel_x] ^= 1;
                            }
                        }
                    }
                }
            },

            0xE => {
                // Familia E: Instrucciones de teclado
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let operacion_teclado = opcode & 0x00FF;

                match operacion_teclado {
                    0x9E => {
                        // EX9E: Skip if Key Pressed. Si la tecla cuyo código está en VX está presionada, saltar
                        if self.teclas[self.v[x] as usize] {
                            self.pc += 2;
                        }
                    },
                    0xA1 => {
                        // EXA1: Skip if Key NOT Pressed
                        if !self.teclas[self.v[x] as usize] {
                            self.pc += 2;
                        }
                    },
                    _ => {}
                }
            },

            0xF => {
                // Familia F: Instrucciones del sistema (timers, fuentes, memoria)
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let final_opcode = opcode & 0x00FF;

                match final_opcode {
                    0x07 => {
                        // FX07: VX = delay_timer. Leer el reloj y guardarlo en un registro
                        self.v[x] = self.delay_timer;
                    },
                    0x0A => {
                        // FX0A: Wait for Key. Congelar la CPU hasta que se presione cualquier tecla
                        // Truco: retrocedeemos el PC para que la CPU repita esta instrucción en loop
                        let mut se_apreto = false;
                        for btn in 0..16 {
                            if self.teclas[btn] {
                                self.v[x] = btn as u8;
                                se_apreto = true;
                                break;
                            }
                        }
                        if !se_apreto {
                            self.pc -= 2; // Rewind: volver a leer esta misma instrucción
                        }
                    },
                    0x15 => {
                        // FX15: delay_timer = VX. Arrancar el reloj de cuenta regresiva
                        self.delay_timer = self.v[x];
                    },
                    0x18 => {
                        // FX18: sound_timer = VX. Arrancar el buzzer
                        self.sound_timer = self.v[x];
                    },
                    0x1E => {
                        // FX1E: I += VX. Avanzar el dedo pintor
                        self.i += self.v[x] as u16;
                    },
                    0x29 => {
                        // FX29: I = Font address of VX. Apuntar I al dibujo del número VX
                        // La fuente vive en 0x050. Cada carácter mide 5 bytes, entonces multiplicamos.
                        self.i = 0x050 + (self.v[x] as u16 * 5);
                    },
                    0x33 => {
                        // FX33: BCD (Binary Coded Decimal). Separar un número en centenas, decenas, unidades
                        // Para que el juego pueda dibujar puntajes como "254" en pantalla
                        let valor = self.v[x];
                        let centenas = valor / 100;
                        let decenas = (valor / 10) % 10;
                        let unidades = valor % 10;
                        self.ram[self.i as usize] = centenas;
                        self.ram[(self.i + 1) as usize] = decenas;
                        self.ram[(self.i + 2) as usize] = unidades;
                    },
                    0x55 => {
                        // FX55: Store registers. Volcar V0..VX en la RAM (Save State)
                        for iteracion in 0..=x {
                            self.ram[self.i as usize + iteracion] = self.v[iteracion];
                        }
                    },
                    0x65 => {
                        // FX65: Load registers. Cargar V0..VX desde la RAM (Load State)
                        for iteracion in 0..=x {
                            self.v[iteracion] = self.ram[self.i as usize + iteracion];
                        }
                    },
                    _ => {}
                }
            },

            _ => {
                // Instrucción desconocida o no implementada - ignorar silenciosamente
            }
        }
    }
}
