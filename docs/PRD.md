# PRD

## Produto

OpenVoice e um copiloto pessoal Linux-first para reunioes, raciocinio e captura de contexto.

O produto deve combinar:

- escuta do desktop em tempo real
- transcricao e traducao futura em streaming
- ditado da propria fala
- copiloto conversacional com contexto da sessao atual
- entrada multimodal com imagem e, no futuro, captura de tela
- memoria persistente conectada ao Obsidian

O objetivo nao e ser apenas um transcritor.

O objetivo e criar um assistente sempre acessivel que:

- entende o que esta acontecendo agora
- responde rapido a perguntas do usuario
- ajuda o usuario a falar, pensar e decidir melhor
- transforma conversa em conhecimento reutilizavel

## Tese De Produto

Ferramentas atuais normalmente se dividem em silos:

- uma para transcrever
- outra para resumir
- outra para perguntar para uma LLM
- outra para consultar notas
- outra para capturar contexto visual

OpenVoice deve unir esses fluxos numa experiencia unica.

O diferencial do produto e:

- ouvir o desktop em realtime por padrao
- manter ditado como trilha separada e valiosa
- permitir perguntar para a IA sobre o que acabou de acontecer
- aceitar imagem e contexto visual como parte da interacao
- evoluir para responder com base no vault do Obsidian

## Problema

Quem trabalha em Linux e participa de reunioes, consome conteudo tecnico ou organiza ideias enfrenta um fluxo fragmentado:

- o contexto do que esta sendo dito se perde rapido
- perguntar para uma LLM exige copiar, colar e reexplicar contexto
- audio, notas e acoes ficam separados
- insights do momento raramente viram conhecimento persistente
- ferramentas de assistencia costumam ser pouco integradas ao desktop Linux

Resultado:

- mais carga mental
- retrabalho
- respostas lentas
- notas pobres
- baixa continuidade entre sessao atual e historico pessoal

## Solucoes Que O Produto Deve Oferecer

### 1. Entender o que esta acontecendo agora

- ouvir o desktop em realtime
- transcrever com fluidez suficiente para parecer legenda
- no futuro, traduzir em tempo real

### 2. Permitir interacao imediata com IA

- abrir um chat rapido sem sair do contexto
- permitir perguntar sobre a fala atual
- permitir enviar imagem para pedir ajuda contextual
- no futuro, permitir captura de tela como insumo imediato

### 3. Ajudar o usuario a responder melhor

- sugerir resposta
- resumir rapidamente o que foi dito
- apontar proximos passos
- no futuro, atuar como copiloto de reuniao

### 4. Preservar e reutilizar contexto

- salvar sessoes
- permitir revisitar transcript e resumo
- conectar a memoria do usuario no Obsidian
- usar historico e notas como base de resposta

### 5. Reduzir friccao operacional

- HUD rapido
- acoes com hotkeys
- historico pesquisavel
- workspace de sessao claro

## Usuario-Alvo

Principal:

- usuarios Linux power users
- devs, PMs, founders, researchers e criadores
- usuarios de Obsidian
- pessoas que trabalham com reunioes, conteudo e pensamento continuo

Secundario:

- usuarios de ditado continuo
- usuarios que querem contexto ao vivo sem depender de copiar e colar tudo

## Casos De Uso Principais

### Reuniao

O usuario quer:

- ouvir e transcrever o que esta acontecendo
- perguntar para a IA "o que decidiram agora?"
- pedir ajuda sobre o que responder
- salvar o contexto da sessao

### Conteudo tecnico

O usuario quer:

- assistir um video
- deixar a IA ouvindo o desktop
- perguntar sobre um trecho que acabou de passar
- mandar uma imagem do slide ou da tela para esclarecer algo

### Ditado e raciocinio

O usuario quer:

- falar ideias rapidamente
- capturar o proprio pensamento
- transformar isso em texto
- organizar depois no vault

### Consulta de memoria pessoal

O usuario quer:

- perguntar algo para a IA
- receber resposta influenciada pelo historico salvo
- cruzar sessao atual com notas antigas do Obsidian

## Diferenciacao De Produto

O competidor trata "IA ouvindo" e "perguntar para IA" como botoes separados.

OpenVoice deve seguir uma abordagem mais integrada:

- a escuta realtime do desktop e capacidade nativa do produto
- o chat nasce em cima da sessao atual, nao como caixa vazia
- ditado continua como feature independente e forte
- imagem e captura visual entram como extensao natural do copiloto
- Obsidian funciona como memoria real, nao so export final

## Superficies Do Produto

### 1. Floating Bar

Barra minima sempre acessivel para:

- abrir home
- iniciar / parar realtime
- iniciar / parar ditado ou captura
- abrir ask box
- mostrar hotkeys principais

### 2. Home / Hub

Area principal de produto para:

- busca global
- sessoes recentes
- features principais
- status de providers e auth
- acesso a settings e key mapping

### 3. Session Workspace

Tela dedicada para uma sessao:

- transcript
- resumo
- metadata
- acao de continuar sessao
- acao de escrever resumo
- acao de exportar
- chat contextual da sessao

### 4. Ask / Chat Surface

Superficie de chat rapido para:

- perguntar sobre o que esta sendo dito agora
- perguntar sobre uma sessao salva
- mandar imagem
- no futuro, anexar captura de tela

Essa superficie pode existir em dois modos:

- compacta, como painel rapido
- expandida, como workspace de copiloto

## Capabilities De Produto

### Realtime Listening

- ouvir desktop em streaming
- transcrever em tempo real
- traduzir em tempo real no futuro
- suportar perfis de latencia vs precisao

### Dictation

- capturar a fala do proprio usuario
- transformar em texto
- manter esse fluxo independente do realtime do desktop

### AI Chat

- perguntar algo manualmente
- receber resposta contextual
- usar sessao atual como contexto implicito
- permitir envio de imagem

### Meeting Copilot

- responder perguntas sobre a conversa atual
- resumir
- sugerir o que falar
- destacar decisoes e proximos passos

### Session Intelligence

- nomear sessao
- resumir sessao
- pesquisar sessao
- continuar sessao

### Personal Memory

- integrar com Obsidian
- recuperar notas relacionadas
- responder com base em memoria pessoal

### Visual Context

- aceitar imagem enviada manualmente
- evoluir para captura de tela
- usar isso como contexto adicional para o copiloto

### Shortcuts And Key Mapping

- expor hotkeys principais
- permitir key mapping configuravel
- tornar atalhos parte da UX principal

## Principios De UX

- rapido para abrir
- rapido para perguntar
- rapido para voltar ao contexto
- chat e transcript devem coexistir sem competir
- settings nao devem dominar a experiencia
- a sessao deve ter identidade propria
- o produto deve parecer um assistente vivo, nao um painel tecnico

## Estrategia De Navegacao

- `floating bar` para agir agora
- `home / hub` para navegar e descobrir
- `session workspace` para mergulhar em contexto
- `ask / chat surface` para interagir com IA rapidamente

## Roadmap De Produto

### Fase 1. Shell E Fluidez

- floating bar forte
- transcricao realtime fluida
- ditado preservado
- atalhos visiveis

### Fase 2. Hub E Historico

- home / hub
- sessoes recentes
- historico pesquisavel
- detalhe de sessao

### Fase 3. Chat E Workspace

- ask box
- chat contextual da sessao
- continuar sessao
- escrever resumo
- respostas rapidas sobre a fala atual

### Fase 4. Multimodalidade

- envio de imagem
- captura de tela
- copiloto usando contexto visual

### Fase 5. Memoria E Copiloto Avancado

- integracao com Obsidian
- respostas baseadas em notas
- sugestoes do que falar
- suporte a traducao realtime
- key mapping configuravel

## Requisitos Funcionais

### Floating Bar

- deve ser minima e sempre acessivel
- deve permitir iniciar e parar os fluxos principais
- deve permitir abrir home e chat

### Chat

- o usuario deve poder perguntar a uma LLM rapidamente
- o usuario deve poder enviar imagem como entrada
- a resposta deve aparecer numa superficie de chat clara e imediata
- o chat deve poder usar a sessao atual como contexto

### Realtime

- o produto deve escutar o desktop em streaming
- deve transcrever em tempo real
- deve evoluir para traducao futura
- deve permitir operar como camada de legenda

### Ditado

- o produto deve manter o fluxo de transcrever a propria fala
- o fluxo de ditado nao deve ser abandonado em favor do realtime do desktop

### Historico

- o usuario deve conseguir localizar sessoes
- o usuario deve conseguir abrir uma sessao como workspace
- o usuario deve conseguir continuar o contexto de uma sessao anterior

### Resumo E Acoes

- o usuario deve poder pedir resumo
- o usuario deve poder copiar e exportar
- o produto deve poder destacar insights, decisoes e proximos passos

### Obsidian

- o produto deve usar o vault como memoria futura
- o produto deve poder responder com base nas notas
- o produto deve poder salvar resultado de volta no vault

### Visual Context

- o produto deve aceitar imagem como input
- o produto deve evoluir para captura de tela

### Shortcuts

- o produto deve exibir hotkeys principais
- o produto deve evoluir para key mapping configuravel

## Requisitos Nao Funcionais

- startup rapido
- latencia baixa para interacao
- fluidez de UX
- arquitetura modular
- codigo legivel
- Linux first

## Fora De Escopo Por Enquanto

- colaboracao multiusuario
- analytics
- automacoes profundas de calendario
- suporte amplo irrestrito a todos os desktops Linux desde o primeiro momento

## Criterios De Sucesso

- o usuario consegue iniciar uma sessao realtime com baixa friccao
- o transcript parece util em tempo real
- o usuario consegue fazer uma pergunta para a IA sem sair do contexto
- o usuario consegue abrir, buscar e retomar sessoes
- o produto preserva caminho claro para copiloto, multimodalidade e memoria no Obsidian
