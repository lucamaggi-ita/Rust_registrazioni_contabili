# Rust_registrazioni_contabili

Porte em Rust de [Programma_registrazioni-contabili](https://github.com/lucamaggi-ita/Programma_registrazioni-contabili)
(agente *Fantozzi*: registro contabil de NF-e para restaurante no Brasil).

> **Nome anterior: `Rust_schede_tecniche`.** Este projeto foi iniciado por
> engano no repositorio e na pasta destinados as fichas tecnicas. Em
> 2026-10-15 repositorio, pasta e crate foram renomeados para a identidade
> contabil. O porte das fichas tecnicas e' outro produto, ainda nao iniciado,
> e tera' como fonte `programma-schede-tecniche`.

## Estado atual

| Fase | Conteudo | Estado |
|---|---|---|
| 1 | Dependencias, modelo de dados, leitura e validacao das regras YAML | **PRONTA** (revisao 2026-10-15 aplicada) |
| 2 | Leitura de NF-e (`quick-xml`), normalizacao de texto, classificacao produtos/despesas | **PRONTA** (revisao 2026-10-15 aplicada) |
| 3 | Alocacao em lancamentos, pre-livro diario, DRE gerencial, relatorios | NAO_INICIADA |

Funciona de ponta a ponta: validar a configuracao, ler uma NF-e em XML e
classificar cada item contra o plano de contas. A fase 2 **nao grava nada**:
transformar itens classificados em lancamentos e' a fase 3.

## Uso

```bash
cargo run -- validar-config                 # usa os caminhos padrao de config/
cargo run -- validar-config --estrito       # alertas tambem reprovam (uso em CI)

cargo run -- importar-nfe --xml samples/nfe_exemplo/nfe_oesa_12566_20260505.xml

cargo run -- classificar --texto "CERVEJA HEINEKEN LONG NECK 330ML"
cargo run -- classificar --tipo despesas --texto "Conta de luz Neoenergia 05/2026"

cargo run -- auditar-aliases --tipo produtos            # para onde cada alias leva
cargo run -- auditar-aliases --tipo produtos --estrito  # divergencias reprovam
```

Codigo de saida: `0` = APROVADO, `1` = REPROVADO ou erro de carregamento.
`--estrito` serve para **alertas**: violacoes de contrato sao sempre ERRO e
reprovam tambem sem o flag.

**Todos** os comandos carregam a configuracao pela mesma validacao de
`validar-config`: erro bloqueia, alerta vai para stderr. Nenhum comando
opera sobre uma configuracao que `validar-config` reprova.

## Contratos implementados

### Fase 1 - modelo e configuracao

| ID | Contrato | Onde | Teste |
|---|---|---|---|
| C1 | Valores monetarios em `Decimal`, nunca `f64` | `modelos.rs` | `decimal_nao_introduz_erro_binario` |
| C2 | Dado faltante != zero: `Dado<T>` com valor, fonte, status e motivo | `modelos.rs` | `dado_ausente_nao_vira_zero`, `dado_exige_fonte_nao_vazia` |
| C3 | `deny_unknown_fields`: chave errada no YAML = erro, nao silencio | `regras.rs`, `modelos.rs` | `c3_chave_desconhecida_no_yaml_falha` |
| C4 | `tipo` de conta e enum fechado | `modelos.rs` | `tipos_de_conta_correspondem_as_strings_do_python` |
| C5 | Plano de contas: IDs unicos, campos preenchidos, contas de revisao presentes e do tipo revisao | `regras.rs` | `c5_*` |
| C6 | Toda regra aponta para conta existente; fallback do tipo revisao | `regras.rs` | `c6_*`, `cli_validar_config.rs` |
| C7 | Proveniencia dos YAML e dos samples declarada | `config/FONTE.md`, `samples/FONTE.md` | - |
| C8 | Tipos validados tem campos privados e um unico caminho de validacao | `modelos.rs` | `competencia_round_trip_sempre_valido` |

### Fase 2 - leitura e classificacao

| ID | Contrato | Onde | Teste |
|---|---|---|---|
| C9 | A normalizacao **aplica** os flags `normalizacao:` do YAML em vez de assumi-los | `normalizacao.rs` | `flags_desligados_mudam_o_resultado` |
| C10 | **Ausente nunca e' zero.** Obrigatorio pelo layout = erro com caminho; facultativo = ausencia representada no modelo (`Dado`) com estado e motivo, nunca um numero | `nfe.rs`, `modelos.rs` | `valor_obrigatorio_ausente_nao_vira_zero`, `cobranca_*`, `round_trip_preserva_a_ausencia_do_importe` |
| C11 | Empate de prioridade escolhe a **primeira** regra, como `max()` do Python | `classificacao.rs` | `empate_de_prioridade_escolhe_a_primeira_regra` |
| C12 | `matches` e contas ambiguas ordenados: saida reproduzivel | `classificacao.rs` | `matches_sao_deterministicos_e_ordenados` |
| C13 | Secao declarada no YAML e nao aplicada pelo classificador vira alerta explicito | `regras.rs` | `validar-config` |
| C14 | O parser nao altera conteudo XML valido: entidades (`&amp;`, `&#46;`) e CDATA entram no texto | `nfe.rs` | `referencias_de_caractere_sao_resolvidas`, `cdata_entra_no_texto` |
| C15 | Nenhum comando usa configuracao que `validar-config` reprova | `main.rs` | `tests/cli_configuracao_obrigatoria.rs` |
| C16 | A auditoria de alias mede para onde o alias **leva de fato**, com baseline fixada | `classificacao.rs` | `auditoria_de_aliases_corresponde_a_baseline_declarada` |

O Python so le `regras` e `conta_fallback_id`. C13 torna visivel o resto:

- `regras_por_fornecedor` (1 regra: fornecedor com descricao generica "KILOS")
  **nao e' aplicada** na classificacao, nem aqui nem no Python — la, so
  `classification_rule_proposal.py` a le;
- `politica_de_match` declara que alias de ate 3 caracteres deveriam casar so
  por palavra inteira: **ninguem aplica isso**. Hoje `cerv` casa por
  substring, como no Python.

Decidir se essas duas regras devem passar a valer e' uma mudanca de
comportamento, nao de porte: fica fora da fase 2.

### C10 em detalhe: ausencia nao e' zero

Um valor facultativo ausente **nao** vira `0`. `Nfe::valor_cobranca` e
`Nfe::valor_tributos_inclusos` sao `Dado<Decimal>`: valor, fonte, estado e
motivo. Quatro situacoes, quatro estados distintos:

| Fonte | Estado | Valor | Saida da CLI |
|---|---|---|---|
| sem `<cobr>` | `N.A.` | nenhum | `N.A. (NF-e sem bloco <cobr>)` |
| `<cobr>` sem `<fat>` | `NAO_DISPONIVEL` | nenhum | `NAO_DISPONIVEL (<cobr> presente mas sem bloco <fat>)` |
| `<fat>` sem `<vLiq>` | `MANCANTE` | nenhum | `MANCANTE (<fat> presente mas sem <vLiq>)` |
| `<vLiq>0.00</vLiq>` | `OK` | `0.00` | `0.00` |

O zero real declarado pela fonte continua zero. O valor **nunca** e'
reconstruido somando as parcelas: sem `<fat>`, as duas parcelas de 17.50
permanecem e o faturamento fica ausente.

Um agregado ao qual falta um componente condicional (`vFCPST`, `vIPIDevol`)
sai como `PARCIAL` com os componentes ausentes nomeados: um agregado
incompleto nao pode parecer completo.

### Auditoria de alias (C16)

`auditar-aliases` classifica cada alias isolado e mostra a que conta ele leva
de fato. Baseline medida nas regras canonicas:

| Arquivo | Alias | Sem classificacao | Levam a outra conta | Ambiguos |
|---|---|---|---|---|
| produtos | 454 | 0 | 6 | 9 |
| despesas | 117 | 0 | 1 | 2 |

Divergir nao e' por si so um erro: outra regra pode ser mais especifica ou ter
prioridade maior. Mas tem de ser uma escolha vista. Exemplos reais:

- `cervejeira` esta declarado em `13.01.009` (Possivel Investimento) e vence
  como `2.01.001` (Cervejas), porque o alias de 3 letras `cerv` casa por
  substring — exatamente o caso que `politica_de_match` pretendia evitar;
- `aluguel maquina cartao` (`7.02.004`) vence como `4.01.001` (Aluguel do Imovel);
- `papel manteiga` (`2.03.001`) vence como `2.02.006` (manteiga).

O teste `auditoria_de_aliases_corresponde_a_baseline_declarada` fixa esses
numeros. Se mudarem, a suite falha e mostra a lista: a baseline so pode ser
atualizada junto com uma decisao declarada sobre as regras.

### Divergencias intencionais em relacao ao Python

| O que | Python | Aqui | Porque |
|---|---|---|---|
| Ordem de `matches` e das contas no alerta de ambiguidade | `set`, ordem arbitraria | ordenada | saida reproduzivel entre execucoes |
| Texto do alerta de ambiguidade | `repr` de um `set` | `"Classificacao ambigua: contas 2.01.001, 2.01.002"` | legivel e estavel |
| Flags de `normalizacao:` | ignorados | aplicados | ver C9; nos YAML reais sao todos `true`, logo mesmo resultado |
| `evitar_aliases` | lido so pelo gerador de propostas | idem (nao aplicado) | nao e' regra de classificacao |
| Valor obrigatorio ausente no XML | `Decimal("0")` silencioso | **erro** que nomeia o caminho | dinheiro ausente nao e' dinheiro zero; ver C10 |
| `total/ICMSTot` ausente | totais a zero | **erro** | uma NF-e de zero reais entraria na contabilidade |

### Decisoes registradas

- **Sem fallback silencioso.** `PlanoContas::por_id` devolve `Option`. O unico
  fallback e' `conta_revisao`, declarado, que devolve tambem o motivo usado.
- **Tipo errado de conta de revisao = ERRO bloqueante**, nao alerta.
- **`tipo_lancamento` e' `TipoConta`**: o Python propaga `conta.tipo` direto
  (`allocation.py:77/148/164`), entao os 11 tipos tem de ser representaveis.
- **Elementos fora do namespace NF-e nao entram no parser** (a assinatura
  `ds:Signature` nao pode injetar campos), com limites de profundidade e de
  numero de elementos.
- **`Competencia`: intervalo 1900..=2999** (`ANO_MIN`/`ANO_MAX`), escolha desta
  porta, nao do Python.

## Dependencias

| Crate | Uso | Observacao |
|---|---|---|
| `serde` (derive) | serializacao do modelo e das configuracoes | |
| `serde_yaml` | leitura de `config/*.yaml` | **deprecado upstream** (0.9.34+deprecated). Decidir substituto antes de producao |
| `quick-xml` | leitura de NF-e com resolucao de namespace | |
| `clap` (derive) | CLI | |
| `rust_decimal` | valores monetarios exatos | `f64` daria erro de precisao silencioso em dinheiro |
| `chrono` | datas (`NaiveDate`) | equivalente de `datetime.date` |
| `unicode-normalization` | NFD + remocao de acentos | equivalente de `unicodedata` do Python |

## Verificacao

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

105 testes: 92 de unidade na lib, 3 no binario, 10 de integracao de CLI.

Testes que amarram o codigo aos dados reais, nao a fixtures:

- `configuracao_real_do_repositorio_nao_tem_erros` valida os YAML de `config/`;
- `fallback_fora_da_area_de_revisao_reprova_sem_estrito` executa a CLI e checa
  o codigo de saida;
- o modulo `nfe::tests` repete os valores esperados de
  `tests/test_nfe_importer.py` sobre a mesma NF-e de exemplo;
- `classificacao::tests` repete os casos de `tests/test_classifier.py`;
- `nenhum_alias_real_fica_sem_classificacao` e
  `auditoria_de_aliases_corresponde_a_baseline_declarada` passam pelos 571
  alias dos dois arquivos de regras;
- `tests/cli_configuracao_obrigatoria.rs` executa a CLI de verdade e garante
  que nenhum comando aceita configuracao reprovada.

`tests/fixtures/` contem copias **propositalmente defeituosas ou reduzidas**,
usadas pelos testes de regressao: `regras_produtos_fallback_custo.yaml` (regras
com fallback errado) e `nfe_sem_fat.xml` (o campione canonico sem o bloco
`<fat>`). Nao sao fonte de regras nem substituem o campione: este nunca e'
alterado para fazer um teste passar.

## Riscos conhecidos

1. **Dupla fonte de regras.** `config/` e' copia do repo Python. Ver
   [`config/FONTE.md`](config/FONTE.md).
2. `serde_yaml` deprecado (acima).
3. **Regras declaradas e nao aplicadas** (`regras_por_fornecedor`,
   `politica_de_match`): hoje visiveis como alerta, ver C13. Decisao pendente.
4. `mapeamento_dre.yaml` e `csv_columns.yaml` versionados mas **nao lidos**.
5. **Cobertura nao modelada em `Dado<T>`**: hoje so valor, fonte, status e
   motivo. Debito a fechar antes de saidas operacionais que dependam dela.
6. O parser le `dhEmi` (NF-e 4.00). Notas no layout antigo com `dEmi` seriam
   recusadas, exatamente como no Python.
7. **O parser e' mais estrito que o Python** em valores obrigatorios ausentes
   (ver tabela de divergencias). Uma NF-e que o Python importava com zeros
   agora e' recusada nomeando o campo. E' intencional, mas se aparecer um
   fornecedor que omite uma tag, o import para: nesse caso a decisao e'
   explicita, nao silenciosa.
8. Os 7 alias divergentes e 11 ambiguos da tabela C16 continuam como estao:
   sao dados para decisao, nao bugs corrigidos de oficio.
