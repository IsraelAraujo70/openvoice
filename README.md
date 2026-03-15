# OpenVoice

Spike nativo em Rust + Iced para testar overlay transparente no Linux
com foco atual em Hyprland/Wayland.

O build agora prioriza `wayland` no `Iced`, com foco operacional em Hyprland.
No backend Wayland do `winit`, `always-on-top` continua nao sendo garantido, entao
comportamentos de overlay precisam ser tratados como responsabilidade do compositor.

Para stealth em screen share no Hyprland, o app agora tenta usar um plugin local do compositor
compilado a partir deste repo. Quando o plugin nao esta disponivel, o OpenVoice volta para o
fallback oficial do Hyprland com `no_screen_share`, que protege a captura desenhando retangulos
pretos.

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
- tentativa de HUD flutuante
- HUD com `Start Recording` / `Stop Recording`
- captura do microfone padrao
- envio de WAV em base64 para o OpenRouter
- transcricao copiada para clipboard e primary selection
- tentativa de `click-through` via `iced::window::enable_mouse_passthrough`

## Limitações

- seleção de monitor ainda não existe neste spike
- o comportamento final de passthrough ainda depende do compositor no Linux
- o app esta priorizando Hyprland/Wayland, nao multiplos backends Linux ao mesmo tempo
- `always-on-top` no Wayland nao e garantido por `winit`; integracao com Hyprland continua necessaria
- no Hyprland, rules antigas de `windowrule no_screen_share` podem continuar ativas na sessao atual ate um `hyprctl reload`
