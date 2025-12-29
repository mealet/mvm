; Strings manual comparison.
; Prints "not equals" if strings are different.
; NOTE: Try to change values in `str1` and `str2` and see how it works

section .data
  str1:
    ascii "hello"
  str2:
    ascii "hello"
  len:
    [. - str2]

  ne_str:
    ascii "not equals!\n"
  ne_len:
    [. - ne_str]
section .text
entry _start

_start:
  ; setting pointers to strings starts
  
  mov %r0, str1
  mov %r1, str2

  add %r0, $8 ; u64 address offset (because of ascii directive)
  add %r1, $8 ; -------------------|

  jmp cmp_loop

cmp_loop:
  ; loading characters from pointers

  load8 %r3, %r0
  load8 %r4, %r1

  ; comparing characters

  cmp %r3, %r4

  ; if not equals (not 0) exit with output

  jnz print_ne

  ; incrementing pointers

  add %r0, $1
  add %r1, $1

  ; checking for null terminator

  load8 %r3, %r0
  load8 %r4, %r1

  mov %accumulator, $0

  cmp %r3, %accumulator
  jz exit

  cmp %r3, %accumulator
  jz exit

  ; returning to loop

  jmp cmp_loop

exit:
  ; -- syscall void exit(int status) --
  mov %r0, $0 ; int status
  mov %call, $0 ; `exit` syscall
  int $syscall ; system call interrupt

print_ne:
  ; -- syscall void write(int output, void *buffer, size_t length) --

  mov %r0, $1 ; int output (stdout)

  mov %r1, ne_str; void* buffer (ptr to message)
  add %r1, $8 ; string address offset

  mov %r2, ne_len ; size_t length (length variable)

  mov %call, $2 ; `write` syscall
  int $syscall ; system call interrupt

  jmp exit
