# Audio Capture MVP

## Objetivo

O MVP de audio do OpenVoice grava duas trilhas separadas por sessao:

- `mic.wav` para a fala local
- `system.wav` para o audio reproduzido no desktop

O app nao mistura as duas fontes no momento. A separacao e intencional para permitir transcricao independente e diferenciacao futura entre a voz local e a fala da reuniao.

## Arquitetura

```text
src/modules/audio/
├── application.rs
├── domain.rs
└── infrastructure/
    ├── microphone.rs
    ├── storage.rs
    └── system.rs
```

- `application.rs`: inicia e finaliza a sessao dual
- `domain.rs`: tipos de sessao, trilha, metadata e artefatos
- `infrastructure/microphone.rs`: captura o microfone com `cpal`
- `infrastructure/system.rs`: captura o audio do desktop via `pactl` + `parec`
- `infrastructure/storage.rs`: persiste WAVs e `metadata.json`

## Fluxo atual

1. `StartDictation` inicia uma sessao dual
2. o microfone abre via `cpal`
3. o audio do sistema abre no monitor source do sink padrao
4. `StopDictation` finaliza as duas trilhas
5. a sessao salva:
   - `mic.wav`
   - `system.wav`
   - `metadata.json`
6. o modulo `dictation` transcreve as duas trilhas separadamente
7. o app salva `transcripts.json` na mesma pasta da sessao

## Persistencia local

As sessoes ficam em:

- `~/.local/share/openvoice/sessions/<session-id>/` por padrao
- ou `XDG_DATA_HOME/openvoice/sessions/<session-id>/`

Arquivos esperados:

- `mic.wav`
- `system.wav`
- `metadata.json`
- `transcripts.json`

## Compatibilidade Linux

O MVP assume um ambiente com:

- PulseAudio, ou
- PipeWire com compatibilidade `pipewire-pulse`

Para a trilha do sistema, o app depende de:

- `pactl` para descobrir o sink padrao
- `parec` para ler o monitor source correspondente

## Limitacoes conhecidas

- o `system.wav` grava o mix do sink padrao, nao audio por aplicativo
- alguns setups podem incluir eco, retorno ou a propria voz reproduzida pela app de reuniao
- pode haver pequeno drift entre `mic.wav` e `system.wav` em sessoes longas
- trocar o sink padrao durante a gravacao nao reconfigura automaticamente a captura em andamento
- o backend atual nao faz diarizacao real; a separacao vem da origem da trilha

## Decisoes do MVP

- falha ao iniciar qualquer uma das duas fontes aborta a sessao
- as transcricoes sao executadas em sequencia
- o clipboard recebe um texto combinado com secoes separadas por trilha
- a UI mostra status resumido para nao poluir o HUD

## Proximos passos

- adicionar backend PipeWire nativo
- permitir selecao manual de fonte de system audio
- armazenar observacoes de drift e alinhamento por trilha
- integrar o resultado dual com o modulo de Obsidian

## Checklist manual

- abrir um video no YouTube e confirmar que `system.wav` recebe audio
- falar no microfone e confirmar que `mic.wav` recebe a fala local
- validar que `metadata.json` e `transcripts.json` sao gerados
- confirmar que o clipboard final contem secoes para `System audio` e `My voice`
