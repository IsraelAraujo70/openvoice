# PRD

## Produto

OpenVoice e um app Linux-first para:

- capturar audio local e de reunioes
- transformar audio em texto
- oferecer transcricao gravada e transcricao realtime
- permitir interacao com LLM durante a reuniao
- sugerir contexto, respostas e proximos passos em tempo real
- preparar esse conteudo para organizacao futura em notas
- manter um HUD leve para controle rapido

O produto e inspirado na fluidez de ferramentas de assistencia em tempo real, mas o foco aqui nao e entrevista. O foco e produtividade pessoal, reunioes e captura de pensamento.

## Problema

Hoje o fluxo tipico de quem trabalha em Linux com reunioes e notas e ruim:

- gravar reuniao e separado do ato de anotar
- transcricao costuma exigir ferramentas externas ou SaaS fechados
- notas finais raramente chegam organizadas
- overlays desktop no Linux sofrem com limitacoes de compositor e backend

Resultado:

- perda de contexto
- retrabalho manual
- notas inconsistentes
- friccao alta para transformar fala em conhecimento util

## Visao

Construir um assistente Linux-native que capture audio, gere texto util e organize conhecimento com o minimo de friccao possivel, preservando uma arquitetura modular para crescer de HUD minimo para fluxo completo de reuniao, ditado e notas.

Em direcao de produto, o OpenVoice deve evoluir de transcritor para copiloto pessoal de reunioes e raciocinio:

- escutar a conversa em tempo real
- consultar contexto da sessao atual
- recuperar contexto do historico salvo no Obsidian
- permitir perguntas ao vivo para uma LLM durante a reuniao
- sugerir o que falar com base no contexto da conversa e nas notas anteriores
- salvar o resultado final como conhecimento persistente

## Usuario-Alvo

Principal:

- pessoas que usam Linux diariamente
- usuarios de Obsidian
- devs, founders, researchers, PMs e criadores que participam de reunioes frequentes

Secundario:

- usuarios que querem ditado continuo estilo flow
- quem prefere controle local e baixo acoplamento com SaaS

## Principios De Produto

- Linux first
- rapido para abrir e usar
- friccao minima
- integracao forte com Obsidian no caminho de produto
- arquitetura que aceite processamento local e remoto
- overlay simples, nao chamativo

## Estado Atual

Hoje o projeto ja implementa:

- HUD desktop leve em Rust + Iced
- settings persistidas em disco
- captura dual gravada:
  - microfone
  - system audio
- persistencia local por sessao com WAVs e metadata
- transcricao pos-captura via OpenRouter
- transcricao realtime do audio do sistema via OpenAI Realtime usando `OpenAI API key`
- seletor de idioma para o realtime
- clipboard com texto final combinado por trilha
- fluxo OAuth com ChatGPT persistido para uso futuro

Hoje o projeto ainda nao implementa:

- area dedicada de UI para transcript realtime
- chat contextual ao vivo sobre a reuniao atual
- sugestoes em tempo real do que falar
- recuperacao de contexto do Obsidian durante a sessao
- integracao funcional com Obsidian
- diarizacao real
- selecao manual de fonte de system audio
- hotkeys globais
- resumo e estruturacao automatica de notas

## Escopo Atual Do MVP

### 1. Overlay Shell

- abrir no monitor principal
- ficar acima das outras janelas quando o backend permitir
- suportar mouse passthrough quando o backend permitir
- exibir estado minimo do app

### 2. Captura Gravada

- iniciar captura manualmente
- parar captura manualmente
- gravar microfone e system audio em trilhas separadas
- persistir artefatos locais por sessao

### 3. Transcricao Pos-Captura

- transcrever `mic.wav` e `system.wav` em sequencia
- salvar `transcripts.json`
- copiar um texto combinado para o clipboard

### 4. Transcricao Realtime

- iniciar e parar a sessao manualmente
- transcrever o audio do sistema em streaming
- permitir escolha de modelo e idioma

### 5. Copiloto Ao Vivo

- preparar o produto para chat contextual durante a reuniao
- permitir perguntas para a LLM sobre o que acabou de ser dito
- preparar sugestoes de resposta e talking points em tempo real

### 6. Providers E Auth

