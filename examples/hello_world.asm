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
