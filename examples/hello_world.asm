; Hello World Example
; %call = %r9
; $syscall = $80

section .data
  hello:
    ascii "Hello, World!"

  len:
    [. - hello]

section .text
entry _start

_start:
  ; -- syscall void write(int output, void *buffer, size_t length) --

  mov %r0, $0 ; int output (stdout)

  mov %r1, hello ; void* buffer (ptr to message)
  mov %r2, len ; size_t length (length variable)

  mov %call, $2 ; `write` syscall
  int $syscall ; system call interrupt

  jmp exit

exit:
  ; -- syscall void exit(int status) --
  mov %r0, $0 ; int status
  mov %call, $0 ; `exit` syscall
  int $syscall ; system call interrupt
