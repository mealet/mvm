# ⚡ Mealet Virtual Machine
![Written In Rust](https://img.shields.io/badge/Rust-red?style=basic&label=Written+In&color=%23e02012)
![License](https://img.shields.io/badge/MIT-red?style=basic&label=License&color=%23e02012)
[![justforfunnoreally.dev badge](https://img.shields.io/badge/JustForFunNoReally-dev-e02012)](https://justforfunnoreally.dev)

**MVM** - fast virtual machine written in Rust. It provides simple assembly with fast execution. <br/>
Virtual machine process allocates virtual memory and operates it with static memory and stack size.

> ⚠️ Project is under active development. Please, wait for stable releases and detailed documentation!

## 🧭 Installation
Install the project with **cargo** package manager:
```shell
cargo install --git https://github.com/mealet/deen
```

## 👀 Example
You can find more examples in [examples](./examples) directory.

```asm
; Hello World Example

section .data
  hello:
    ascii "Hello, World!\n"

  len:
    [. - hello]

section .text
entry _start

; ===| Functions |===

print:
  ; -- syscall void write(int output, void *buffer, size_t length) --

  mov %r2, %r1 ; size_t length (length variable)

  mov %r1, %r0 ; void* buffer (ptr to message)
  add %r1, $8 ; string address offset

  mov %r0, $1 ; int output (stdout)

  mov %call, $2 ; `write` syscall
  int $syscall ; system call interrupt

  ret

exit:
  ; -- syscall void exit(int status) --

  mov %r0, $0 ; int status
  mov %call, $0 ; `exit` syscall

  int $syscall ; system call interrupt

  ret

; ===| Program Entrypoint |===

_start:
  mov %r0, hello
  mov %r1, len

  call print
  call exit
```

## ❓ How it Works
Virtual Machine is an _instruction interpreter_, which allocates certain amount of memory (can be read from program file, or set by user manually with cli).
After memory is allocated and setted up, main process inserts program right at the start and updates registers such as: instruction pointer, memory pointer, frame & stack pointer. <br/>
Each instruction is a byte opcode (defined in `vm::isa::Opcode`). VM reads source binary, skips data section (because it is used only for pointers) to text section and executes the program.
Instructions can edit registers, memory (by pointers), and whole VM state. Registers (like main memory) is a `MemoryBuffer`, indexing is calculated by the machine. <br/>
Each register is a unsigned 64-bit number slot:
- `r0, r1, ..., r8` - General Purpose
- `r9` - System Call
- `r10` - Accumulator
- `r12` - Stack Pointer
- `r13` - Frame Pointer
- `r14`- Memory Pointer (next byte after program)

Assembler is a separated compiler with pre-installed constants and registers names. It provides lexer, parser, semantical analyzer and codegen (which contains labels, constants and pointers resolver).
Assembler and VM executor are not connected by the idea, but this implementation requires each module exist because of `error` module and `opcode` enumeration (for esaier changes and better code readability). <br/>
MVM has its own binary format, assembly compiler must follow it to successfully complete task:

> 1. Metadata section (optional):
> ```
> [0,0,0,0,0,0,0,0] |> 64-bit number (memory size)
> [0,0,0,0,0,0,0,0] |> 64-bit number (stack size)
> ```
> 2. Data Section:
> ```
> 0x01 |> data section start opcode
> ...
> 0xff 0x02 |> text section start sequence
> ```
> 3. Text Section
> ```
> ... |> program bytes
> ```
>
> Merging it will give us binary file with program:
> ```
> 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0x01 ... 0xff 0x02 ...
> |-----------------------------|  |       |=======|===|> section `.text` sequence
> |-----------------------------|  |-------------------|> section `.data` sequence
> |-----------------------------|----------------------|> metadata section
> ```

## 📎 License
The project is licensed under the MIT License. <br/>
See [LICENSE](LICENSE) for more information.
