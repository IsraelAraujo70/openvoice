# OpenVoice

OpenVoice e um app desktop Linux-first para capturar reunioes, apoiar fluxo de ditado e salvar resultado no Obsidian.

O repositorio esta sendo reconstruido em Rust com `Iced`, com foco inicial em um overlay leve para Linux.

## Estado Atual

Hoje o projeto implementa:

- janela transparente
- overlay minimo `OPENVOICE / WIP`
- tentativa de mouse passthrough
- inicializacao no monitor principal
- preferencia por `x11`/XWayland para suportar melhor comportamento de janela

Ainda nao implementa:

- captura de audio
- transcricao
- integracao com Obsidian
- hotkeys globais
- selecao manual de monitor
- fluxo real de reuniao

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

O objetivo nao e ser um "helper de entrevista".

O objetivo e ser um assistente de reunioes e fluxo de pensamento para Linux:

- capturar reunioes
- transcrever
- resumir
- salvar no Obsidian
- oferecer um HUD/overlay minimo e rapido

## Documentos

- escopo do produto: [docs/PRD.md](/home/israel/projetos/projetos-opensource/openvoice/docs/PRD.md)

## Skills Locais

Skills uteis neste repo:

- `iced-rust-best-practices`
- `frontend-design`
- `electrobun-best-practices` apenas para referencia historica, nao para a stack atual
