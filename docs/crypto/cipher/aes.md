<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Advanced Encryption Standard (AES-128 / AES-256)

The cryptographic subsystem (`crates/crypto/src/cipher/aes/`) implements standard AES symmetric block encryption conforming to FIPS 197.

---

## 1. AES Round Transformations

Each 128-bit block (16 bytes) is arranged as a $4 \times 4$ column-major matrix state. An encryption cycle executes 10 rounds (for AES-128) consisting of:

1. **SubBytes**: Non-linear byte substitution using the cryptographic S-Box table.
2. **ShiftRows**: Cyclically shifts rows by offsets $0, 1, 2, 3$:
   $$\begin{bmatrix} s_{0,0} & s_{0,1} & s_{0,2} & s_{0,3} \\ s_{1,0} & s_{1,1} & s_{1,2} & s_{1,3} \\ s_{2,0} & s_{2,1} & s_{2,2} & s_{2,3} \\ s_{3,0} & s_{3,1} & s_{3,2} & s_{3,3} \end{bmatrix} \to \begin{bmatrix} s_{0,0} & s_{0,1} & s_{0,2} & s_{0,3} \\ s_{1,1} & s_{1,2} & s_{1,3} & s_{1,0} \\ s_{2,2} & s_{2,3} & s_{2,0} & s_{2,1} \\ s_{3,3} & s_{3,0} & s_{3,1} & s_{3,2} \end{bmatrix}$$
3. **MixColumns**: Multiplies columns by a fixed polynomial in the Galois field $GF(2^8)$. (Omitted in the final round).
4. **AddRoundKey**: Bitwise XOR with the expanded round subkey.

---

## 2. Key Expansion (Rijndael Schedule)

Takes the 128-bit master key and produces 11 round keys (44 32-bit words) using round constant array (`Rcon`) and S-Box rotation.
