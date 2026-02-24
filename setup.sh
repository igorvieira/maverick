#!/bin/bash

# Claude Code Setup Script
# Instala MCPs e configura o ambiente

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLAUDE_DIR="$HOME/.claude"
SETTINGS_FILE="$CLAUDE_DIR/settings.json"

echo "🚀 Claude Code Setup"
echo "===================="

# Verifica se o Claude Code está instalado
if ! command -v claude &> /dev/null; then
    echo "❌ Claude Code não encontrado. Instale primeiro: npm install -g @anthropic-ai/claude-code"
    exit 1
fi

echo "✅ Claude Code encontrado"

# Verifica se jq está instalado
if ! command -v jq &> /dev/null; then
    echo "📦 Instalando jq..."
    if command -v brew &> /dev/null; then
        brew install jq
    else
        echo "❌ jq não encontrado. Instale manualmente: brew install jq"
        exit 1
    fi
fi

# Backup do settings.json
if [ -f "$SETTINGS_FILE" ]; then
    cp "$SETTINGS_FILE" "$SETTINGS_FILE.backup"
    echo "📋 Backup criado: $SETTINGS_FILE.backup"
fi

# Adiciona MCPs globais
echo ""
echo "📡 Configurando MCPs globais..."

GLOBAL_MCPS=$(cat "$SCRIPT_DIR/mcp-servers/global.json")

if [ -f "$SETTINGS_FILE" ]; then
    # Merge MCPs globais com settings existentes
    EXISTING=$(cat "$SETTINGS_FILE")
    MERGED=$(echo "$EXISTING" | jq --argjson new "$(echo "$GLOBAL_MCPS" | jq '.mcpServers')" '.mcpServers = (.mcpServers // {}) + $new')
    echo "$MERGED" > "$SETTINGS_FILE"
else
    # Cria novo settings.json
    echo "$GLOBAL_MCPS" | jq '{mcpServers: .mcpServers}' > "$SETTINGS_FILE"
fi

echo "✅ MCPs globais configurados"

# Instala dependências dos MCPs
echo ""
echo "📦 Verificando dependências..."

# uvx (para serena e basic-memory)
if ! command -v uvx &> /dev/null; then
    echo "  Instalando uv..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
fi

echo "✅ Dependências verificadas"

# Resumo
echo ""
echo "🎉 Setup completo!"
echo ""
echo "MCPs Globais instalados:"
echo "  - serena"
echo ""
echo "Para adicionar MCPs por projeto, copie o conteúdo de:"
echo "  $SCRIPT_DIR/mcp-servers/project.json"
echo ""
echo "Para usar o template de CLAUDE.md:"
echo "  cp $SCRIPT_DIR/templates/linear-figma.md /path/to/project/CLAUDE.md"
