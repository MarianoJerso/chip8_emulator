# CHIP-8 — Referencia Completa de Instrucciones

## Manejo de Bits (ALU — Opcode 8)

Estas son las más útiles para Bitcoin Core, criptografía y cualquier trabajo de bajo nivel.

| Opcode | Nombre | Operación | Uso típico |
|--------|--------|-----------|------------|
| `8XY1` | OR | `VX \|= VY` | Activar bits específicos (encender flags) |
| `8XY2` | AND | `VX &= VY` | Aislar bits con una máscara (lectura de flags) |
| `8XY3` | XOR | `VX ^= VY` | Invertir bits / toggles / cifrado simple |
| `8XY6` | SHR | `VX >>= 1` | Dividir por 2, extraer el bit menos significativo (LSB) |
| `8XYE` | SHL | `VX <<= 1` | Multiplicar por 2, extraer el bit más significativo (MSB) |

### Cómo se usan en la práctica (aplicable a Bitcoin Core)

```
// Activar un flag sin tocar los demás (OR)
estado |= 0b00000100   // Enciende el bit 2 sin afectar los otros 7

// Leer si un flag está activo (AND como máscara)
resultado = valor & 0b00001111  // Aísla los 4 bits bajos (un nibble del byte)

// Invertir bits / XOR (base del cifrado y checksums de Bitcoin)
hash_a ^ hash_b  // Mezcla dos hashes — técnica real usada en Merkle Trees

// Extraer el LSB (bit más bajo) — usado en serialización de datos
bit_0 = valor & 1  // 1 si el número es impar, 0 si es par

// Desplazamiento para construir máscaras y leer campos de bits
byte >> 4     // Extrae el nibble alto (los 4 bits más significativos)
byte & 0x0F   // Extrae el nibble bajo (los 4 bits menos significativos)
```

---

## Aritmética (ALU — Opcode 8)

| Opcode | Nombre | Operación | VF (flag) |
|--------|--------|-----------|-----------|
| `8XY4` | ADD | `VX += VY` | `1` si el resultado supera 255 (overflow) |
| `8XY5` | SUB | `VX -= VY` | `1` si NO hubo underflow (VX >= VY) |
| `8XY7` | SUBN | `VX = VY - VX` | `1` si NO hubo underflow (VY >= VX) |
| `8XY0` | LD | `VX = VY` | Sin efecto |

### El Carry Flag / Borrow Flag

El registro `VF` actúa como el **Carry Flag** de la CPU real. Es idéntico al concepto usado en el procesador 6502 de la NES/Apple II y en la arquitectura x86 de tu computadora:

```
// Suma con overflow (como suma de enteros grandes en Bitcoin)
(255 + 10) = 265 → truncado a 9, VF = 1 (hubo carry)

// Resta con borrow (como comparar alturas de bloques en una blockchain)
(5 - 10) = -5 → truncado a 251 (underflow), VF = 0 (hubo borrow)
```

---

## Instrucciones Condicionales (Skip Instructions)

| Opcode | Condición | Equivalente en Rust |
|--------|-----------|---------------------|
| `3XNN` | Skip if `VX == NN` | `if v[x] == nn` |
| `4XNN` | Skip if `VX != NN` | `if v[x] != nn` |
| `5XY0` | Skip if `VX == VY` | `if v[x] == v[y]` |
| `9XY0` | Skip if `VX != VY` | `if v[x] != v[y]` |
| `EX9E` | Skip if `key[VX]` presionada | `if keypad[v[x]]` |
| `EXA1` | Skip if `key[VX]` NO presionada | `if !keypad[v[x]]` |

---

## Control de Flujo

| Opcode | Nombre | Descripción |
|--------|--------|-------------|
| `1NNN` | JP | Salto incondicional a la dirección NNN |
| `2NNN` | CALL | Llamar a subrutina (guarda retorno en el Stack) |
| `00EE` | RET | Retornar de subrutina (recupera dirección del Stack) |

---

## Registros y Memoria

| Opcode | Nombre | Descripción |
|--------|--------|-------------|
| `6XNN` | LD Vx | Cargar valor inmediato NN en el registro VX |
| `7XNN` | ADD Vx | Sumar NN a VX sin modificar VF |
| `ANNN` | LD I | Apuntar el Index Register (I) a la dirección NNN |
| `FX1E` | ADD I | Avanzar I sumándole VX |
| `FX55` | STORE | Volcar registros V0..VX en la RAM desde I |
| `FX65` | LOAD | Cargar en V0..VX desde la RAM en I |

---

## Sistema y Periféricos

| Opcode | Nombre | Descripción |
|--------|--------|-------------|
| `00E0` | CLS | Limpiar la pantalla (VRAM = 0) |
| `DXYN` | DRW | Dibujar sprite de N filas en (VX, VY) con XOR |
| `CXNN` | RND | `VX = random_byte & NN` |
| `FX07` | LD Vx, DT | Leer el delay timer en VX |
| `FX15` | LD DT, Vx | Setear el delay timer con VX |
| `FX18` | LD ST, Vx | Setear el sound timer con VX |
| `FX0A` | WAIT | Bloquear la CPU hasta que se presione una tecla |
| `FX29` | LDF | Apuntar I al sprite de la fuente del dígito VX |
| `FX33` | BCD | Separar VX en centenas/decenas/unidades en RAM |
