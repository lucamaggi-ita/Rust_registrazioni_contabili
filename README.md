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
| 2 | Leitura de NF-e (`quick-xml`), normalizacao de texto, classificacao produtos/despesas | **PRONTA** (revisoes 2026-10-15 e 2026-10-16 aplicadas; C17 fechado com audit independente) |
| 3A | Analise e especificacao da alocacao (contratos, formulas, casos numericos) | **PRONTA** (`SPECIFICA_FASE3A_REGISTRAZIONI.md`, audit tecnico superado) |
| 3B | Alocacao de NF-e em lancamentos com reconciliacao (`alocacao.rs`) | **PRONTA**: audit tecnico independente **superado** em 2026-10-16 (R3B-01/02/03 fechados); aguarda apenas o registro em Git autorizado |
| 3C.1 | Analise e especificacao de pre-livro, revisao e exportacao | **PRONTA** (`SPECIFICA_FASE3C_PRELIBRO_REVISIONE_EXPORT.md`, revisao 2.1) |
| 3C.2 | Pre-livro em memoria: particoes, resumos, invariantes (`prelivro.rs`) | **PRONTA**: audit tecnico independente **superado** em 2026-10-17 (R3C2-01/02/03 fechados); aguarda apenas o registro em Git autorizado |
| 3C.3 | Estados e revisao humana: transicoes, rastreamento | NAO_INICIADA |
| 3C.4 | Adaptadores de entrada bruta, arquivo autoritativo, exportacao | NAO_INICIADA |
| 3C.5 | Uso pela CLI | NAO_INICIADA |
| 3D | DRE gerencial por competencia | NAO_INICIADA |

Funciona de ponta a ponta: validar a configuracao, ler uma NF-e em XML,
classificar cada item contra o plano de contas e **alocar o documento em
lancamentos com reconciliacao**. A fase 3B **nao grava nada**: `alocar_nfe` e'
uma funcao pura que devolve os lancamentos em memoria. Gravar, revisar e
exportar e' 3C; agregar a DRE e' 3D.

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
| C17 | **Um documento, uma NF-e.** Mais de uma raiz XML ou mais de um `<infNFe>`, **inclusive aninhado**, = erro com saida 1, nunca sucesso parcial na primeira nota | `nfe.rs` | `duas_raizes_sao_recusadas`, `segunda_raiz_fora_do_namespace_tambem_e_recusada`, `lote_com_duas_nfe_e_recusado`, `infnfe_aninhado_e_recusado`, `documento_canonico_tem_uma_so_nfe`, `tests/cli_importar_nfe.rs` |

### Fase 3B - alocacao em lancamentos

Numeracao definitiva dos contratos implementados em `alocacao.rs`, com o
identificativo provisorio da especificacao 3A entre parenteses. As seis
decisoes de dominio **D1-D6 foram aprovadas por Luca em 2026-10-16**.

| ID | Contrato | Onde | Teste |
|---|---|---|---|
| C18 | **Uma registro por (documento, conta)**, na ordem de primeira aparicao; valor = soma dos `vProd` do grupo; diferenca de total em conta propria (F3A-01…06) | `alocacao.rs` | `aloca_campione_canonico_em_tres_lancamentos`, `diferenca_*` |
| C19 | **Ausente nao explica.** Componente necessaria com `Dado` != `Ok` nao pode declarar explicada a diferenca: documento `PARCIAL`, diferenca `REVISAR` (F3A-07, **D2**) | `alocacao.rs` | `componente_parcial_nao_explica_diferenca`, `documento_parcial_com_componente_necessaria_parcial` |
| C20 | **Rateio das duplicatas** sobre `vNF`, restos na ultima linha e coluna; tres invariantes verificadas; rateio nao definido = erro, nunca zero inventado (F3A-08…10, **D1**) | `alocacao.rs` | `riparto_das_rate_fecha_por_linha_e_por_coluna`, `total_zero_com_rate_nao_vira_zero_tecnico`, `total_netto_zero_com_lancamentos_nao_nulos` |
| C21 | **Dominio monetario em centavos**: valor fora do dominio = erro que nomeia campo e valor; zeros finais nao desqualificam; `qCom`/`vUnCom` excluidos; `Dado` ausente nao e' zerado (F3A-26, **D6**) | `alocacao.rs` | `importo_fora_do_dominio_monetario_e_recusado`, `zeros_finais_nao_squalificam_o_importo`, `dado_assente_non_viene_azzerato_per_passare_il_dominio` |
| C22 | **Identidade**: chave vazia bloqueia; numero e identidade de emitente/destinatario vazios vao a revisao com aviso proprio; `serie` sem efeito (F3A-23a/b, **D5**) | `alocacao.rs` | `chave_vazia_nao_produz_lancamento`, `numero_vazio_marca_revisar`, `*_sem_identidade_marca_revisar` |
| C23 | **Hash de quatro campos** igual ao Python; payload repetido no mesmo documento e' aviso, nunca descarte; nenhuma deduplicacao persistente (F3A-15/27, **D4**) | `alocacao.rs` | `hash_do_campione_canonico_corresponde_ao_payload`, `payload_duplicado_no_documento_gera_aviso` |
| C24 | **Reconciliacao com quatro esitos** independentes (total, duplicatas, cobertura, componentes) e uma regra de agregacao `ERRO > PARCIAL > OK`; o esperado de itens fica `NAO_DISPONIVEL`, nunca inventado (F3A-28) | `alocacao.rs` | `copertura_parcial_com_det_sem_prod`, `rate_que_nao_fecham_geram_valor_inconsistente` |

