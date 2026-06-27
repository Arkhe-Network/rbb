cat << 'INNER_EOF' > specs/squidbleed_detection.tla
---- MODULE squidbleed_detection ----
EXTENDS Integers, Sequences

CONSTANTS MAX_BUFFER, MAX_STRING

VARIABLES
  buffer,        (* \in Seq(0..255) *)
  pointer,       (* 0..MAX_BUFFER *)
  copyFrom,      (* 0..MAX_BUFFER *)
  end_of_string  (* 0..MAX_BUFFER *)

(* Função strchr no padrão C11 — inclui o terminador nulo na busca *)
strchr(ch, str, i) ==
    LET pos == CHOOSE j \in i..Len(str) : str[j] = ch
    IN IF pos = Len(str) THEN pos ELSE pos

Init ==
    /\ buffer = <<>>
    /\ pointer = 1
    /\ copyFrom = 1
    /\ end_of_string = 10

(* ============================================================ *)
(* O BUG QUE CAUSOU O SQUIDBLEED: strchr sem verificação de \0 *)
(* ============================================================ *)

(* Loop vulnerável — exatamente o que estava no Squid Proxy *)
VulnerableLoop ==
    /\ pointer < end_of_string
    /\ copyFrom < end_of_string
    /\ strchr(32, buffer, pointer) \in pointer..end_of_string
    /\ pointer' = strchr(32, buffer, pointer)

(* == CORREÇÃO == *)
(* Loop correto: verifica se chegou ao fim da string antes de chamar strchr *)
CorrectLoop ==
    /\ pointer <= end_of_string
    /\ IF pointer < end_of_string /\ buffer[pointer] # 0
       THEN
           LET pos == strchr(32, buffer, pointer)
           IN pointer' = IF pos < end_of_string THEN pos + 1 ELSE pointer
       ELSE
           pointer' = pointer

Next == CorrectLoop \/ VulnerableLoop

(* Invariante de segurança: pointer nunca ultrapassa end_of_string *)
Invariant ==
    pointer <= end_of_string

(* Propriedade que deve ser verificada: o loop nunca ultrapassa o buffer *)
SafetyProperty ==
    [] (CorrectLoop => Invariant)

(* Propriedade violada pelo bug: *)
VulnerableProperty ==
    [] (VulnerableLoop => Invariant)  (* FALSO! *)

====
INNER_EOF

cat << 'INNER_EOF' > specs/syscall.tla
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
INNER_EOF
