# Audio Capture MVP

## Objetivo

O MVP de audio do OpenVoice hoje cobre dois fluxos diferentes:

- captura gravada de duas trilhas por sessao
- transcricao realtime do audio do sistema

Na captura gravada, o app salva:

- `mic.wav` para a fala local
- `system.wav` para o audio reproduzido no desktop

O app nao mistura as duas fontes no momento. A separacao e intencional para permitir transcricao independente, diferenciar contexto da reuniao da fala local e manter a base pronta para evolucoes futuras.

## Modulos envolvidos

```text
src/modules/
├── audio/
│   ├── application.rs
│   ├── domain.rs
│   └── infrastructure/
│       ├── microphone.rs
│       ├── storage.rs
│       └── system.rs
├── dictation/
│   ├── application.rs
│   ├── domain.rs
│   └── infrastructure.rs
├── live_transcription/
│   ├── application.rs
│   ├── domain.rs
│   └── infrastructure/
│       └── openai_realtime.rs
└── settings/
    ├── application.rs
    ├── domain.rs
    └── infrastructure.rs
```

- `audio`: abre, encerra e persiste a sessao dual
- `dictation`: prepara WAV, faz transcricao pos-captura e salva `transcripts.json`
- `live_transcription`: faz streaming do audio do sistema para o OpenAI Realtime
- `settings`: persiste API keys, modelos e idioma do realtime

## Fluxo 1: captura gravada em duas trilhas

1. `StartDictation` inicia uma sessao dual
2. o microfone abre via `cpal`
3. o audio do sistema abre no monitor source do sink padrao
4. `StopDictation` finaliza as duas trilhas
5. a sessao salva:
   - `mic.wav`
   - `system.wav`
   - `metadata.json`
6. o modulo `dictation` transcreve as duas trilhas em sequencia
7. o app salva `transcripts.json` na mesma pasta da sessao
8. o clipboard recebe um texto combinado com secoes separadas por trilha

### Provider usado neste fluxo

- a transcricao pos-captura usa `OpenRouter`
- o audio e convertido para WAV mono em `16_000 Hz`
- o request atual envia `input_audio` para o modelo configurado em settings

O modelo padrao hoje e:

- `google/gemini-2.5-flash-lite:nitro`

## Fluxo 2: transcricao realtime do system audio

1. o usuario inicia a sessao realtime pelo HUD
2. o app abre um stream continuo do audio do sistema
3. o audio e enviado para `wss://api.openai.com/v1/realtime?intent=transcription`
4. a sessao envia `transcription_session.update` com:
   - modelo de transcricao
   - idioma opcional
   - `server_vad`
   - reducao de ruido `far_field`
5. a API devolve eventos incrementais e transcricoes finais

### Provider usado neste fluxo

- o realtime usa `OpenAI API key`
- o realtime nao depende do login OAuth do ChatGPT
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

## Persistencia local

As sessoes gravadas ficam em:

- `~/.local/share/openvoice/sessions/<session-id>/` por padrao
- ou `XDG_DATA_HOME/openvoice/sessions/<session-id>/`

Arquivos esperados na captura gravada:

- `mic.wav`
- `system.wav`
- `metadata.json`
- `transcripts.json`

As configuracoes do app ficam em:

- `~/.config/openvoice/settings.json`
- ou `XDG_CONFIG_HOME/openvoice/settings.json`

Se o usuario fizer login OAuth do ChatGPT, a sessao pode ser persistida em:

- keyring do sistema, quando disponivel
- `~/.config/openvoice/auth.json` como fallback

## Compatibilidade Linux

O MVP assume um ambiente com:

- PulseAudio, ou
- PipeWire com compatibilidade `pipewire-pulse`

Para a trilha do sistema, o app depende de:

- `pactl` para descobrir o sink padrao
- `parec` para ler o monitor source correspondente

Para o microfone gravado, o app usa:

- `cpal`

## Decisoes atuais

- falha ao iniciar qualquer uma das duas fontes aborta a captura gravada
- as transcricoes pos-captura sao executadas em sequencia
- o clipboard recebe um texto combinado com secoes `System audio` e `My voice`
- o realtime hoje usa somente o audio do sistema
- o transcript realtime nao fica renderizado no HUD por enquanto
- logs verbosos de realtime ficam desligados por padrao
- `OPENVOICE_LOG_REALTIME_TRANSCRIPTS=1` liga apenas logs de transcricao final no terminal

## Limitacoes conhecidas

- `system.wav` grava o mix do sink padrao, nao audio por aplicativo
- alguns setups podem incluir eco, retorno ou a propria voz reproduzida pela app de reuniao
- pode haver pequeno drift entre `mic.wav` e `system.wav` em sessoes longas
- trocar o sink padrao durante a gravacao nao reconfigura automaticamente a captura em andamento
- o backend atual nao faz diarizacao real; a separacao vem da origem da trilha
- a qualidade do realtime ainda depende bastante do idioma configurado, do VAD e da qualidade do audio do sistema
- OAuth ja existe como base de autenticacao, mas nao e usado pelo realtime atual

## Proximos passos

- adicionar backend PipeWire nativo
- permitir selecao manual de fonte de system audio
- criar uma area dedicada de UI para o transcript realtime
- armazenar observacoes de drift e alinhamento por trilha
- integrar a saida dual com o modulo de Obsidian
- decidir quando o fluxo OAuth entra de fato nos modelos de escrita

## Checklist manual

### Captura gravada

- abrir um video no YouTube e confirmar que `system.wav` recebe audio
- falar no microfone e confirmar que `mic.wav` recebe a fala local
- validar que `metadata.json` e `transcripts.json` sao gerados
- confirmar que o clipboard final contem secoes para `System audio` e `My voice`

### Realtime

- salvar uma `OpenAI API key` nas settings
- escolher um modelo suportado, como `gpt-4o-transcribe`
- opcionalmente fixar `Portuguese` para melhorar PT-BR
- rodar com `OPENVOICE_LOG_REALTIME_TRANSCRIPTS=1` para inspecionar as transcricoes finais no terminal
