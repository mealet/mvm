; MVM Allocator Example

section .data
section .text
entry _start

; ===| Functions |===

fn_exit:
  mov %call, $sys_exit
  int $int_syscall
  ret

fn_alloc:
  mov %call, $sys_alloc
  int $int_syscall
  ret

fn_free:
  mov %call, $sys_free
  int $int_syscall
  ret


; ===| Program Entry Point |===

_start:
  ; alloc

  mov %r0, $8 ;-----|> 8 bytes alloc
  call fn_alloc ;---|> (64 bit).

  dbg %accumulator

  ; usage

  mov %r1, $123 ; --------------|> storing 64 bit number
  store64 %accumulator, %r1 ; --|> to allocated memory.

  load64 %r0, %accumulator ; ---|> now loading it to verify
  dbg %r0 ; --------------------|> the value from memory.

  ; free

  mov %r0, %accumulator ; ------|> dropping allocated memory
  call fn_free ; ---------------|> by its pointer

  ; exit
  mov %r0, $0
  call fn_exit