**Limite tecnico, nao regra de dominio.** Toda a aritmetica de `alocacao.rs`
acontece em **centavos inteiros** (`i128`), nao em `Decimal`. Riscontro R3B-01
do audit de 2026-10-16: com importes extremos mas legitimos pelo dominio de
C21, `Decimal` entra em panico no produto do rateio e **perde um centavo em
silencio** na soma — `checked_add` inclusive, que devolve `Some` com o valor
arredondado. Em centavos inteiros a soma e o produto sao exatos; quando o
resultado nao cabe, o alocador devolve
`ErroAlocacao::CapacidadeNumericaExcedida` nomeando operacao e valores, em vez
de um resultado parcial ou de um panico. **Nao e' um teto de importe aprovado**:
e' o limite da representacao, declarado.

Duas precisoes vindas do audit mirado (R3B-01, residuos A e B):

1. a volta de centavos para `Decimal` **reduz zeros finais** baixando a escala
   ate 0 quando precisa, porque um valor inteiro grande e' representavel a
   escala 0 mesmo quando `mantissa x 100` nao cabe: impor escala 2 restringia
   artificialmente o dominio do proprio `Decimal`. Nenhum digito diferente de
   zero e' descartado e nada e' arredondado;
2. divisao e resto do rateio usam `checked_div`/`checked_rem`, porque
   `i128::MIN / -1` nao e' representavel e os operadores comuns entrariam em
   **panico**.

Testes: `riparto_com_importi_estremi_nao_entra_em_panico`,
`dois_itens_extremos_em_contas_distintas_preservam_o_centavo`,
`valor_agregado_nao_representavel_e_erro`,
`divisao_do_rateio_nao_entra_em_panico_com_minimo`,
`round_trip_de_centesimos_respeita_o_dominio_de_decimal`,
`decimal_perde_centavo_na_soma_extrema`,
`decimal_estoura_no_produto_do_rateio`.

Decisoes registradas tambem aqui: um item sem descricao **e' conservado** com
aviso `ITEM_SEM_DESCRICAO` e `REVISAR`, sem inventar descricao (F3A-24); a
linha DRE e o centro de custo ficam declarados em
`Reconciliacao::campi_rinviati` com estado, fonte e motivo, em vez de serem
strings vazias silenciosas (F3A-17/18/19).

### Fase 3C.2 - pre-livro em memoria

Implementado em `prelivro.rs` em 2026-10-17; **audit tecnico independente superado** no mesmo dia, depois da correcao de R3C2-01/02/03:
os contratos conservam os identificadores de analise `F3C-nn` da especificacao
e **nao** recebem numeracao `C25+`, porque a fase 3C tem cinco blocos e so dois
estao fechados: uma numeracao definitiva de fase incompleta seria prematura. As
decisoes de dominio **D10(a)** e **D13(a)** foram aprovadas por Luca em
2026-10-17; D7, D8, D9, D11 e D12 seguem abertas para os blocos seguintes.

