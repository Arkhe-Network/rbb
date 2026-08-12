import { Plugin, TFile, Notice } from 'obsidian';
import * as crypto from 'crypto';

export default class ArkheBridgePlugin extends Plugin {
  async onload() {
    console.log("Loading ArkheBridgePlugin...");

    // 1. Comando: Gerar hash da nota atual
    this.addCommand({
      id: 'generate-note-hash',
      name: 'Generate SHA-256 hash for current note',
      callback: () => this.generateHashForCurrentNote(),
    });

    // 2. Comando: Validar selo
    this.addCommand({
      id: 'validate-seal',
      name: 'Validate seal against hash',
      callback: () => this.validateSealCommand(),
    });

    // 3. Evento: Atualizar hash ao salvar
    this.registerEvent(
      this.app.vault.on('modify', (file) => {
        if (file instanceof TFile && file.extension === 'md') {
          // Apenas atualiza se não foi uma mudança no frontmatter diretamente
          // para evitar loop infinito
          this.updateHashInFrontmatter(file);
        }
      })
    );

    // 4. API exposta para outros plugins
    (this.app as any).arkhe = {
      getNoteHash: (file: TFile) => this.getNoteHash(file),
      validateSeal: (file: TFile) => this.validateSeal(file),
    };
  }

  onunload() {
    console.log("Unloading ArkheBridgePlugin...");
  }

  async generateHashForCurrentNote() {
    const file = this.app.workspace.getActiveFile();
    if (!file) {
      new Notice("Nenhum arquivo ativo.");
      return;
    }

    const content = await this.app.vault.read(file);
    // Removemos o frontmatter atual para calcular o hash apenas do conteúdo
    const contentWithoutFrontmatter = content.replace(/^---[\s\S]+?---\n/, '');
    const hash = crypto.createHash('sha256').update(contentWithoutFrontmatter).digest('hex');

    await this.app.fileManager.processFrontMatter(file, (fm) => {
      fm.hash = hash;
      fm.timestamp = new Date().toISOString();
    });
    new Notice(`Hash gerado: ${hash.slice(0, 8)}...`);
  }

  async updateHashInFrontmatter(file: TFile) {
    const content = await this.app.vault.read(file);
    const contentWithoutFrontmatter = content.replace(/^---[\s\S]+?---\n/, '');
    const newHash = crypto.createHash('sha256').update(contentWithoutFrontmatter).digest('hex');

    // Ler o hash do conteúdo do arquivo em vez do cache (evita loop infinito devido a cache assíncrono)
    const match = content.match(/^hash:\s*"?([^"\n]+)"?$/m);
    const currentHash = match ? match[1] : null;

    if (currentHash !== newHash) {
      try {
        await this.app.fileManager.processFrontMatter(file, (fm) => {
          fm.hash = newHash;
          fm.modified = new Date().toISOString();
        });
      } catch (e) {
        console.error("Erro ao processar frontmatter", e);
      }
    }
  }

  getNoteHash(file: TFile): string | null {
    const cache = this.app.metadataCache.getFileCache(file);
    return cache?.frontmatter?.hash || null;
  }

  validateSeal(file: TFile): boolean {
    const cache = this.app.metadataCache.getFileCache(file);
    const seal = cache?.frontmatter?.selo;
    const hash = cache?.frontmatter?.hash;
    if (!seal || !hash) return false;
    // Verificação simples: selo contém hash parcial
    return seal.includes(hash.slice(0, 8));
  }

  validateSealCommand() {
    const file = this.app.workspace.getActiveFile();
    if (!file) {
      new Notice("Nenhum arquivo ativo para validar selo.");
      return;
    }
    const isValid = this.validateSeal(file);
    if (isValid) {
      new Notice("✅ Selo Válido!");
    } else {
      new Notice("❌ Selo Inválido ou Ausente.");
    }
  }
}
