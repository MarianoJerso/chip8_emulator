//creamos una constante global para el tamaño de la ram que siempre va a ser el mismo. Es una constante porque simula las limitaciones físicas de una placa de silicio real soldada en una consola.
const MEMORY_SIZE: usize = 4096;
//creamos una constante global para la direccion en memoria en la que siempre vamos a arrancar cuando inicializamos la cpu. Los primeros 512 bytes (de la 0x000 a la 0x1FF) estaban físicamente reservados para que la consola CHIP-8 original guardara su propio sistema operativo básico (y más adelante, las fuentes de las letras). Por eso nuestro Program Counter siempre debe nacer apuntando a la 0x200. Si arranca en la 0, ¡el procesador intentaría "jugar" con el código de su propio sistema operativo y colapsaría!
const START_ADDRESS: u16 = 0x200;

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
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            ram: [0; MEMORY_SIZE],
            v: [0; 16],
            i: 0,
            pc: START_ADDRESS,
            stack: [0; 16],
            sp: 0,
        }
    }
    //guardamos en la primera y segunda posicion de la ram dos datos o bytes en especifico,Como el tamaño de una instrucción en CHIP-8 es de 2 bytes (16 bits) pero nuestra memoria RAM solo sabe guardar 1 byte (8 bits) en cada ranura, la forma de meter una instrucción entera a la fuerza es partirla a la mitad y meterla en dos índices consecutivos, empezando en la dirección en la que inicializamos nuestra CPU 
    pub fn cargar_dummy_rom(&mut self, byte1: u8, byte2: u8) {
        self.ram[START_ADDRESS as usize] = byte1;
        self.ram[(START_ADDRESS + 1) as usize] = byte2;
    }
    //guardamos las mitades de una instruccion en cualquier posicion de la ram
    pub fn cargar_dummy_rom_extra(&mut self, byte1: u8, byte2: u8, offset: usize) {
        self.ram[offset] = byte1;
        self.ram[offset + 1] = byte2;
    }

    pub fn tick(&mut self) {
        //creamos dos variable de 16 bits, cada una contiene la mitad de una instruccion que almacenamos previamente en la ram y cada una de estas la convertimos en al tipo u16 para su posterior union
        let byte_alto = self.ram[self.pc as usize] as u16; 
        let byte_bajo = self.ram[(self.pc + 1) as usize] as u16;
        //Unimos las dos partes de la instruccion utilizando operadores de bits, la variable que contiene la primera parte de la instruccion la movemos 8 posiciones a la izquierda para que ocupe los primeros 8 bits de la instruccion y usamos el operador de inclusion para fusionar la segunda parte de la instruccion con la primera. Asi creamos una variable que contiene la instruccion de 16bit original.
        let opcode = (byte_alto << 8) | byte_bajo;

        println!("El procesador acaba de leer la orden: {:04X}", opcode);

        self.pc += 2;

        let primer_digito = (opcode & 0xF000) >> 12;

        match primer_digito {
            0x1 => {
                let direccion = opcode & 0x0FFF; 
                println!("  ↳ [EJECUTANDO] Saltando (Jump) a la dirección de memoria {:03X}...", direccion);
                
                self.pc = direccion; 
            },

            0x6 => {
                // Buscamos cuál es el registro X aislando el segundo dígito
                let x = ((opcode & 0x0F00) >> 8) as usize;
                // Buscamos el valor NN aislando los dos últimos dígitos
                let nn = (opcode & 0x00FF) as u8;
                
                println!("  ↳ [EJECUTANDO] Guardando el valor {} en el registro V{:X}...", nn, x);
                self.v[x] = nn;
            },

            0x7 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                
                println!("  ↳ [EJECUTANDO] Sumando {} al registro V{:X}...", nn, x);
                // Usamos wrapping_add porque si el registro llega a 255 y le sumamos 1, 
                // en CHIP-8 debe volver a 0 sin romper el emulador
                self.v[x] = self.v[x].wrapping_add(nn);
            },
            
            0xA => {
                let direccion = opcode & 0x0FFF;
                println!("  ↳ [EJECUTANDO] Guardando la dirección {:03X} en el dedo pintor 'I'...", direccion);
                
                self.i = direccion;
            },

            _ => {
                println!("  ↳ [ERROR O NO IMPLEMENTADO] Instrucción desconocida: {:04X}", opcode);
            }
        }
    }
}