- usar `OpenRouter` para o fluxo gravado
- usar `OpenAI API key` para o fluxo realtime
- manter `OpenAI OAuth` como base de autenticacao para modelos de escrita e fluxos futuros

## Fora Do MVP Atual

- colaboracao em tempo real
- compartilhamento multiusuario
- automacoes complexas de calendario
- analytics
- suporte amplo a todos os desktops Linux nativamente
- resumo em streaming altamente refinado
- salvamento automatico no Obsidian

## Requisitos Funcionais

### Overlay

- o app deve abrir com um HUD pequeno e discreto
- o app deve tentar respeitar monitor principal
- o app deve funcionar em X11/XWayland primeiro

### Captura Gravada

- o usuario deve conseguir iniciar e parar uma sessao
- cada sessao deve gerar um identificador e artefatos locais
- a sessao deve salvar `mic.wav`, `system.wav` e `metadata.json`

### Transcricao Gravada

- o sistema deve transcrever as duas trilhas separadamente
- a saida deve ser persistida em `transcripts.json`
- o clipboard deve receber um texto combinado por trilha

### Transcricao Realtime

- o usuario deve conseguir iniciar o realtime sem depender de OAuth
- o realtime deve usar `OpenAI API key`
- o realtime deve aceitar idioma `Auto` ou um idioma fixo configurado pelo usuario
- a camada de realtime nao deve ficar acoplada ao fluxo gravado

### Copiloto Ao Vivo

- o produto deve permitir perguntas para uma LLM durante a reuniao
- as respostas devem poder usar a conversa atual como contexto
- o produto deve poder evoluir para sugerir o que falar em seguida
- o produto deve poder usar notas e historico como memoria contextual

### Providers E Auth

- o sistema deve suportar integracao com OpenRouter
- o sistema deve suportar integracao com OpenAI Realtime via API key
- o sistema deve prever um fluxo OAuth com conta OpenAI para features futuras
- auth e provider nao devem ficar misturados com a camada de UI
- tokens, sessao e refresh devem morar em uma camada dedicada de infraestrutura

### Obsidian

- o produto deve preservar um caminho claro para integracao com Obsidian
- o produto deve evoluir para salvar sessoes, resumos, decisoes e proximos passos no Obsidian
- o Obsidian deve servir como memoria persistente para consultas futuras
- a primeira entrega funcional de Obsidian ainda nao faz parte do estado implementado atual

## Requisitos Nao Funcionais

- startup rapido
- baixo uso de memoria
- arquitetura modular
- codigo legivel
- comportamento previsivel entre backends suportados

## Restricoes Tecnicas

- stack atual: Rust + Iced
- backend preferido no momento: X11/XWayland
- Wayland puro pode limitar `always-on-top`, posicionamento e certos comportamentos de overlay
- system audio depende hoje de `pactl` e `parec`

## Arquitetura Atual

```text
src/
├── app/
├── ui/
├── platform/
└── modules/
    ├── audio/
    ├── auth/
    ├── dictation/
    ├── live_transcription/
    └── settings/
```

## Arquitetura Desejada

```text
src/
├── app/
├── ui/
├── platform/
├── modules/
│   ├── overlay/
│   ├── meetings/
│   ├── audio/
│   ├── dictation/
│   ├── live_transcription/
│   ├── auth/
│   └── obsidian/
└── support/
```

## Fases

### Fase 1

- overlay shell estavel
- monitor principal
- passthrough
- always-on-top quando suportado

### Fase 2

- captura dual de audio
- sessoes
- persistencia local
- transcricao pos-captura

### Fase 3

- transcricao realtime
- configuracao de provider por fluxo
- refinamento de UX do HUD

### Fase 4

- chat contextual ao vivo sobre a sessao atual
- recuperacao de contexto no Obsidian
- sugestoes do que falar com base na reuniao e nas notas
- modelos de escrita usando OAuth quando fizer sentido de produto
- integracao inicial com Obsidian
- hotkeys globais
- templates e automacao de notas

## Criterios De Sucesso Do MVP

- abrir rapido no Linux
- permitir iniciar uma captura gravada
- permitir iniciar uma sessao realtime
- gerar transcricao basica nos dois fluxos
- persistir artefatos locais de forma previsivel
- manter codigo organizado para crescer sem reescrita total