| ID | Contrato | Onde | Teste |
|---|---|---|---|
| F3C-01 | A entrada e' `AlocacaoNfe` + contexto tipado, nunca um workbook; `construir_pre_livro` e' **pura** | `prelivro.rs` | `t01_campione_canonico_*` |
| F3C-01b | **Contexto e alocacao tem de falar do mesmo documento**: chave, numero, data, competencia e CNPJ da empresa sao comparados antes de agregar; divergencia = erro nomeado, nenhuma linha reescrita | `prelivro.rs` | `r1_chave_discordante_*`, `r1_empresa_discordante_*`, `r1_competencia_discordante_*`, `r1_contexto_coerente_*` |
| F3C-02/07 | **Toda linha recebida cai em exatamente uma particao**; nenhum descarte implicito (I1, I2) | `prelivro.rs` | `t15_conservacao_e_contagens_fecham` |
| F3C-03 | Particao com precedencia declarada e **todas as causas** conservadas em `motivos` | `prelivro.rs` | `t14_particao_unica_com_mais_de_um_motivo` |
| F3C-08 | Resumos por `(cnpj_empresa, competencia, conta_id)`; empresa nao identificada em particao propria, **nunca** fundida (D13) | `prelivro.rs` | `t10_duas_empresas_*`, `t11_empresa_nao_identificada_*` |
| F3C-09 | Nome da conta vem do **`PlanoContas` recebido como parametro**; nomes de linha divergentes conservados e sinalizados; conta fora do plano = erro nomeado | `prelivro.rs` | `t12_nome_do_conto_*`, `t22_*`, `t23_*` |
| F3C-11 | Diagnostico e resumos conservam tudo, **marcado**: cada resumo diz de quantos documentos `OK`/`PARCIAL`/`ERRO` vem e nomeia os nao `OK`; extrato aprovado so com linhas `APROVADO` de documentos nao `ERRO`; **cada exclusao e' listada** com identidade, valor, estado e motivo (D10) | `prelivro.rs` | `t05_documento_erro_*`, `t18_exclusoes_*`, `r2_resumo_marca_*`, `r2_problema_de_um_grupo_*` |
| F3C-12 | A `Reconciliacao` do documento e' conservada **integra**, nao achatada em colunas de linha | `prelivro.rs` | `t09_reconciliacao_*` |
| F3C-25 | Ordenacao deterministica: permutar a entrada nao muda o resultado, **nem nos conteineres agregados** (avisos e nomes divergentes tem ordem canonica) nem nos metadados do lote (I7) | `prelivro.rs` | `t19_permutar_a_ordem_*`, `r3_permutacao_com_avisos_*`, `r3_metadados_do_lote_*` |
| F3C-26 | **Somas em centavos inteiros**, com os helpers compartilhados de `dinheiro.rs`; agregado nao representavel = erro, nunca truncamento nem pre-livro parcial | `dinheiro.rs`, `prelivro.rs` | `t16_somas_por_chave_*`, `t21_soma_de_varios_documentos_*` |
| F3C-27 | Cada documento do lote aparece **mesmo sem linhas**, com chave, empresa, competencia e estado (I8) | `prelivro.rs` | `t06_documento_sem_linhas_*` |

**`dinheiro.rs`: uma so aritmetica monetaria no crate.** Os helpers de centavos
da fase 3B foram movidos para um modulo interno compartilhado, sem duplicacao e
**sem mudanca de comportamento**: os testes da 3B continuam identicos e verdes.
Era o pre-requisito do bloco, porque somar `Decimal` entre documentos
individualmente validos pode perder um centavo em silencio.

Fora deste bloco, por contrato: entradas brutas (`INCOMPLETO`,
`NAO_RECONHECIDO`), arquivo autoritativo, exportacao, import do historico, CLI
e transicoes de revisao humana.

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
| `sha2` | `hash_registro` dos lancamentos (fase 3B) | adicionada em 2026-10-16 com autorizacao explicita; mesmo payload de quatro campos do Python, para que os codigos continuem comparaveis com o historico |

## Verificacao

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

208 testes: 192 de unidade na lib, 3 no binario, 13 de integracao de CLI.

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
usadas pelos testes de regressao. Cada uma declara no proprio cabecalho a
origem, como foi construida e o que se espera dela:

| Fixture | Defeito intencional | Protege |
|---|---|---|
| `regras_produtos_fallback_custo.yaml` | fallback que nao e' conta de revisao | C6, C15 |
| `nfe_sem_fat.xml` | campione canonico sem o bloco `<fat>`, as duas parcelas de 17.50 ficam | C10 |
| `nfe_duas_raizes.xml` | dois documentos concatenados, a segunda nota com `nNF` 12567 | C17 |
| `nfe_infnfe_annidata.xml` | XML bem formado com um `<infNFe>` aninhado dentro do outro | C17 |
| `plano_sem_contas_de_ajuste.yaml` | plano valido na forma, **sem** `2.05.001`/`2.05.002` | C18, C24 (erro `ContaDeAjusteAusente`) |

Nao sao fonte de regras nem substituem o campione: este nunca e' alterado para
fazer um teste passar.

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
9. **Nao existe validacao XSD.** C17 conta raizes e `<infNFe>`, nao verifica o
   layout: um XML bem formado mas fora do layout NF-e (por exemplo `infNFe`
   aninhado) e' recusado pela contagem, nao por um schema. Se a validacao real
   do layout for necessaria, e' uma decisao separada, com fonte e testes
   proprios.
