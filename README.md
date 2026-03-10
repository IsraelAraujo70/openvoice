# OpenVoice

Spike nativo em Rust + Iced para testar overlay fullscreen transparente no Linux
com tentativa de mouse passthrough.

O build atual foi fixado em `x11` para rodar via XWayland no GNOME/Wayland,
porque o backend Wayland do `winit` nao implementa `always-on-top`.

## Rodar

Cadastre a OpenRouter API key direto no HUD e clique em `Save Settings`.
O app salva a chave e o modelo em `~/.config/openvoice/settings.json`.

```bash
cargo run
```

Para iniciar sem passthrough e manter a janela interativa:

```bash
OPENVOICE_MOUSE_PASSTHROUGH=0 cargo run
```

## Controles

- `P`: alterna mouse passthrough enquanto a janela ainda tem foco
- `Esc`: fecha a janela enquanto ela ainda tem foco

## Estado atual

- janela fullscreen
- transparente
- sem decorations
- always-on-top
- HUD com `Start Recording` / `Stop Recording`
- captura do microfone padrao
- envio de WAV em base64 para o OpenRouter
- transcricao copiada para clipboard e primary selection
- tentativa de `click-through` via `iced::window::enable_mouse_passthrough`

## Limitações

- seleção de monitor ainda não existe neste spike
- o comportamento final de passthrough ainda depende do compositor no Linux
- o app esta priorizando `x11`/XWayland, nao o backend Wayland nativo
