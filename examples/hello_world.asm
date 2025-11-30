; Hello World Example
; %call = %r9
; $syscall = $0x80

.data
  hello:
    .ascii "Hello, World!"
  len = [. - &hello]

.text

_start:
  // -- syscall void write(int output, void *buffer, size_t length) --

  mov %r0, $0 // int output (stdout)
  ldr %r1, &hello // void* buffer (ptr to message)
  ldr %r2, len // size_t length (length variable)

  mov %call, $2 // `write` syscall
  int $syscall // system call interrupt

  // -- syscall void exit(int status) --
  mov %r0 $0 // int status
  mov %call $0 // `exit` syscall
  int $syscall // system call interrupt
