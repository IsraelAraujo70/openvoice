# PRD

## Produto

OpenVoice e um app Linux-first para:

- capturar reunioes
- transformar audio em texto
- organizar notas automaticamente
- salvar e enriquecer esse conteudo no Obsidian
- oferecer um overlay/HUD leve para controle rapido

O produto e inspirado na fluidez de ferramentas de assistencia em tempo real, mas o foco aqui nao e entrevista. O foco e produtividade pessoal, reunioes e captura de pensamento.

## Problema

Hoje o fluxo tipico de quem trabalha em Linux com reunioes e notas e ruim:

- gravar reuniao e separado do ato de anotar
- transcricao costuma exigir ferramentas externas ou SaaS fechados
- notas finais raramente chegam organizadas no Obsidian
- overlays desktop no Linux sofrem com limitacoes de compositor e backend

Resultado:

- perda de contexto
- retrabalho manual
- notas inconsistentes
- friccao alta para transformar fala em conhecimento util

## Visao

Construir um assistente Linux-native que capture audio, gere texto util e organize conhecimento no Obsidian com o minimo de friccao possivel.

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
- integracao forte com Obsidian
- arquitetura que aceite processamento local e remoto
- overlay simples, nao chamativo

## Escopo Do MVP

### 1. Overlay Shell

- abrir no monitor principal
- ficar acima das outras janelas quando o backend permitir
- suportar mouse passthrough quando o backend permitir
- exibir estado minimo do app

### 2. Sessao De Captura

- iniciar captura manualmente
- parar captura manualmente
- mostrar estado da sessao

### 3. Pipeline Basico De Audio

- selecionar input de audio
- salvar audio temporario da sessao
- preparar esse audio para transcricao

### 4. Transcricao Basica

- transcrever uma sessao apos encerramento
- persistir transcricao em arquivo local

### 5. Obsidian Basico

- configurar caminho do vault
- criar nota de reuniao
- salvar transcricao e resumo simples

### 6. Provider Basico

- preparar a arquitetura para multiples providers de IA
- permitir integracao futura com OpenRouter
- permitir integracao futura com login OpenAI via fluxo de conta/assinatura quando esse caminho for adotado pelo produto

## Fora Do MVP

- colaboracao em tempo real
- compartilhamento multiusuario
- automacoes complexas de calendario
- analytics
- suporte amplo a todos os desktops Linux nativamente
- resumo em streaming altamente refinado

## Requisitos Funcionais

### Overlay

- o app deve abrir com um HUD pequeno e discreto
- o app deve tentar respeitar monitor principal
- o app deve funcionar em X11/XWayland primeiro

### Captura

- o usuario deve conseguir iniciar e parar uma sessao
- cada sessao deve gerar um identificador e artefatos locais

### Transcricao

- o sistema deve aceitar providers locais ou remotos no futuro
- a camada de transcricao nao deve ficar acoplada a uma unica implementacao

### Providers E Auth

- o sistema deve suportar integracao com OpenRouter
- o sistema deve prever um fluxo de autenticacao com conta OpenAI inspirado em abordagens como a do OpenClaw
- auth e provider nao devem ficar misturados com a camada de UI
- tokens, sessao e refresh devem morar em uma camada dedicada de infraestrutura

### Obsidian

- o usuario deve definir o vault alvo
- o app deve criar ou atualizar uma nota de reuniao

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

- captura de audio
- sessoes
- persistencia local

### Fase 3

- transcricao
- resumo
- integracao inicial com Obsidian

### Fase 4

- integracao com OpenRouter
- login OpenAI via fluxo de conta/assinatura
- flow dictation
- hotkeys globais
- templates e automacao de notas

## Criterios De Sucesso Do MVP

- abrir rapido no Linux
- permitir iniciar uma sessao
- gerar transcricao basica
- salvar resultado no Obsidian
- manter codigo organizado para crescer sem reescrita total
