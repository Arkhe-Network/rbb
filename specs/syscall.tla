---- MODULE syscall ----
EXTENDS Integers, Sequences

VARIABLES memory, pointer, buffer_size, str

(* strchr no padrão C11 — inclui \0 na busca *)
strchr(char, s) ==
    LET pos == CHOOSE i \in 1..Len(s) : s[i] = char
    IN pos

Init ==
    /\ memory = <<>>
    /\ pointer = 1
    /\ buffer_size = 10
    /\ str = <<>>

(* Loop correto: verifica bounds antes de strchr *)
CorrectLoop ==
    /\ pointer \in 1..buffer_size
    /\ pointer < buffer_size /\ str[pointer] # 0
    /\ strchr(' ', str) # 0 => pointer' = pointer + 1
    \/ pointer = buffer_size \/ str[pointer] = 0 => pointer' = pointer

(* Violação Squidbleed: sem verificação de \0 *)
VulnerableLoop ==
    /\ pointer \in 1..buffer_size
    /\ strchr(' ', str) # 0 => pointer' = pointer + 1

Next == CorrectLoop \/ VulnerableLoop
====
