//creamos una constante global para el tamaño de la ram que siempre va a ser el mismo. Es una constante porque simula las limitaciones físicas de una placa de silicio real soldada en una consola.
const MEMORY_SIZE: usize = 4096;
//creamos una constante global para la direccion en memoria en la que siempre vamos a arrancar cuando inicializamos la cpu. Los primeros 512 bytes (de la 0x000 a la 0x1FF) estaban físicamente reservados para que la consola CHIP-8 original guardara su propio sistema operativo básico (y más adelante, las fuentes de las letras). Por eso nuestro Program Counter siempre debe nacer apuntando a la 0x200. Si arranca en la 0, ¡el procesador intentaría "jugar" con el código de su propio sistema operativo y colapsaría!
const START_ADDRESS: u16 = 0x200;

// Constantes globales para el tamaño universal de la pantalla de CHIP-8
const PANTALLA_ANCHO: usize = 64;
const PANTALLA_ALTO: usize = 32;

// Tipografía oficial de la consola de 80 bytes (Representación gráfica de los números del 0 al F)
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

//creamos una estructura publica para simular nuestro procesador de 8-bit
pub struct Cpu {
    //la ram es un array de integers de 8 bits y puede almacenar hasta 4096 combinaciones
    ram: [u8; MEMORY_SIZE],
    //creamos un array de integers de 8 bits con un tamaño maximo de 16, esto es para representar nuestros registros donde vamos a hacer las operaciones(suma, resta, mover en memoria) en ves de la ram
    v: [u8; 16],   
    // Registro especial (El dedo pintor) para guardar direcciones de memoria grandes (u16) que apuntan a dibujos o fuentes.
    i: u16,
    // Program Counter (El dedo lector), un puntero que guarda en qué dirección (u16) de la RAM está la instrucción que debemos ejecutar AHORA.
    pc: u16,
    //creamos un array para simular nuestro stack que solo puede recordar 16 direcciones de retorno
    stack: [u16; 16],
    //creamos un stack pointer para saber a que direccion tenemos que retornar cuando salgamos del stack
    sp: u8,
    // Nuestra "Placa de Video" interna: Una matriz bidimensional (Filas y Columnas) de pixeles
    // Como solo usamos 0 (negro) y 1 (blanco), usamos u8 para guardar los estados
    pub pantalla: [[u8; PANTALLA_ANCHO]; PANTALLA_ALTO],
    // El Joystick Hexadecimal (16 botones). true = presionado, false = suelto.
    pub teclas: [bool; 16],
    // Temporizadores de 60Hz. Cuando son mayores a 0, la CPU los resta hasta llegar a 0 automáticamente.
    pub delay_timer: u8,
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

        // Cargamos la fuente en el arranque del "Sistema Operativo" (Empieza en la ranura oficial 0x050)
        for f in 0..80 {
            cpu.ram[0x050 + f] = FUNTES_SISTEMA[f];
        }

