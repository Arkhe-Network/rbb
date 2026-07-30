import Lake
open Lake DSL

/-!
  Cathedral Arkhe — Monorepo v1.3

  Libraries:
    CathedralArkhe           — Core physics (Tiers 0–6). §55.1 gate applies.
    CathedralArkhe.Abstract  — Pure Lean 4 core. SANDBOX-COMPILED. No Mathlib.
    CathedralArkhe.App       — Application modules. Relaxed gate (max 1 sorry).
-/

package cathedralArkhe where
  leanOptions := #[⟨`pp.unicode.fun, true⟩]

@[default_target]
lean_lib CathedralArkhe where

lean_lib CathedralArkhe.Abstract where
  -- Pure Lean 4 core: group actions, quotient towers, agent definitions
  -- Compiles in empty environment. Zero Mathlib imports.

lean_lib CathedralArkhe.App where
  -- Application modules: WCAG, typography, responsive layout
  -- Allowed up to 1 sorry (documented scaffold).

require mathlib from git
  "https://github.com/leanprover-community/mathlib4.git" @ "master"
