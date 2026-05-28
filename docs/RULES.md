## Regras do Projeto

### Base

- Manter um repositorio Git local desde o inicio para facilitar restauracao e
  seguranca.
- Comitar toda mudanca concluida com uma mensagem clara sobre o que foi
  implementado.
- Usar mensagens de commit em ingles por padrao, com texto curto, claro e no
  imperativo.
- Manter `docs/TODO.md` sempre atualizado de forma limpa, enxuta, simples,
  facil de entender e direto ao ponto.
- Trabalhar sempre sobre uma base limpa, nova, funcional e sem
  retrocompatibilidade.
- Nao adicionar codigo, dependencias, logs, testes, arquivos ou configuracoes
  sem necessidade real.
- Nunca versionar informacoes confidenciais, como `.env`, chaves, tokens ou
  dados privados.
- Nada de testes extras, preferir testar em runtime.
- O projeto deve ser direto ao ponto, sem enrolacao, sem desvios e implementar funções reais e funcionais.

### Codigo

- Usar boas praticas de programacao, C++ moderno e o recurso mais recente,
  seguro e estavel que o toolchain suportar, priorizando C++26 quando
  disponivel.
- Manter o codigo limpo, enxuto, semantico, modular, robusto, otimizado e facil
  de evoluir.
- Aplicar Single Responsibility Principle e Don't Repeat Yourself em codigo,
  configuracao e documentacao.
- Usar tipagem extremamente forte e nomes claros, sem ambiguidade.
- Manter projeto, codigo, funcoes, estados e fluxos previsiveis, com entradas,
  saidas, efeitos colaterais e erros explicitos.
- Evitar numeros magicos; valores fixos devem ter nome, tipo e contexto.
- Nao manter codigo morto, duplicado, obsoleto ou provisoriamente esquecido.
- Nao criar arquivos de codigo proprios com nomes em snake_case ou separadores
  artificiais; usar PascalCase sem `_`, como `ExemploExemplo.cpp`, salvo quando
  uma convencao externa obrigatoria exigir outro formato.

### Arquitetura

- Organizar o projeto em modulos pequenos, coesos e com responsabilidade clara.
- Preferir APIs pequenas, abstracoes simples e contratos explicitos.
- Evitar comportamento implicito, fallback silencioso ou caminhos alternativos
  que escondam falhas reais do sistema.
- Nao esconder falhas de inicializacao com fallback silencioso.
- Facilitar manutencao, atualizacao e adicao de novas funcoes sem espalhar
  regras de negocio.
- Criar menus interativos limpos, diretos e funcionais, sem configuracoes,
  controles, textos ou estados extras que nao tenham uso real e imediato.
- Simplificar continuamente partes separadas demais, redundantes ou dificeis de
  atualizar.
- Otimizar sem sacrificar clareza, seguranca ou manutencao.

### Qualidade

- Verificar continuamente riscos de memory leaks, lifetime incorreto e uso
  inseguro de recursos.
- Preferir testes em runtime live, logs e smoke tests somente quando agregarem
  valor real; evitar excesso e ruido.
- Priorizar validacao em runtime live como fonte principal de confianca.
- Nao criar nem rodar testes extras quando a mudanca puder ser validada de
  forma direta no runtime.
- Atualizar dependencias, configuracoes e praticas de seguranca sempre que
  houver uma opcao moderna, segura e compativel com a base atual.
- Antes de finalizar uma mudanca, revisar duplicacoes, codigo morto,
  ambiguidade, seguranca e impacto de manutencao.