        cpu
    }
    // Esta funcion es como "insertar el cartucho" en la consola. Copia el archivo ROM en la RAM empezando en 0x200
    pub fn cargar_rom(&mut self, datos: &[u8]) {
        for (i, &byte) in datos.iter().enumerate() {
            // Evaluamos que no se pase del tamaño de la RAM
            let direccion = START_ADDRESS as usize + i;
            if direccion < MEMORY_SIZE {
                self.ram[direccion] = byte;
            }
        }
    }

    // Restador de relojes mecánicos a ejecutarse siempre a 60 Hz por el bucle central de Windows
    pub fn decrementar_temporizadores(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // Acá es donde la placa madre real le mandaría el alto voltaje al zumbador (Buzzer/Speaker)
                // println!("*BEEP*"); 
            }
            self.sound_timer -= 1;
        }
    }

    pub fn tick(&mut self) {
        //creamos dos variable de 16 bits, cada una contiene la mitad de una instruccion que almacenamos previamente en la ram y cada una de estas la convertimos en al tipo u16 para su posterior union
        let byte_alto = self.ram[self.pc as usize] as u16; 
        let byte_bajo = self.ram[(self.pc + 1) as usize] as u16;
        //Unimos las dos partes de la instruccion utilizando operadores de bits
        let opcode = (byte_alto << 8) | byte_bajo;

        // Comentamos la bitácora de ejecución para que corra ligero a 600Hz
        // println!("El procesador acaba de leer la orden: {:04X}", opcode);

        self.pc += 2;

        let primer_digito = (opcode & 0xF000) >> 12;

        match primer_digito {
            0x0 => {
                // Hay dos operaciones que empiezan con 0x0
                if opcode == 0x00E0 {
                    // Opcode 00E0: Clear Screen
                    self.pantalla = [[0; PANTALLA_ANCHO]; PANTALLA_ALTO];
                } else if opcode == 0x00EE {
                    // Opcode 00EE: Return (Retornar de una subrutina)
                    // Bajamos en el ascensor (disminuimos el puntero de la pila)
                    self.sp -= 1;
                    // Le preguntamos a la pila en qué dirección de memoria nos quedamos la última vez
                    self.pc = self.stack[self.sp as usize];
                } else {
                    // println!("  ↳ [IGNORADO] Código asociado a mainframes viejas: {:04X}", opcode);
                }
            },

            0x1 => {
                let direccion = opcode & 0x0FFF; 
                
                self.pc = direccion; 
            },

            0x3 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                // Opcode 3XNN: Skip next instruction if VX == NN
                // Es literalmente un "if (v[x] == nn)". Si es cierto, nos salteamos la siguiente línea de código sumándole 2 extra al avance del Program Counter.
                if self.v[x] == nn {
                    self.pc += 2;
                }
            },

            0x4 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                // Opcode 4XNN: Skip next instruction if VX != NN
                // Este es el "if (v[x] != nn)".
                if self.v[x] != nn {
                    self.pc += 2;
                }
            },

            0x5 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                // Opcode 5XY0: Skip next instruction if VX == VY
                // Acá comparamos dos cajas de registros entre ellas ("if v[x] == v[y]")
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            },

            0x2 => {
                // Opcode 2NNN: Call Subroutine (Llamar a Función)
                // Primero guardamos nuestra posición de memoria actual en el estante de la Pila...
                self.stack[self.sp as usize] = self.pc;
                // Subimos en el ascensor, indicando que hay una página de memoria nueva guardada
                self.sp += 1;
                // Finalmente, saltamos hacia la nueva función usando el salto incondicional común
                let direccion = opcode & 0x0FFF;
                self.pc = direccion;
            },

            0x6 => {
                // Buscamos cuál es el registro X aislando el segundo dígito
                let x = ((opcode & 0x0F00) >> 8) as usize;
                // Buscamos el valor NN aislando los dos últimos dígitos
                let nn = (opcode & 0x00FF) as u8;
                
                // println!("  ↳ [EJECUTANDO] Guardando el valor {} en el registro V{:X}...", nn, x);
                self.v[x] = nn;
            },

            0x7 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                
                // Usamos wrapping_add porque si el registro llega a 255 y le sumamos 1, 
                // en CHIP-8 debe volver a 0 sin romper el emulador
                self.v[x] = self.v[x].wrapping_add(nn);
            },

            0x8 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                let operacion = opcode & 0x000F; // El último dígito define la matemática a usar

                match operacion {
                    0x0 => self.v[x] = self.v[y], // Set directo
                    0x1 => self.v[x] |= self.v[y], // OR lógico (Bits)
                    0x2 => self.v[x] &= self.v[y], // AND lógico (Bits)
                    0x3 => self.v[x] ^= self.v[y], // XOR lógico (Bits)
                    
                    0x4 => {
                        // Suma con Acarreo (Overflow). 
                        // Rust tiene funciones puras nativas (`overflowing_add`) que nos devuelven
                        // tanto el resultado truncado como un booleano avisando si explotó el límite de u8 (255)
                        let (suma, se_paso) = self.v[x].overflowing_add(self.v[y]);
                        self.v[x] = suma;
                        // Regla CHIP-8: Si se pasó de 255, VF = 1. Si no, VF = 0.
                        self.v[0xF] = if se_paso { 1 } else { 0 }; 
                    },
                    
                    0x5 => {
                        // Resta con "Préstamo" (Borrow/Underflow). Ocurre cuando restamos hasta dejarlo en negativo.
                        let (resta, cayo_abajo_de_cero) = self.v[x].overflowing_sub(self.v[y]);
                        self.v[x] = resta;
                        // Regla de CHIP-8 "al revés": Si NO cayó por debajo de cero (si x >= y), VF = 1.
                        self.v[0xF] = if cayo_abajo_de_cero { 0 } else { 1 };
                    },
                    
                    0x6 => {
                        // Shift Right (Desplazar los bits hacia la derecha, dividiendo por 2 básicamente)
                        // Atrapamos el bit que se "cayó" por el borde derecho y lo guardamos en VF
                        self.v[0xF] = self.v[x] & 1;
                        self.v[x] >>= 1;
                    },
                    
                    0x7 => {
                        // Resta Inversa (VY - VX en vez de VX - VY)
                        let (resta, cayo_abajo_de_cero) = self.v[y].overflowing_sub(self.v[x]);
                        self.v[x] = resta;
                        self.v[0xF] = if cayo_abajo_de_cero { 0 } else { 1 };
                    },
                    
                    0xE => {
                        // Shift Left (Desplazar los bits hacia la izquierda, multiplicando por 2)
                        // Atrapamos el bit supremo que se "cayó" por la izquierda y va a VF
                        self.v[0xF] = (self.v[x] >> 7) & 1;
                        self.v[x] <<= 1;
                    },
                    
                    _ => {
                        // Instrucción no implementada o error en la ROM
                        // println!("ALU Error: Matemática desconocida {:04X}", opcode);
                    }
                }
            },

            0x9 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                // Opcode 9XY0: Skip next instruction if VX != VY
                // Igual que la familia 5, pero negando ("if v[x] != v[y]")
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            },
            
            0xA => {
                let direccion = opcode & 0x0FFF;
                // println!("  ↳ [EJECUTANDO] Guardando la dirección {:03X} en el dedo pintor 'I'...", direccion);
                
                self.i = direccion;
            },

            0xC => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                let numero_random = rand::random::<u8>();
                // CXNN: Crea un numero aleatorio y lo "enmascara" con el parámetro NN usando AND Lógico.
                self.v[x] = numero_random & nn;
            },

            0xD => {
                // aislar registros
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                // 'N' indica cuántas filas de alto tiene el sprite (cada fila es de 1 byte/8 pixeles)
                let altura = (opcode & 0x000F) as u16; 
                
                // Las coordenadas de origen base se leen de los registros V correspondientes
                // Si la coordenada excede el ancho/alto, envuelve la pantalla con modulo (%)
                let coord_x = self.v[x] as usize % PANTALLA_ANCHO;
                let coord_y = self.v[y] as usize % PANTALLA_ALTO;
                
                // println!("  ↳ [EJECUTANDO] DXYN: Dibujando bloque en X: {}, Y: {}, Alto: {}", coord_x, coord_y, altura);

                // Inicializamos el registro de colision (VF) en 0 antes de dibujar
                self.v[0xF] = 0;

                // Leemos las N filas del sprite desde la RAM (empezando en la dirección que marca el dedo 'I')
                for fila in 0..altura {
                    let byte_sprite = self.ram[(self.i + fila) as usize];

                    // Cada fila tiene exactamente 8 bits (1 byte = 8 pixeles de ancho fijo)
                    for columna in 0..8 {
                        // Extraemos el valor del pixel individual (0 o 1) empujando los bits y pasando una mascara
                        let pixel = (byte_sprite >> (7 - columna)) & 1;

                        if pixel == 1 {
                            // Calculamos en que celda de la matriz va este pixel especifico
                            let pixel_x = coord_x + columna;
                            let pixel_y = coord_y + fila as usize;

                            // En CHIP-8 clasico, los pixeles se "cortan" si tocan el borde (clipping)
                            if pixel_x < PANTALLA_ANCHO && pixel_y < PANTALLA_ALTO {
                                // Si el pixel de la pantalla ya estaba prendido (1), registrar colisión
                                if self.pantalla[pixel_y][pixel_x] == 1 {
                                    self.v[0xF] = 1;
                                }

                                // El operador XOR (^=) invierte el estado de la pantalla.
                                // Si estaba en 0 pasa a 1 (lo dibuja). Si estaba en 1 pasa a 0 (lo borra).
                                self.pantalla[pixel_y][pixel_x] ^= 1;
                            }
                        }
                    }
                }
            },

            0xE => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let operacion_teclado = opcode & 0x00FF; // Aislamos los últimos dos dígitos

                match operacion_teclado {
                    0x9E => {
                        // EX9E: Skip if Key Pressed
                        // Va al Array Fisico booleano y si dice 'true', salta un renglón
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
                    _ => {
                        // println!("Error Teclado NO implementado: {:04X}", opcode);
                    }
                }
            },

            0xF => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let final_opcode = opcode & 0x00FF;

                match final_opcode {
                    0x07 => {
                        // FX07: Leer el Delay Timer y guardarlo en un registro
                        self.v[x] = self.delay_timer;
                    },
                    0x0A => {
                        // FX0A: Congelar la CPU hasta recibir una pulsación de tecla y guardarla en VX
                        let mut se_apreto = false;
                        for btn in 0..16 {
                            if self.teclas[btn] {
                                self.v[x] = btn as u8;
                                se_apreto = true;
                                break;
                            }
                        }
                        // Truco Maestro de Emuladores: Si no hay tecla apretada, anulamos el avance natural del lector de memoria
                        // restándole 2 al Program Counter, obligando así al juego a colgarse en un Bucle Infinito leyendo
                        // esta misma línea hasta que alguien accione el control.
                        if !se_apreto {
                            self.pc -= 2;
                        }
                    },
                    0x15 => {
                        // FX15: Setear/Pisarlo al Delay Timer con el número de un registro
                        self.delay_timer = self.v[x];
                    },
                    0x18 => {
                        // FX18: Setear el temporizador de Sonido
                        self.sound_timer = self.v[x];
                    },
                    0x1E => {
                        // FX1E: Sumarle VX al dedo pintor 'I'
                        self.i += self.v[x] as u16;
                    },
                    0x29 => {
                        // FX29: Apuntar el dedo 'I' al lugar de la RAM donde está guardado el dibujo de la letra que queremos
                        // Como instalamos la fuente estática en 0x050 y cada letra mide 5 bytes, la fórmula es simplemente multiplicar por 5.
                        self.i = 0x050 + (self.v[x] as u16 * 5);
                    },
                    0x33 => {
                        // FX33: Separación Digital (Convertir un número Hexadecimal a BCD Decimal Puro: Centenas, Decenas, Unidades)
                        // Sirve para que el juego sepa cómo escribirte el Puntuaje "254" en la pantalla de la TV.
                        // Y lo guarda físicamente en la memoria RAM en las posiciones de las coordenadas I, I+1 e I+2
                        let valor = self.v[x];
                        let centenas = valor / 100;
                        let decenas = (valor / 10) % 10;
                        let unidades = valor % 10;
                        
                        self.ram[self.i as usize] = centenas;
                        self.ram[(self.i + 1) as usize] = decenas;
                        self.ram[(self.i + 2) as usize] = unidades;
                    },
                    0x55 => {
                        // FX55: Guardado Masivo (Dump) de la vida del procesador a la RAM
                        // Permítimos que el juego escupa de un golpe el estado de múltiples registros a la vez en la placa madre
                        for iteracion in 0..=x {
                            self.ram[self.i as usize + iteracion] = self.v[iteracion];
                        }
                    },
                    0x65 => {
                        // FX65: Carga Masiva (Load) leyendo desde la Memoria RAM y escribiendo adentro el procesador
                        // Ideal para los puntos de control o "Cargar Nivel" rápido de Save States.
                        for iteracion in 0..=x {
                            self.v[iteracion] = self.ram[self.i as usize + iteracion];
                        }
                    },
                    _ => {
                        // println!("Error Opcode de Memoria F Desconocido: {:04X}", opcode);
                    }
                }
            },

            _ => {
                // println!("  ↳ [ERROR O NO IMPLEMENTADO] Instrucción desconocida: {:04X}", opcode);
            }
        }
    }
}
