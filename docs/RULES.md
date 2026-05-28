# Constituição do Projeto

## Regra Suprema

O Codex / IA deve seguir todas as regras do projeto rigorosamente e sem exceção.

Nenhuma implementação, otimização, abstração, refatoração, dependência, atalho ou decisão arquitetural pode violar as regras definidas neste documento.

Quando existir conflito, a ordem de prioridade deve ser sempre:

1. Regras do projeto
2. Integridade da arquitetura
3. Manutenibilidade
4. Previsibilidade
5. Segurança
6. Performance
7. Velocidade de desenvolvimento

Velocidade nunca justifica degradação arquitetural.

---

## Regras Modulares Prioritárias

A modularidade existe em dois níveis obrigatórios e ambos possuem prioridade muito alta.

### Pastas e arquivos

* A organização modular de pastas e arquivos é obrigatória.
* Nenhum código novo deve ser criado em pastas genéricas ou acumuladas sem responsabilidade clara.
* A estrutura de diretórios deve refletir módulos reais, pequenos, coesos e previsíveis.
* Cada pasta deve possuir propósito explícito e conter apenas arquivos diretamente relacionados.
* Quando a estrutura estiver ambígua, a modularização deve vir antes de novas funcionalidades.

### Código

* O código interno de cada módulo também deve ser modular, coeso e explícito.
* Cada classe, função, estado e contrato deve possuir responsabilidade única e clara.
* Nenhuma implementação deve concentrar múltiplas responsabilidades por conveniência.
* Quando uma função, classe ou arquivo começar a acumular regras diferentes, a separação modular deve vir antes de novas funcionalidades.

---

# Filosofia Central

O projeto deve permanecer:

* Limpo
* Previsível
* Minimalista
* Modular
* Explícito
* Fácil de manter
* Escalável
* Pronto para produção

A base do código deve evoluir continuamente em direção à simplicidade, nunca à complexidade.

Toda implementação deve resolver problemas reais usando a menor complexidade necessária.

Evitar:

* Overengineering
* Abstrações prematuras
* Soluções temporárias
* Comportamentos ocultos
* Fluxos implícitos
* Caos defensivo
* Inconsistência arquitetural

O sistema deve continuar compreensível meses depois sem depender de contexto histórico.

---

# Regras do Repositório

* Manter um repositório Git local desde o início.
* Comitar toda mudança lógica concluída e validada.
* Mensagens de commit devem:

  * estar em inglês
  * ser curtas
  * usar verbo no imperativo
  * descrever claramente mudanças reais
* Manter `docs/TODO.md` minimalista, atualizado e acionável.
* Nunca versionar segredos, credenciais, tokens ou dados privados.
* Evitar arquivos, dependências, assets, logs ou ferramentas sem necessidade real.
* Preferir uma base limpa e funcional ao invés de preservar compatibilidade obsoleta.
* Evitar acumular dívida técnica intencionalmente.

---

# Regras de Arquitetura

## Estrutura

* Organizar sistemas em módulos pequenos e coesos.
* Cada módulo deve possuir responsabilidade clara.
* Preferir composição ao invés de herança.
* Preferir contratos explícitos ao invés de comportamento implícito.
* Preferir fluxos determinísticos ao invés de “mágica dinâmica”.
* Preferir abstrações simples ao invés de camadas profundas de abstração.
* Evitar objetos gigantes e acúmulo centralizado de lógica.
* Regras de negócio não devem se espalhar de forma imprevisível.

## Design

* APIs devem permanecer pequenas e explícitas.
* Entradas, saídas, efeitos colaterais e falhas devem ser sempre visíveis.
* Nenhum fallback silencioso.
* Nenhum caminho oculto de recuperação de inicialização.
* Nenhuma falsa resiliência escondendo falhas reais.
* Erros devem aparecer de forma clara e previsível.
* Transições de estado devem ser rastreáveis.

## Evolução

* Sistemas devem ser preparados para expansão futura sem reescritas destrutivas.
* Refatorações devem simplificar o projeto, não reorganizar complexidade.
* Reduzir fragmentação sempre que possível.
* Remover continuamente código morto, obsoleto ou duplicado.

---

# Regras de Código

## Estilo

