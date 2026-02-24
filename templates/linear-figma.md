# Workflow de Desenvolvimento com Linear + Figma

Este projeto utiliza MCPs do Linear e Figma para um fluxo estruturado de desenvolvimento.

## Fluxo de Trabalho

### 1. Leitura da Task (Linear)

Ao receber uma task do Linear:
- Usar `mcp__linear__get_issue` para obter detalhes completos
- Identificar links do Figma na descrição
- Extrair requisitos e critérios de aceite
- Apresentar resumo ao usuário para confirmação

### 2. Análise do Design (Figma)

Se houver link do Figma:
- Extrair `fileKey` e `nodeId` da URL
- Usar `mcp__figma__get_design_context` para obter código e estrutura
- Usar `mcp__figma__get_variable_defs` para tokens de design (cores, espaçamentos, tipografia)
- Usar `mcp__figma__get_screenshot` para visualização do design
- Documentar tokens encontrados para uso na implementação

### 3. Planejamento

SEMPRE entrar em modo de planejamento antes de implementar:
- Usar `EnterPlanMode` para tarefas não-triviais
- Listar todos os arquivos que serão criados/modificados
- Detalhar componentes necessários
- Mapear tokens do Figma para variáveis CSS/código
- Aguardar aprovação do usuário antes de prosseguir

### 4. Implementação

Durante a implementação:
- Criar tasks com `TaskCreate` para acompanhamento
- Seguir tokens e especificações do Figma
- Manter código limpo e sem over-engineering
- Atualizar status das tasks conforme progresso

### 5. Revisão

Após implementação:
- Revisar código gerado
- Verificar aderência ao design do Figma
- Garantir que todos os requisitos foram atendidos
- Apresentar resultado ao usuário

### 6. Documentação Final (Linear)

Ao finalizar a task:
- Usar `mcp__linear__create_comment` para adicionar na issue:
  - Link do PR (se houver)
  - Passo a passo de como testar
  - Screenshots ou observações relevantes
- Usar `mcp__linear__update_issue` para atualizar status se necessário

## Comandos Úteis

### Buscar minhas tasks
```
mcp__linear__list_issues com assignee: "me"
```

### Buscar task específica
```
mcp__linear__get_issue com id: "ISSUE-123"
```

### Obter contexto do Figma
```
URL: https://figma.com/design/{fileKey}/{fileName}?node-id={nodeId}
mcp__figma__get_design_context com fileKey e nodeId extraídos
```

## Formato do Comentário de Teste (Linear)

Ao finalizar, adicionar comentário no formato:

```markdown
## Implementação Concluída

### Arquivos Modificados
- `path/to/file.tsx` - Descrição da mudança

### Como Testar
1. Passo 1
2. Passo 2
3. Passo 3

### Observações
- Notas relevantes sobre a implementação
```

## Git e Commits

- **NUNCA** incluir "Co-Authored-By" nos commits - apenas o autor do usuário
- Commits devem ser simples, sem assinaturas extras
- Formato do commit: apenas a mensagem, sem rodapé de co-autoria

## Regras

- SEMPRE aguardar aprovação do usuário antes de implementar
- SEMPRE ler a task completa antes de começar
- SEMPRE verificar o Figma quando disponível
- NUNCA pular a fase de planejamento para tasks complexas
- NUNCA atualizar status do Linear sem confirmação do usuário
