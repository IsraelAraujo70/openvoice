# TODO

## Objetivo

Este documento concentra o backlog tecnico do projeto.

Ele responde:

- o que ainda precisa ser implementado
- quais lacunas do estado atual ainda existem
- como agrupar o trabalho por area

O `spec.md` descreve o que existe hoje.
O `PRD.md` descreve a direcao de produto.
O `TODO.md` descreve o trabalho pendente.

## Navegacao E Superficies

- criar `home / hub`
- transformar `sessions` em explorador + detalhe de sessao
- tirar `settings` do papel de tela principal do produto
- introduzir `ask / chat surface`

## Sessao

- adicionar titulo de sessao
- adicionar resumo por sessao
- adicionar busca por transcript e metadata
- adicionar acao `continuar sessao`

## Chat E Copiloto

- criar estado dedicado de chat
- permitir perguntar usando contexto da sessao atual
- permitir abrir chat sem depender de sessao salva
- desenhar como respostas da LLM aparecem sem competir com subtitles

## Multimodalidade

- aceitar upload de imagem
- definir pipeline de captura de tela
- modelar contexto visual no dominio de copiloto

## Obsidian

- definir contrato de leitura do vault
- definir estrategia de retrieval sobre notas
- definir formato de export de sessoes, resumos e proximos passos

## Realtime

- persistir `delta` se benchmark justificar
- considerar busca mais robusta no historico
- decidir se `item_id` deve ser persistido de verdade
- avaliar se o worker realtime deve evoluir para arquitetura mais separada no futuro

## Audio

- avaliar resampler mais sofisticado se benchmark mostrar ganho real
- adicionar selecao manual de source / sink
- evoluir diagnostico de audio do sistema

## Shortcuts

- implementar hotkeys globais
- criar key mapping persistido
- expor atalhos no produto
