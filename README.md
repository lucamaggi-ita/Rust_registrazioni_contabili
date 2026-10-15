# Rust_schede_tecniche

Porte em Rust de [Programma_registrazioni-contabili](https://github.com/lucamaggi-ita/Programma_registrazioni-contabili)
(agente *Fantozzi*: registro contabil de NF-e para restaurante no Brasil).

## Estado atual: Fase 1

| Fase | Conteudo | Estado |
|---|---|---|
| 1 | Dependencias, modelo de dados, leitura e validacao das regras YAML | **PRONTA** (revisao 2026-10-15 aplicada) |
| 2 | Parser NF-e (`quick-xml`), normalizacao de texto, classificacao produtos/despesas | NAO_INICIADA |
| 3 | Pre-livro diario, DRE gerencial, relatorios | NAO_INICIADA |

O que ja funciona de ponta a ponta: carregar `config/*.yaml`, validar contra os
contratos abaixo e reprovar a configuracao se algum contrato for violado.

## Uso

```bash
cargo run -- validar-config                 # usa os caminhos padrao de config/
cargo run -- validar-config --estrito       # alertas tambem reprovam (uso em CI)
cargo run -- validar-config --plano-contas <path> --regras-produtos <path> --regras-despesas <path>
```

Codigo de saida: `0` = APROVADO, `1` = REPROVADO ou erro de carregamento.

`--estrito` serve para **alertas**. Violacoes de contrato sao sempre ERRO e
reprovam tambem sem o flag.

## Contratos implementados

| ID | Contrato | Onde | Teste |
|---|---|---|---|
| C1 | Valores monetarios em `Decimal`, nunca `f64` | `modelos.rs` | `decimal_nao_introduz_erro_binario`, `decimal_sobrevive_round_trip` |
| C2 | Dado faltante != zero: `Dado<T>` com valor, fonte, status e motivo | `modelos.rs` | `dado_ausente_nao_vira_zero`, `dado_exige_fonte_nao_vazia`, `dado_ausente_exige_motivo` |
| C3 | `deny_unknown_fields`: chave errada no YAML = erro, nao silencio | `regras.rs`, `modelos.rs` | `c3_chave_desconhecida_no_yaml_falha`, `conta_com_campo_desconhecido_falha` |
| C4 | `tipo` de conta e enum fechado: valor desconhecido falha | `modelos.rs` (`TipoConta`) | `tipo_conta_desconhecido_falha`, `tipos_de_conta_correspondem_as_strings_do_python` |
| C5 | Plano de contas: sem IDs duplicados, sem campos vazios, contas de revisao obrigatorias presentes **e do tipo revisao** | `PlanoContas::validar` | `c5_*` |
| C6 | Toda regra aponta para uma conta existente; o fallback existe **e e' do tipo revisao** | `RegrasClassificacao::validar` | `c6_*`, `cli_validar_config.rs` |
| C7 | Proveniencia dos YAML declarada (repo + commit de origem) | `config/FONTE.md` | - |
| C8 | Tipos validados (`Competencia`, `Dado<T>`) tem campos privados e um unico caminho de validacao, compartilhado por construtores e serde | `modelos.rs` | `competencia_round_trip_sempre_valido`, `dado_deserializado_incoerente_falha` |

C6 nao existia na versao Python: la, um `conta_id` errado so estourava em
tempo de execucao, ao processar a primeira nota que casasse com a regra.

### Decisoes registradas

- **Sem fallback silencioso.** `PlanoContas::por_id` devolve `Option`, nunca uma
  conta "parecida". O unico fallback e' `conta_revisao`, que e' declarado e
  devolve tambem o motivo efetivamente usado (`OUTROS` quando o motivo pedido
  nao esta mapeado).
- **Tipo errado de conta de revisao = ERRO bloqueante**, nao alerta: um fallback
  que aponte para uma conta ordinaria faria os produtos nao classificados
  desaparecerem dentro dos custos correntes.
- **`tipo_lancamento` e' `TipoConta`**, nao um enum proprio: o Python propaga
  `conta.tipo` direto (`allocation.py:77/148/164`), entao os 11 tipos tem de ser
  representaveis — inclusive `revisao`, `passivo`, `provisao`, `patrimonial` e
  `deducao_receita`, que sao justamente os casos que nao podem sumir.
- **Defaults declarados**, nao implicitos: `exige_revisao = false`,
  `prioridade = 0` (as regras de despesas nao trazem prioridade), ambos
  documentados no proprio campo.
- **`Competencia` e `Dado<T>` tem campos privados**: nao e' possivel construir
  `2026-13`, nem um dado `OK` sem valor, nem um valor sem fonte. A validacao nao
  depende de `debug_assert` (que some em release).
- **Intervalo de `Competencia`: 1900..=2999** (`ANO_MIN`/`ANO_MAX`). Escolha
  desta porta, nao do Python; `from_data` passa pela mesma validacao.

## Dependencias

| Crate | Uso | Observacao |
|---|---|---|
| `serde` (derive) | serializacao do modelo e das configuracoes | |
| `serde_yaml` | leitura de `config/*.yaml` | **deprecado upstream** (0.9.34+deprecated). Decidir substituto (`serde_norway` / `yaml_serde`) antes de producao |
| `quick-xml` | parser de NF-e | declarada, **ainda nao usada**: entra na fase 2 |
| `clap` (derive) | CLI | |
| `rust_decimal` | valores monetarios exatos | fora da lista inicial: `f64` daria erro de precisao silencioso em dinheiro |
| `chrono` | datas (`NaiveDate`) | fora da lista inicial: equivalente de `datetime.date` |

## Verificacao

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

42 testes: 36 de unidade na lib, 3 no binario, 3 de integracao de CLI.

Dois deles amarram o codigo aos dados reais:

- `configuracao_real_do_repositorio_nao_tem_erros` roda a validacao sobre os
  YAML de `config/`: um arquivo de regras quebrado derruba a suite;
- `fallback_fora_da_area_de_revisao_reprova_sem_estrito` executa a CLI de
  verdade e checa o codigo de saida.

`tests/fixtures/` contem copias **propositalmente defeituosas** usadas pelos
testes de regressao. Nao sao fonte de regras.

## Riscos conhecidos

1. **Dupla fonte de regras.** `config/` e' copia do repo Python. Ver as regras
   anti-divergencia em [`config/FONTE.md`](config/FONTE.md).
2. `serde_yaml` deprecado (acima).
3. `mapeamento_dre.yaml` e `csv_columns.yaml` estao versionados mas **nao sao
   lidos** por nenhum codigo Rust ainda.
4. **Cobertura nao modelada em `Dado<T>`.** A carta de identidade pede tambem
   cobertura; hoje so ha valor, fonte, status e motivo. Debito declarado, a
   fechar antes de qualquer saida operacional que dependa de cobertura.
