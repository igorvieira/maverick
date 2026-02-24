# Claude Code Setup

Minha configuração pessoal do Claude Code com MCPs, skills, commands e workflows.

## Estrutura

```
claude/
├── mcp-servers/              # Configurações de MCP servers
│   ├── global.json           # MCPs globais (todos os projetos)
│   └── project.json          # MCPs por projeto
├── skills/                   # Skills customizadas
│   └── maverick/             # Workflow autônomo de desenvolvimento
│       └── SKILL.md
├── commands/                 # Commands (slash commands)
│   ├── maverick.md           # /maverick - desenvolvimento autônomo
│   ├── maverick-single.md    # /maverick-single - para worktrees
│   ├── senior-architect.md   # /senior-architect - arquitetura
│   ├── senior-frontend.md    # /senior-frontend - React/Next.js
│   ├── senior-backend.md     # /senior-backend - Go/microservices
│   └── senior-qa.md          # /senior-qa - testes e qualidade
├── templates/                # Templates de CLAUDE.md
│   └── linear-figma.md       # Workflow Linear + Figma
└── setup.sh                  # Script de instalação
```

## MCP Servers Disponíveis

### Globais
- **serena** - Agente de código inteligente

### Por Projeto
- **figma** - Integração com Figma (design)
- **linear** - Integração com Linear (tasks)
- **chrome-devtools** - DevTools do Chrome
- **basic-memory** - Memória persistente

## Instalação

### Rápida
```bash
./setup.sh
```

### Manual

1. Copie os MCPs globais:
```bash
# Adicione ao seu ~/.claude/settings.json na chave "mcpServers"
cat mcp-servers/global.json
```

2. Para projetos específicos, adicione ao settings.json do projeto:
```bash
cat mcp-servers/project.json
```

3. Copie o template de CLAUDE.md para seu projeto:
```bash
cp templates/linear-figma.md /path/to/your/project/CLAUDE.md
```

## MCPs Utilizados

| MCP | Tipo | Uso |
|-----|------|-----|
| Figma | HTTP | Design to code |
| Linear | HTTP | Task management |
| Serena | stdio | Code agent |
| Chrome DevTools | stdio | Browser debugging |
| Basic Memory | stdio | Persistent memory |

## Requisitos

- Claude Code CLI instalado
- Node.js (para MCPs stdio)
- Python/uvx (para serena e basic-memory)

## Skills & Commands

### Maverick - Desenvolvimento Autônomo

O Maverick é um workflow que coordena agentes seniores para completar tasks do Linear de ponta a ponta.

```bash
# Single ticket
/maverick AP-552

# Multiple tickets (parallel worktrees)
/maverick AP-552,AP-553,AP-554
```

**Fluxo:**
```
LINEAR → ARCHITECT → [APPROVAL] → IMPLEMENT → QA → DELIVER
           ↑
     ÚNICO CHECKPOINT
```

### Agentes Seniores

| Command | Descrição |
|---------|-----------|
| `/senior-architect` | Análise arquitetural e design de sistemas |
| `/senior-frontend` | Desenvolvimento React/Next.js com design system |
| `/senior-backend` | Desenvolvimento Go/microservices |
| `/senior-qa` | Testes, QA visual (Figma + Chrome DevTools) |

### Instalação dos Commands

Copie a pasta `commands/` e `skills/` para o `.claude/` do seu projeto:

```bash
cp -r commands/ /path/to/project/.claude/
cp -r skills/ /path/to/project/.claude/
```

## Ralph Loop (Plugin)

O Maverick funciona melhor com o plugin `ralph-loop` para execução autônoma:

```bash
/ralph-loop:ralph-loop "/maverick AP-552" --max-iterations 30 --completion-promise "MAVERICK_COMPLETE"
```

O plugin está disponível no marketplace oficial do Claude Code.

## Licença

MIT
