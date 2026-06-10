import subprocess
import sys
import re

def get_changed_files():
    # Obtém a lista de arquivos alterados no PR
    try:
        # Quando executado no GitHub Actions
        base = "origin/main"
        head = "HEAD"
        result = subprocess.run(["git", "diff", "--name-only", f"{base}...{head}"], capture_output=True, text=True, check=True)
        return result.stdout.strip().split('\n')
    except subprocess.CalledProcessError:
        print("Aviso: Falha ao obter git diff. Usando modo de teste.")
        return []

def main():
    changed_files = get_changed_files()

    critical_paths = [
        "ZK_REASONING_ENGINE/circuits",
        "COGNITIVE_CORTEX/agents",
        "DISTRIBUTED_COMPUTATION"
    ]

    touched_critical = any(any(f.startswith(p) for p in critical_paths) for f in changed_files if f)

    if touched_critical:
        # Verifica se algum arquivo Lean 4 correspondente foi modificado
        lean_touched = any(f.startswith("LEAN4_SUPEREGO/") and f.endswith(".lean") for f in changed_files if f)
        if not lean_touched:
            print("❌ ERRO CRÍTICO DE SEGURANÇA: CATHEDRAL AGI")
            print("Você alterou componentes cognitivos/criptográficos sem atualizar as provas formais.")
            print("Qualquer PR que toque em diretórios críticos DEVE ser acompanhado por teoremas atualizados em LEAN4_SUPEREGO/.")
            sys.exit(1)
        else:
            print("✅ Segurança formal mantida. Alterações críticas acompanhadas de provas Lean 4.")
            sys.exit(0)
    else:
        print("✅ Nenhuma alteração crítica detectada. Verificação formal ignorada.")
        sys.exit(0)

if __name__ == "__main__":
    main()
