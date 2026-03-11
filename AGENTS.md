# OpenVoice

OpenVoice e um app desktop Linux-first para transcricao, copiloto de reunioes, ditado e memoria pessoal conectada ao Obsidian.

O repositorio esta sendo construido em Rust com `Iced`.

## Estado Atual

Hoje o projeto implementa:

- HUD desktop leve
- settings persistidas em disco
- captura gravada com microfone e system audio
- transcricao pos-captura
- transcricao realtime do audio do sistema
- perfis de realtime
- persistencia local de sessoes realtime
- janela de sessoes gravadas
- subtitle window para transcript realtime
- fluxo OAuth preparado para capacidades futuras

Ainda nao implementa:

- home / hub do produto
- chat contextual
- workspace de sessao
- integracao funcional com Obsidian
- hotkeys globais
- key mapping configuravel
- captura de tela

## Estrutura Do Projeto

```text
src/
├── main.rs
├── app/
├── ui/
└── platform/
```

### `src/main.rs`

Entrypoint minimo. So sobe a aplicacao.

### `src/app`

Casca da aplicacao Iced.

Use para:

- bootstrap do app
- definicao de `Message`
- `State`
- `update`
- subscriptions globais

Regra:

- nao colocar regra visual aqui
- nao enterrar integracoes Linux diretamente em `bootstrap` ou `update` se puderem morar em `platform` ou `modules`

### `src/ui`

Camada de interface.

Use para:

- views
- componentes
- tema
- estilos

Regra:

- `view` deve ser puro
- UI nao deve descobrir estado do sistema operacional sozinha
- evitar side effects nesta camada

### `src/platform`

Integracoes com Linux e detalhes de runtime.

Use para:

- janela
- monitor
- deteccao via `xrandr`
- hotkeys globais
- audio
- adaptacoes X11/Wayland

Regra:

- tudo que depende de backend ou comportamento do desktop deve ficar aqui

## Estrutura Alvo

Conforme o produto crescer, a estrutura deve convergir para:

```text
src/
├── main.rs
├── app/
├── ui/
├── platform/
├── modules/
│   ├── overlay/
│   ├── meetings/
│   ├── obsidian/
│   ├── audio/
│   └── dictation/
└── support/
```

Cada modulo em `src/modules` deve tender para:

```text
module-name/
├── application/
├── domain/
└── infrastructure/
```

Significado:

- `application`: casos de uso e orquestracao
- `domain`: entidades, regras e contratos
- `infrastructure`: adapters concretos, IO, providers, filesystem, APIs

## Regras De Arquitetura

- manter a arquitetura do Iced clara: `State`, `Message`, `update`, `view`
- usar `Task` para trabalho solicitado por uma mensagem
- usar `Subscription` para streams passivos
- nao transformar `update` em saco de gatos
- nao misturar regra de negocio com rendering
- isolar comportamento especifico de Linux/X11/Wayland em `platform`
- testar funcoes puras primeiro, principalmente parsers e regras

## Linux E Backend

Este projeto atualmente prioriza `x11`/XWayland no `Cargo.toml`.

Motivo:

- no backend Wayland do `winit`, `always-on-top` nao esta implementado
- alguns comportamentos de overlay sao inconsistentes em GNOME/Wayland

Se uma feature depender de:

- z-order
- posicionamento exato de janela
- click-through
- monitor geometry

valide primeiro se o backend atual suporta isso.

## Objetivo De Produto

O objetivo nao e ser um helper de entrevista.

O objetivo e ser um copiloto pessoal Linux-native para:

- ouvir o desktop em realtime
- transcrever
- permitir perguntar para uma LLM com contexto da sessao atual
- aceitar contexto visual no futuro
- reutilizar memoria pessoal do Obsidian
- oferecer um HUD rapido e um hub de produto mais rico

## Documentos

- visao e roadmap de produto: [docs/PRD.md](/home/israel/projetos/projetos-opensource/openvoice/docs/PRD.md)
- estado atual implementado: [docs/spec.md](/home/israel/projetos/projetos-opensource/openvoice/docs/spec.md)
- backlog tecnico atual: [docs/TODO.md](/home/israel/projetos/projetos-opensource/openvoice/docs/TODO.md)

## Skills Locais

Skills uteis neste repo:

- `iced-rust-best-practices`
- `frontend-design`
- `electrobun-best-practices` apenas para referencia historica, nao para a stack atual
