# Spec

## Objetivo

Este documento descreve o estado atual da aplicacao.

Ele existe para responder:

- o que ja esta implementado
- como o codigo esta organizado hoje
- quais contratos e fluxos ja existem
- quais lacunas tecnicas ainda precisam ser fechadas

O `PRD.md` descreve a direcao de produto.
O `spec.md` descreve a realidade implementada neste momento.

## Stack Atual

- Rust
- Iced
- Linux-first
- backend preferido no momento: X11 / XWayland
- `cpal` para microfone
- `parec` + `pactl` para system audio
- `tungstenite` para websocket realtime
- `rusqlite` para persistencia local de sessoes realtime

## Shell Da Aplicacao

Hoje a aplicacao roda como um shell em `src/app/` com:

- `State` central em [`src/app/state.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/app/state.rs)
- `Message` global em [`src/app/message.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/app/message.rs)
- `update` em [`src/app/update.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/app/update.rs)
- bootstrap em [`src/app/bootstrap.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/app/bootstrap.rs)

### Janelas Atuais

O runtime hoje lida com duas janelas:

- janela principal (compartilhada entre HUD e Home via morph)
- subtitle window (flutuante, passthrough)

A janela principal alterna entre dois modos:

- **HUD**: barra compacta (380x96), sempre no topo, transparente
- **Home**: view expandida (700x800), nivel normal, com abas

Ainda nao existe:

- chat surface
- session workspace completo

## Modulos Atuais

### `settings`

Arquivos:

- [`src/modules/settings/domain.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/modules/settings/domain.rs)
- [`src/modules/settings/application.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/modules/settings/application.rs)
- [`src/modules/settings/infrastructure.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/modules/settings/infrastructure.rs)

Responsabilidade atual:

- representar settings persistidas
- validar e normalizar inputs
- carregar e salvar `settings.json`

### Settings Persistidas Hoje

Campos atuais em `AppSettings`:

- `openrouter_api_key`
- `openai_realtime_api_key`
- `openrouter_model`
- `openai_realtime_model`
- `openai_realtime_language`
- `openai_realtime_profile`

Defaults atuais:

- `openrouter_model = google/gemini-2.5-flash-lite:nitro`
- `openai_realtime_model = gpt-4o-transcribe`
- `openai_realtime_language = ""`
- `openai_realtime_profile = balanced`

Perfis suportados:

- `caption`
- `balanced`
- `accuracy`

Caminho atual do arquivo:

- `~/.config/openvoice/settings.json`
  ou `XDG_CONFIG_HOME/openvoice/settings.json`

Observacao:

- hoje settings ainda misturam provider config com preferencia de UX de realtime
- no futuro talvez valha separar `provider settings`, `realtime tuning` e `user preferences`

### `audio`

Arquivos principais:

- [`src/modules/audio/domain.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/modules/audio/domain.rs)
- [`src/modules/audio/infrastructure/microphone.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/modules/audio/infrastructure/microphone.rs)
- [`src/modules/audio/infrastructure/system.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/modules/audio/infrastructure/system.rs)
- [`src/modules/audio/infrastructure/storage.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/modules/audio/infrastructure/storage.rs)

Responsabilidade atual:

- capturar microfone
- capturar system audio
- persistir artefatos de captura no fluxo gravado

#### Captura De Microfone

Hoje:

- usa `cpal`
- abre `default_input_device`
- captura samples para memoria
- retorna `CapturedTrack`

Limite atual:

- ainda nao existe selecao manual de dispositivo

#### Captura De System Audio

Hoje:

- usa `parec`
- detecta monitor source com `pactl`
- aceita override por `OPENVOICE_MONITOR_SOURCE`
- prefere sink default; se necessario cai para source `RUNNING`

Pipeline realtime atual:

- captura em `48kHz`, `stereo`, `s16le`
- faz downmix para mono
- faz decimation filtrada para `24kHz`
- envia chunks de cerca de `40ms`

Observacao:

- esse pipeline foi melhorado para fluidez e qualidade, mas ainda e um pipeline custom simples
- existe espaco futuro para um resampler mais sofisticado se benchmarking justificar

### `dictation`

Arquivos:

- [`src/modules/dictation/application.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/modules/dictation/application.rs)
- [`src/modules/dictation/domain.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/modules/dictation/domain.rs)
- [`src/modules/dictation/infrastructure.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/modules/dictation/infrastructure.rs)

