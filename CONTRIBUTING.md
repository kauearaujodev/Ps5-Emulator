CONTRIBUTING.md

```markdown
# 🤝 Guia de Contribuição - PS5 Virtual Emulator

Obrigado por considerar contribuir com o PS5 Virtual Emulator! Este documento descreve como você pode ajudar.

---

## 📋 Código de Conduta

Ao participar deste projeto, você concorda em:

- Ser respeitoso com todos os contribuidores
- Fornecer feedback construtivo
- Aceitar críticas de forma profissional
- Focar no que é melhor para a comunidade

---

## 🚀 Como Contribuir

### 1. Reportar Bugs

Use o [GitHub Issues](https://github.com/seu-usuario/ps5-emulator/issues) para reportar bugs.

**Template:**
```markdown
**Descrição**
Descrição clara do bug

**Passos para reproduzir**
1. Faça isso
2. Faça aquilo
3. Veja o erro

**Comportamento esperado**
O que deveria acontecer

**Ambiente**
- OS: [ex: Ubuntu 22.04]
- Rust Version: [ex: 1.75.0]
```

2. Sugerir Melhorias

Use Issues com a tag enhancement.

Template:

```markdown
**Sua ideia**
Descreva sua ideia

**Por que é útil**
Benefícios da implementação

**Como implementar**
Sugestão de implementação
```

3. Pull Requests

Passos:

1. Fork o repositório
2. Clone seu fork:
   ```bash
   git clone https://github.com/seu-usuario/ps5-emulator.git
   cd ps5-emulator
   ```
3. Crie uma branch:
   ```bash
   git checkout -b feature/sua-feature
   ```
4. Faça suas alterações
5. Teste:
   ```bash
   cargo test
   cargo fmt
   cargo clippy
   ```
6. Commit:
   ```bash
   git commit -m "feat: descrição clara"
   ```
7. Push:
   ```bash
   git push origin feature/sua-feature
   ```
8. Abra um Pull Request

---

📝 Convenções de Código

Estilo

· Use cargo fmt para formatação
· Siga as diretrizes do Rust API Guidelines
· Comentários em português ou inglês (consistente)

Commits

Siga Conventional Commits:

· feat: Nova funcionalidade
· fix: Correção de bug
· docs: Documentação
· style: Formatação
· refactor: Refatoração
· test: Testes
· chore: Manutenção

Exemplos:

```
feat(cpu): adiciona suporte a instrução AVX
fix(memory): corrige bug na alocação
docs(readme): atualiza instruções
```

Testes

· Escreva testes para novas funcionalidades
· Use cargo test para executar todos os testes

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nova_funcionalidade() {
        // Arrange
        // Act
        // Assert
    }
}
```

---

🏗️ Arquitetura do Projeto

Módulos Principais

· cpu: Emulação da CPU Zen 2 (8 cores)
· memory: Gerenciamento de memória virtual (2GB)
· games: Sistema de pacotes PKG e biblioteca

Estrutura de Pastas

```
src/
├── main.rs          # Ponto de entrada
├── memory.rs        # Memória virtual
├── cpu.rs           # CPU PS5
└── games/
    ├── mod.rs       # Exporta módulos
    ├── package.rs   # Pacotes PKG
    ├── installer.rs # Instalador
    └── library.rs   # Biblioteca de jogos
```

---

🧪 Ambiente de Desenvolvimento

Configuração

```bash
# Instale as dependências
rustup component add clippy
rustup component add rustfmt
```

Comandos Úteis

```bash
# Build
cargo build

# Build com otimizações
cargo build --release

# Executar
cargo run

# Executar com logs
RUST_LOG=debug cargo run

# Testes
cargo test

# Verificar estilo
cargo fmt --check

# Verificar com clippy
cargo clippy -- -D warnings

# Documentação
cargo doc --open
```

---

📊 Roadmap

Versão 0.1.0 (Atual)

☑ CPU básica (8 cores)
☑ Memória virtual
☑ Sistema de pacotes PKG
☑ Biblioteca de jogos
☑ 3 jogos demo

Versão 0.2.0 (Próximo)

☐ Mais instruções x86-64
☐ Sistema de GPU básico
☐ Interface CLI avançada
☐ Suporte a saves externos

Versão 0.3.0 (Futuro)

☐ Interface gráfica (GUI)
☐ Networking
☐ Suporte a controles
☐ Sistema de conquistas online

---

❓ FAQ

Como posso ajudar sem saber Rust?

· Melhorar documentação
· Testar o emulador
· Reportar bugs
· Sugerir melhorias

Preciso de hardware específico?

Não, o emulador roda em qualquer sistema com Rust.

Posso usar este código em outros projetos?

Sim, sob a licença MIT.

---

📞 Contato

· GitHub: Kauedev
· Discord: Não temos
· Email: emailps5emulatorsuport.email.com

---

Agradecemos sua contribuição! 🙏
