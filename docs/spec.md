# Spec

## Objetivo

Este documento descreve o estado atual da aplicacao e as restricoes tecnicas atuais.

Ele nao substitui o PRD.

O PRD descreve a direcao de produto.
O `spec.md` descreve o que existe hoje e como o app esta estruturado neste momento.

## Stack Atual

- Rust
- Iced
- foco atual em Linux
- backend preferido no momento: X11 / XWayland

## Estado Atual

Hoje a aplicacao implementa:

- HUD desktop leve
- settings persistidas em disco
- captura gravada com microfone e system audio
- transcricao pos-captura
- transcricao realtime do audio do sistema
- perfis de realtime
- persistencia local de sessoes
- janela de sessoes gravadas
- subtitle window para transcript realtime
- auth OAuth preparado para fluxos futuros

## Estado De UX Atual

Hoje a UX ainda esta em transicao.

O app possui:

- HUD principal
- janela de settings
- janela de sessoes
- janela de subtitle

Ainda nao possui:

- home / hub consolidado
- chat surface
- session workspace completo
- busca de sessoes
- captura de tela
- key mapping configuravel

## Fluxos Atuais

### Realtime

- captura do audio do sistema via `parec`
- envio para OpenAI Realtime via `OpenAI API key`
- subtitle window com parcial e consolidado
- persistencia incremental local

### Gravado

- captura local
- transcricao posterior
- copia para clipboard

## Providers Atuais

- `OpenRouter` no fluxo gravado
- `OpenAI Realtime` no fluxo realtime
- `OpenAI OAuth` mantido para capacidades futuras

## Persistencia Atual

- settings em disco
- sessoes realtime em SQLite local
- artefatos de audio e metadata locais para o fluxo gravado

## Restricoes Tecnicas

- Wayland puro pode limitar comportamento de janela
- system audio depende de `pactl` e `parec`
- parte da ergonomia do overlay depende do backend de janela

## Estrutura Atual

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

## Lacunas Atuais

- falta home do produto
- falta chat contextual
- falta detalhe de sessao como workspace
- falta busca no historico
- falta captura visual
- falta integracao funcional com Obsidian
- faltam hotkeys globais e key mapping

## Observacao

Este documento deve ser atualizado sempre que o estado implementado mudar de forma relevante.