Responsabilidade atual:

- transcricao pos-captura
- combinacao de trilhas
- clipboard final

Observacao:

- o fluxo de ditado continua importante e deve ser preservado como trilha de produto separada do realtime do desktop

#### Fluxo Atual De Captura Gravada

Hoje o fluxo gravado funciona assim:

1. `StartDictation` inicia uma sessao dual
2. o microfone abre via `cpal`
3. o audio do sistema abre no monitor source do sink padrao
4. `StopDictation` finaliza as duas trilhas
5. a sessao salva artefatos locais
6. o modulo `dictation` transcreve as duas trilhas em sequencia
7. o app salva `transcripts.json` na pasta da sessao
8. o clipboard recebe um texto combinado por trilha

Arquivos esperados na captura gravada:

- `mic.wav`
- `system.wav`
- `metadata.json`
- `transcripts.json`

Provider usado neste fluxo:

- `OpenRouter`

Comportamento atual:

- o audio e convertido para WAV mono em `16_000 Hz`
- o request atual envia `input_audio` para o modelo configurado em settings
- o modelo padrao hoje e `google/gemini-2.5-flash-lite:nitro`
- as transcricoes pos-captura sao executadas em sequencia
- o clipboard recebe um texto combinado com secoes separadas por trilha

### `live_transcription`

Arquivos:

- [`src/modules/live_transcription/domain.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/modules/live_transcription/domain.rs)
- [`src/modules/live_transcription/application.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/modules/live_transcription/application.rs)
- [`src/modules/live_transcription/infrastructure/openai_realtime.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/modules/live_transcription/infrastructure/openai_realtime.rs)
- [`src/modules/live_transcription/infrastructure/db.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/modules/live_transcription/infrastructure/db.rs)

Responsabilidade atual:

- montar sessao realtime
- converter settings em tuning de realtime
- conectar no OpenAI Realtime
- persistir sessoes e segmentos

#### Dominio Atual

`LiveTranscriptionConfig` hoje carrega:

- bearer token
- model
- prompt opcional
- language opcional
- noise reduction opcional
- turn detection

Eventos de runtime:

- `Connected`
- `TranscriptDelta`
- `TranscriptCompleted`
- `Warning`
- `Error`
- `Stopped`

#### Tuning Atual Do Realtime

Em [`src/modules/live_transcription/application.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/modules/live_transcription/application.rs):

- o profile vem primeiro das settings
- `OPENVOICE_REALTIME_PROFILE` hoje e fallback
- o profile define threshold, prefix padding e silence duration
- o runtime tambem monta um `prompt` interno curto por idioma/perfil

Profiles atuais:

- `caption`: mais rapido
- `balanced`: default
- `accuracy`: mais conservador

#### Transporte OpenAI Realtime

Em [`src/modules/live_transcription/infrastructure/openai_realtime.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/modules/live_transcription/infrastructure/openai_realtime.rs):

- usa websocket em `wss://api.openai.com/v1/realtime?intent=transcription`
- envia `transcription_session.update`
- envia audio com `input_audio_buffer.append`
- trabalha com timeout curto no socket
- faz polling de audio + socket no mesmo worker

Telemetria atual:

- `OPENVOICE_LOG_REALTIME_TRANSCRIPTS=1`
- `OPENVOICE_LOG_REALTIME_DELTAS=1`
- `OPENVOICE_LOG_REALTIME_METRICS=1`

Metricas atuais:

- tempo ate primeiro chunk de audio
- tempo ate primeiro delta
- tempo ate primeiro completed
- quantidade de chunks enviados
- quantidade de segmentos completos

#### Persistencia Local De Realtime