* Usar Rust moderno no backend Tauri e TypeScript moderno no frontend.
* Usar APIs nativas do Windows apenas em módulos isolados, explícitos e pequenos.
* Priorizar as versões modernas, seguras e estáveis suportadas pelo toolchain atual.
* Manter TypeScript em modo estrito e Rust sem warnings relevantes.
* Manter o código semântico, limpo, direto e otimizado.
* Priorizar legibilidade acima de “esperteza”.
* Usar tipagem forte sempre que possível.
* Evitar nomes ambíguos.
* Evitar indireção desnecessária.
* Evitar complexidade excessiva com generics, traits ou tipos condicionais sem justificativa.

## Nomeação

* Usar nomes claros e descritivos.
* Evitar abreviações, salvo quando universalmente conhecidas.
* Evitar prefixos e sufixos artificiais.
* Arquivos internos de TypeScript devem usar PascalCase sem separadores.
* Módulos Rust devem seguir as convenções do toolchain em snake_case.
* Snake_case em TypeScript só é permitido quando exigido externamente.

## Lógica

Aplicar:

* Single Responsibility Principle
* DRY
* Ownership explícito
* Gerenciamento explícito de lifetime

Evitar:

* Números mágicos
* Código morto
* Código comentado obsoleto
* Mutação oculta de estado
* Ownership implícito
* Manipulação insegura de recursos

Constantes devem sempre possuir:

* significado semântico
* tipo explícito
* clareza contextual

---

# Regras de Runtime e Confiabilidade

* Validação em runtime é a principal fonte de confiança.
* Preferir validação live ao invés de excesso de testes automatizados.
* Criar testes apenas quando entregarem valor real e mensurável.
* Evitar testes barulhentos, redundantes ou caros de manter.
* Logs devem existir apenas quando operacionalmente úteis.
* Evitar poluição de debug.

O sistema deve validar continuamente:

* segurança de memória
* lifetime de recursos
* correção de ownership
* ordem de inicialização
* visibilidade de falhas

---

# Regras de Performance

* Otimizar com responsabilidade.
* Nunca sacrificar manutenção por micro-otimizações.
* Evitar alocações desnecessárias.
* Evitar overhead desnecessário em runtime.
* Priorizar performance estável e previsível.
* Medir antes de otimizar agressivamente.

Performance deve ser intencional, nunca acidental.

---

# Regras de Dependências

* Toda dependência deve justificar sua existência.
* Preferir soluções internas quando a complexidade for baixa.
* Evitar excesso de dependências.
* Manter integrações externas isoladas.
* Atualizar dependências regularmente para versões modernas e seguras.

---

# Regras de UI e UX

* Interfaces devem permanecer limpas, diretas e funcionais.
* Nenhuma complexidade visual sem valor prático.
* Evitar estados, opções ou controles sem utilidade real.
* Menus e fluxos devem minimizar atrito.
* A densidade de informação deve permanecer organizada e intencional.

---

# Regras Operacionais da IA

A IA deve:

* Pensar antes de implementar.
* Preservar consistência arquitetural.
* Detectar riscos futuros de manutenção.
* Alertar violações arquiteturais antes de prosseguir.
* Evitar implementações especulativas.
* Nunca inventar APIs, sistemas ou comportamentos inexistentes.
* Evitar soluções parciais e inacabadas.
* Preferir implementações completas e funcionais.

Antes de finalizar qualquer mudança, sempre revisar:

* duplicação
* código morto
* ambiguidade
* ownership inseguro
* impacto de manutenção
* consistência arquitetural

---

# Padrões Proibidos

Evitar explicitamente:

* Abuso de Singleton
* Abuso de Service Locator
* Globais ocultos
* Dependências circulares
* Árvores profundas de herança
* Mutação de estado sem ownership claro
* God Classes
* Abuso de reflection em runtime
* Ownership implícito de recursos

---

# Regras de Decisão de Engenharia

Quando múltiplas soluções existirem, preferir sempre a que:

1. Reduz manutenção futura
2. Melhora previsibilidade
3. Reduz complexidade oculta
4. Minimiza acoplamento
5. Facilita debugging
6. Possui menos partes móveis
7. Preserva consistência arquitetural

---

# Diretiva Final

Todas as futuras instruções devem ser interpretadas através desta constituição.

Caso uma solicitação entre em conflito com estas regras, o conflito deve ser explicitamente informado antes da implementação continuar.

A integridade de longo prazo do projeto é obrigatória e inegociável.
