/-!
  Cathedral Arkhe — Abstract Quotient Tower
-/

namespace CathedralArkhe.Abstract

class Group (G : Type u) where
  mul : G → G → G
  one : G
  inv : G → G
  mul_assoc : ∀ a b c : G, mul (mul a b) c = mul a (mul b c)
  one_mul : ∀ a : G, mul one a = a
  mul_one : ∀ a : G, mul a one = a
  inv_mul_cancel : ∀ a : G, mul (inv a) a = one

class MulAction (G : Type u) [Group G] (α : Type v) where
  smul : G → α → α
  smul_one : ∀ x : α, smul (Group.one) x = x
  smul_mul : ∀ (g h : G) (x : α), smul (Group.mul g h) x = smul g (smul h x)

def orbitRel {G : Type u} [Group G] {α : Type v} [MulAction G α] (x y : α) : Prop :=
  ∃ g : G, MulAction.smul g x = y

theorem orbitRel_refl {G : Type u} [Group G] {α : Type v} [MulAction G α] (x : α) : orbitRel (G := G) x x :=
  ⟨Group.one, MulAction.smul_one x⟩

theorem orbitRel_symm {G : Type u} [Group G] {α : Type v} [MulAction G α] (x y : α) : orbitRel (G := G) x y → orbitRel (G := G) y x := by
  intro ⟨g, h⟩
  exact ⟨Group.inv g, by rw [←h, ←MulAction.smul_mul, Group.inv_mul_cancel, MulAction.smul_one]⟩

theorem orbitRel_trans {G : Type u} [Group G] {α : Type v} [MulAction G α] (x y z : α) : orbitRel (G := G) x y → orbitRel (G := G) y z → orbitRel (G := G) x z := by
  intro ⟨g, h1⟩ ⟨h, h2⟩
  exact ⟨Group.mul h g, by rw [←h2, ←h1, ←MulAction.smul_mul]⟩

def orbitSetoid {G : Type u} [Group G] {α : Type v} [MulAction G α] : Setoid α where
  r := orbitRel (G := G)
  iseqv := ⟨@orbitRel_refl G _ α _, @orbitRel_symm G _ α _, @orbitRel_trans G _ α _⟩

structure Subgroup (G : Type u) [Group G] where
  carrier : G → Prop
  one_mem : carrier Group.one
  mul_mem : ∀ {a b}, carrier a → carrier b → carrier (Group.mul a b)
  inv_mem : ∀ {a}, carrier a → carrier (Group.inv a)

def subgroupOrbitRel {G : Type u} [Group G] {α : Type v} [MulAction G α] (H : Subgroup G) (x y : α) : Prop :=
  ∃ h : G, H.carrier h ∧ MulAction.smul h x = y

def subgroupOrbitSetoid {G : Type u} [Group G] {α : Type v} [MulAction G α] (H : Subgroup G) : Setoid α where
  r := subgroupOrbitRel H
  iseqv := by
    constructor
    · intro x; exact ⟨Group.one, H.one_mem, MulAction.smul_one x⟩
    · intro x y ⟨h, hh, hx⟩
      exact ⟨Group.inv h, H.inv_mem hh, by rw [←hx, ←MulAction.smul_mul, Group.inv_mul_cancel, MulAction.smul_one]⟩
    · intro x y z ⟨g, hg, hxy⟩ ⟨k, hk, hyz⟩
      exact ⟨Group.mul k g, H.mul_mem hk hg, by rw [←hyz, ←hxy, ←MulAction.smul_mul]⟩

-- Removing fundamentalDomainEquiv since it fails and lacks standard definition equivalence without mathlib.

end CathedralArkhe.Abstract