Em [`src/modules/live_transcription/infrastructure/db.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/modules/live_transcription/infrastructure/db.rs):

- banco SQLite local em `~/.local/share/openvoice/openvoice.db`
- tabela `lt_sessions`
- tabela `lt_segments`
- append incremental por `position`
- finalize da sessao com `stopped_at`

Dados persistidos hoje por sessao:

- `started_at`
- `stopped_at`
- `language`
- `model`
- `segment_count`

Dados persistidos hoje por segmento:

- `session_id`
- `position`
- `item_id` atualmente vazio
- `transcript`
- `completed_at`

Limites atuais:

- ainda nao persistimos deltas separadamente
- ainda nao existe indice/search por conteudo alem do que o banco simples suporta
- ainda nao existe resumo salvo por sessao
- ainda nao existe titulo de sessao

#### Fluxo Atual De Realtime

Hoje o fluxo realtime funciona assim:

1. o usuario inicia a sessao realtime pelo HUD
2. o app abre um stream continuo do audio do sistema
3. o audio e enviado para `wss://api.openai.com/v1/realtime?intent=transcription`
4. a sessao envia `transcription_session.update`
5. a API devolve eventos incrementais e transcricoes finais
6. a subtitle window mostra parcial e consolidado
7. a sessao e persistida incrementalmente no SQLite local

Provider usado neste fluxo:

- `OpenAI API key`

Observacoes:

- o realtime nao depende do login OAuth do ChatGPT
- o realtime hoje usa somente o audio do sistema
- o idioma pode ficar em `Auto` ou ser fixado em settings

Modelo padrao hoje:

- `gpt-4o-transcribe`

Modelos aceitos hoje pelo app:

- `whisper-1`
- `gpt-4o-transcribe`
- `gpt-4o-mini-transcribe`
- `gpt-4o-mini-transcribe-2025-03-20`
- `gpt-4o-mini-transcribe-2025-12-15`

Idiomas expostos na UI hoje:

- `Auto`
- `Portuguese`
- `English`
- `German`
- `Spanish`
- `French`
- `Italian`
- `Japanese`

### `auth`

Responsabilidade atual:

- fluxo OAuth com OpenAI / ChatGPT para capacidades futuras
- snapshot de autenticacao
- logout e persistencia da sessao

Observacao:

- realtime atual nao depende desse OAuth
- realtime usa `OpenAI API key`

## UI Atual

### HUD

A UI atual tem um HUD leve com:

- status (fase, hint, erro)
- start / stop realtime
- start / stop dictation
- abrir Home (botao casa)
- abrir Sessions tab (botao lista)
- toggle passthrough (apenas no modo HUD)

### Home / Hub

A Home e a view expandida principal. Abre morphando a janela HUD para 700x800. Possui tres abas:

#### Aba Inicio

- 3 action cards: Ouvir Desktop, Ditar, Perguntar Algo (desabilitado, badge "em breve")
- cards refletem estado ativo (label muda para "Parar Escuta" / "Parar Ditado")
- clicar em um action card fecha Home, volta ao HUD e inicia a acao
- status hints: mostra estado do realtime, ditado e configuracao de API keys
- sessoes recentes: mostra as ultimas 3 sessoes com preview e link para a aba Sessoes

#### Aba Sessoes

- lista de sessoes salvas com data, idioma, modelo e contagem de segmentos
- busca client-side por texto (filtra por data, idioma, modelo e preview)
- expandir detalhe com transcript completo
- copiar transcript

#### Aba Configuracoes

- configurar OpenRouter API key e modelo
- configurar OpenAI Realtime API key e modelo
- escolher idioma e profile do realtime
- gerenciar OAuth OpenAI para fluxos futuros

### Settings (legado)

Settings nao e mais uma tela separada. Vive dentro da aba Configuracoes na Home.

### Sessions (legado)

Sessions nao e mais uma janela separada. Vive dentro da aba Sessoes na Home.

### Subtitle Window

Hoje existe uma subtitle window para o realtime:

- exibe segmentos completos recentes
- exibe transcript parcial em andamento
- faz fade-out apos encerrar
- o transcript realtime nao fica renderizado no HUD principal

## Contratos Importantes No State Atual

Em [`src/app/state.rs`](/home/israel/projetos/projetos-opensource/openvoice/src/app/state.rs), o estado central hoje já modela:

- id da janela principal (HUD/Home compartilhada)
- id da subtitle window
- `MainView` enum (Hud ou Home)
- `HomeTab` enum (Home, Sessions, Settings)
- settings carregadas
- estado de auth
- recorder de dictation
- sessao realtime ativa
- transcript parcial
- segmentos completos
- estado da persistencia incremental local
- lista de sessoes com busca client-side
- sessao selecionada

Isso significa que o app atual ja esta parcialmente preparado para:

- hub mais rico
- detalhe de sessao
- chat contextual atrelado a uma sessao

Mas ainda nao esta preparado para:

- navegacao mais sofisticada
- view model separada por superficie
- chat state proprio
- multimodalidade

## Restricoes Tecnicas Atuais

- Wayland puro pode limitar `always-on-top`, posicionamento e passthrough
- system audio depende de `pactl` e `parec`
- o monitor source correto nem sempre e o sink default do desktop
- parte da ergonomia do overlay depende de comportamento do window manager
- o update loop ainda carrega bastante responsabilidade centralizada

## Persistencia Atual

### Captura Gravada

As sessoes gravadas ficam em:

- `~/.local/share/openvoice/sessions/<session-id>/`
- ou `XDG_DATA_HOME/openvoice/sessions/<session-id>/`

### Configuracoes

As configuracoes do app ficam em:

- `~/.config/openvoice/settings.json`
- ou `XDG_CONFIG_HOME/openvoice/settings.json`

### Auth

Se o usuario fizer login OAuth do ChatGPT, a sessao pode ser persistida em:

- keyring do sistema, quando disponivel
- `~/.config/openvoice/auth.json` como fallback

## Compatibilidade Linux Atual

O app hoje assume um ambiente com:

- PulseAudio
- ou PipeWire com compatibilidade `pipewire-pulse`

Para system audio:

- `pactl` descobre o sink padrao
- `parec` le o monitor source correspondente

Para microfone:

- `cpal`

## Decisoes Atuais

- falha ao iniciar qualquer uma das duas fontes aborta a captura gravada
- o app nao mistura `mic.wav` e `system.wav`
- o realtime hoje usa somente o audio do sistema
- logs verbosos de realtime ficam desligados por padrao
- `OPENVOICE_LOG_REALTIME_TRANSCRIPTS=1` liga logs de transcricao final no terminal

## Limitacoes Conhecidas

- `system.wav` grava o mix do sink padrao, nao audio por aplicativo
- alguns setups podem incluir eco, retorno ou a propria voz reproduzida pela app de reuniao
- pode haver pequeno drift entre `mic.wav` e `system.wav` em sessoes longas
- trocar o sink padrao durante a gravacao nao reconfigura automaticamente a captura em andamento
- o backend atual nao faz diarizacao real; a separacao vem da origem da trilha
- a qualidade do realtime ainda depende do idioma configurado, do VAD e da qualidade do audio do sistema
- OAuth ja existe como base de autenticacao, mas nao e usado pelo realtime atual

## Estrutura Atual

```text
src/
├── app/
├── ui/
│   ├── home.rs         # Home/Hub view com 3 abas
│   ├── overlay.rs      # HUD floating bar
│   ├── sessions.rs     # Aba Sessoes (tab_content)
│   ├── settings.rs     # Aba Configuracoes (tab_content)
│   ├── subtitle.rs     # Subtitle window flutuante
│   └── components/
├── platform/
└── modules/
    ├── audio/
    ├── auth/
    ├── dictation/
    ├── live_transcription/
    └── settings/
```

## Lacunas Atuais

- falta chat contextual / ask surface
- falta session workspace completo
- falta resumo por sessao
- falta titulo por sessao
- falta captura visual
- falta integracao funcional com Obsidian
- faltam hotkeys globais e key mapping
- falta separar melhor shell de app e estado de features futuras

## Checklist Manual Atual

### Captura Gravada

- abrir um video no YouTube e confirmar que `system.wav` recebe audio
- falar no microfone e confirmar que `mic.wav` recebe a fala local
- validar que `metadata.json` e `transcripts.json` sao gerados
- confirmar que o clipboard final contem secoes para `System audio` e `My voice`

### Realtime

- salvar uma `OpenAI API key` nas settings
- escolher um modelo suportado, como `gpt-4o-transcribe`
- opcionalmente fixar `Portuguese` para melhorar PT-BR
- rodar com `OPENVOICE_LOG_REALTIME_TRANSCRIPTS=1` para inspecionar as transcricoes finais no terminal

## Observacao

Este documento deve ser atualizado sempre que o estado implementado mudar de forma relevante.
