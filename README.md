# Claude Code Setup

Minha configuração pessoal do Claude Code com MCPs, templates e workflows.

## Estrutura

```
claude/
├── mcp-servers/           # Configurações de MCP servers
│   ├── global.json        # MCPs globais (todos os projetos)
│   └── project.json       # MCPs por projeto (copiar para projetos específicos)
├── templates/             # Templates de CLAUDE.md
│   └── linear-figma.md    # Workflow Linear + Figma
└── setup.sh               # Script de instalação
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

## Licença

MIT
