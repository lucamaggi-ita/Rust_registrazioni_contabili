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
| 3C.2 | Pre-livro em memoria: particoes, resumos, invariantes (`prelivro.rs`) | **PRONTA**: audit tecnico independente **superado** em 2026-10-17 (R3C2-01/02/03 fechados), registrada e publicada |
| 3C.3 — especificacao | Revisao humana, versoes e impressao canonica | **PRONTA** (`SPECIFICA_FASE3C3_REVISIONE_VERSIONI.md`, revisao 4): audit tecnico superado em 2026-10-21; as dez subescolhas D8.1–D8.5 e D9.1–D9.5 foram **aprovadas** na opcao (a) |
| 3C.3.a | Impressoes canonicas (`impressao.rs`) e revisao pura em memoria (`revisao.rs`) | **PRONTA**: audit tecnico independente **superado** em 2026-10-21 (R3C3A-01/02/03 fechados), registrada em Git |
| 3C.3.b — especificacao | Persistencia do arquivo autoritativo: formato, codec, protocolo de publicacao e recuperacao | **PRONTA** (`SPECIFICA_FASE3C3B_PERSISTENZA_ARCHIVIO.md`, revisao 3): audit tecnico superado em 2026-10-22; D14–D17 **aprovadas** na opcao (a) |
| 3C.3.b.1 | Arquivo autoritativo em JSON (`arquivo.rs`) e protocolo de IO (`arquivo_io.rs`) | **AUDIT TECNICO INDEPENDENTE SUPERADO** em 2026-10-22: quatro achados (R3C3B1-01/02/03/04) levantados, corrigidos e **fechados** pelo audit dirigido. Falta so' o registro em Git autorizado |
| 3C.4 | Adaptadores de entrada bruta, exportacao para Excel/CSV | NAO_INICIADA |
| 3C.5 | Uso pela CLI | NAO_INICIADA |
| 3D | DRE gerencial por competencia | NAO_INICIADA |

Funciona de ponta a ponta: validar a configuracao, ler uma NF-e em XML,
classificar cada item contra o plano de contas e **alocar o documento em
lancamentos com reconciliacao**. A fase 3B **nao grava nada**: `alocar_nfe` e'
uma funcao pura que devolve os lancamentos em memoria. A 3C.2 agrega esses
lancamentos num pre-livro, e a 3C.3.a registra **decisoes humanas declaradas**
sobre ele — tambem em memoria. A 3C.3.b.1 **grava**: um arquivo JSON unico que
e' a fonte autoritativa (D7(a)) dos contextos originais e das declaracoes de
revisao, com publicacao de um so ponto de commit. **Exportar** para Excel ou
CSV continua sendo 3C.4; agregar a DRE e' 3D.

O que o arquivo **nao** guarda: a vista revisada, os resumos, as particoes e o
extrato. Sao derivaveis dos dados conservados, e salva-los criaria uma segunda
verdade.

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

### Fase 3C.3.a - impressoes canonicas e revisao humana em memoria

Dois modulos novos, ambos **puros**: nenhum relogio, nenhum usuario de sistema
operacional, nenhum acesso ao disco, nenhum historico persistente.

| Contrato | Regra | Codigo | Teste |
|---|---|---|---|
| F3C-04 | Nenhuma promocao automatica a `APROVADO`: so um evento declarado aprova | `revisao.rs` | `t3_01_*`, `t3_01_sem_eventos_*` |
| F3C-05 | Aprovar **nao** apaga avisos nem muda os quatro esitos tecnicos | `revisao.rs` | `t3_02_aprovar_nao_apaga_*` |
| F3C-13 | A decisao se liga ao **conteudo examinado** (`ChaveRevisao` sem lote); `IdentidadeLinha` com lote conserva a proveniencia da execucao | `revisao.rs` | `t3_10_mesmo_conteudo_em_outro_lote_*`, `t3_07_*` |
| F3C-22 | Decisoes e anotacoes se reaplicam numa regeneracao de conteudo identico | `revisao.rs` | `t3_10_*`, `t3_15_*` |
| F3C-23 | Revogar e repetir **nao apagam** nada da trilha | `revisao.rs` | `t3_04_revogacao_*`, `t3_14_repeticao_*` |
| F3C-28 | Autor, instante e motivo vem do chamador; nenhum default, relogio ou usuario do sistema | `revisao.rs` | `t3_05_eventos_malformados_*` |
| F3C-29 | Esito tecnico, decisao humana e estado da vista ficam em tres campos distintos | `revisao.rs` | `t3_01_*`, `t3_02_*` |
| F3C-30 | A projecao escreve **um so** campo (`status_revisao`); os outros 25 de `Lancamento` ficam identicos | `revisao.rs` | `t3_03_projecao_muda_so_*` |
| F3C-31 | Revogar e' um evento novo; a aprovacao anterior permanece e o estado **tecnico** e' restaurado (D9.3) | `revisao.rs` | `t3_04_*` (dois testes) |
| F3C-32 | Evento malformado = erro que nomeia o campo e o evento, nunca a posicao no array | `revisao.rs` | `t3_05_*`, `t3_21_*` |
| F3C-33 | Versao superada ou alvo ausente: **nao** se aplica a outra linha, e o evento fica consultavel | `revisao.rs` | `t3_06_*` (tres testes), `t3_08_*` |
| F3C-34 | `IMPRESSAO-CONTEUDO/v1`: texto canonico declarado, prefixos de comprimento, SHA-256 hexadecimal minusculo | `impressao.rs` | `t3_12_vetor_i_byte_a_byte_*` |
| F3C-35 | Duas linhas com o mesmo `hash_registro` (C23) continuam alvos distintos | `revisao.rs` | `t3_07_duas_linhas_com_o_mesmo_hash_*` |
| F3C-36 | Correcao com chave, conta e total invariados muda a impressao | `impressao.rs`, `revisao.rs` | `t3_08_correcao_*`, `t3_08_versao_superada_*` |
| F3C-37 | A impressao e' invariante a caminho, lote, campos humanos e forma Unicode; **nao** a mudanca de conta/nome de linha | `impressao.rs` | `t3_13_*` (dois testes) |
| F3C-38 | Impressao de apresentacao **separada**: renomear uma conta no plano sinaliza `APRESENTACAO_MUDADA` e nao invalida as aprovacoes | `impressao.rs`, `revisao.rs` | `t3_09_*`, `apresentacao_mudada_*` |
| F3C-39 | Repeticao exata declarada; decisoes incompativeis no mesmo alvo e instante = conflito, sem "vence o ultimo". A escolha do conflito relatado segue a **ordem canonica dos eventos**, nunca a posicao dos documentos | `revisao.rs` | `t3_14_*` (tres testes), `r3_escolha_do_conflito_*`, `r3_conflito_estavel_*` |
| F3C-40 | A sequencia historica **nao** e' calculavel: `Dado` `NAO_DISPONIVEL` com fonte declarada (D8.5) | `revisao.rs` | `t3_16_documento_sem_linhas_*` |
| F3C-41 | Tudo ou nada: em erro nenhuma vista parcial e' produzida | `revisao.rs` | `t3_18_*` |
| F3C-42 | `APROVADO` sem evento de proveniencia nao e' revisao verificada: dado original conservado, fora do extrato revisado, sanavel com evento. Entrar no extrato revisado exige uma aprovacao humana **ativa**: revogada a sanatoria, a linha sai de novo | `revisao.rs` | `t3_16_*` (dois testes), `r1_revogacao_da_sanatoria_*`, `r1_revogacao_de_linha_ok_*` |
| F3C-43 | Todo ordenamento canonico **normaliza as chaves antes** de comparar, compara os bytes UTF-8 e emite em NFC; empates canonicos nao descartam nada | `impressao.rs`, `revisao.rs` | `t3_13_n1_*`, `t3_20_empate_canonico_*`, `avisos_da_vista_*` |
| F3C-44 | Ordem canonica de seis niveis, total sobre os eventos sobreviventes; a posicao no array nao entra na ordem, na saida nem na escolha do erro | `revisao.rs` | `t3_17_*`, `t3_21_*`, `t3_11_*` |
| F3C-45 | So a **identidade exata** autoriza o colapso de uma repeticao; nenhuma deduplicacao semantica | `revisao.rs` | `t3_22_c1_*`, `t3_22_c2_c3_c4_c5_*` |
| F3C-46 | `texto_exato_evento` usa producoes proprias (`str_exato`, `dado_esatto_string`) que conservam bytes originais, presenca e `motivo` dos `Dado`; o instante conserva a hora local, os nanossegundos e o offset **em segundos** | `revisao.rs` | `t3_22_codificacao_exata_e_injetiva_*`, `texto_exato_nao_normaliza_*`, `r2_offset_declarado_com_segundos_*`, `r2_nanossegundos_*` |

Superficie publica nova:

```rust
// impressao.rs
pub fn texto_canonico(&ContextoDocumento) -> String;
pub fn impressao_conteudo(&ContextoDocumento) -> String;         // 64 hex
pub fn contas_mostradas(&ContextoDocumento, &PlanoContas)
    -> Result<Vec<ContaMostrada>, ContaAusenteNoPlano>;
pub fn texto_apresentacao(&str, &[ContaMostrada]) -> String;
pub fn impressao_apresentacao(&str, &[ContaMostrada]) -> String;

// revisao.rs
pub fn aplicar_revisao(&[ContextoDocumento], &PlanoContas, &[EventoRevisao])
    -> Result<VistaRevisada, ErroRevisao>;
pub fn texto_exato_evento(&EventoRevisao) -> String;
pub fn chave_canonica(&EventoRevisao) -> ChaveCanonica;
```

**Limites deste bloco, declarados.** Os eventos sao um **parametro**: nao
existe arquivo, nem banco, nem ficheiro. A trilha fica consultavel em memoria,
e isso **nao** e' conservacao em disco: fechar o processo perde os eventos. O
`autor` e' uma declaracao, nao uma autenticacao: o sistema nao sabe quem
escreveu aquela cadeia. `AlvoAmbiguo` existe no tipo de erro mas **nao e'
alcancavel** pela API: duas linhas com a mesma `(chave, impressao, origem,
conta, ordinal)` exigiriam dois documentos com a mesma chave, que a validacao
da 3C.2 rejeita antes (`alvo_ambiguo_e_interceptado_antes_pela_validacao_da_3c2`
registra qual controle intercepta o caso). Corrigir uma conta, um valor, um
pagamento ou uma regra YAML continuam operacoes separadas, nao alcancaveis
daqui.

**`dinheiro.rs`: uma so aritmetica monetaria no crate.** Os helpers de centavos
da fase 3B foram movidos para um modulo interno compartilhado, sem duplicacao e
**sem mudanca de comportamento**: os testes da 3B continuam identicos e verdes.
Era o pre-requisito do bloco, porque somar `Decimal` entre documentos
individualmente validos pode perder um centavo em silencio.

Fora deste bloco, por contrato: entradas brutas (`INCOMPLETO`,
`NAO_RECONHECIDO`), arquivo autoritativo, exportacao, import do historico, CLI
e transicoes de revisao humana.

### Fase 3C.3.b.1 - arquivo autoritativo e protocolo de IO

Dois modulos com responsabilidades separadas. `arquivo.rs` e' **puro**: nao
abre ficheiros, nao le o relogio e nao conhece caminhos. `arquivo_io.rs` faz
IO e nao decide nada sobre o conteudo.

| Contrato | O que garante | Onde | Teste |
|---|---|---|---|
| PER-01/02 | `FANTOZZI-ARQUIVO/v1`, `versao_schema` 1, e as tres gramaticas (`IMPRESSAO-CONTEUDO/v1`, `IMPRESSAO-APRESENTACAO/v1`, `EVENTO-EXATO/v1`) escritas nos bytes e verificadas na leitura | `arquivo.rs` | `tp11_formato_schema_e_gramatica_tem_erro_proprio` |
| PER-03 | Um esquema mais novo **nao** e' lido por tentativa: `SchemaMaisNovo` antes de qualquer interpretacao | `arquivo.rs` | idem |
| PER-05 | Cada execucao conserva a **identidade do plano** que usou; o plano nao vive na raiz | `arquivo.rs` | `tp02_execucao_conserva_o_proprio_plano` |
| PER-07/20 | Todo monetario e' **texto**. Um numero JSON e' recusado antes de qualquer conversao; o texto passa por `from_str_exact` **e** pela reproducao com `to_string`, porque so o par recusa `3.5e1`, `+35.00`, `35.` e `-0.00` | `arquivo.rs` | `tp04_numero_json_*`, `tp04_decimais_hostis_*` (motivos medidos, um por caso), `tp04_decimais_validos_*`, `tp04_decimal_dentro_de_dado_*` |
| PER-10 | Ausencia, zero e vazio ficam distintos: um `Dado` ausente escreve `null` explicito com estado e motivo, e um estado incoerente e' recusado por `Dado::novo` | `arquivo.rs` | `tp03_ausencia_zero_e_vazio_*`, `tp03_dado_incoerente_*` |
| PER-11/12 | Leitura **restritiva**: DTO com `deny_unknown_fields` em cada struct, sem arvore `serde_json::Value`. Campo desconhecido e chave duplicada sao recusados **tambem aninhados** | `arquivo.rs` | `tp11_campo_desconhecido_aninhado_*`, `tp11_chave_duplicada_aninhada_*` |
| PER-13/14 | Impressoes **recalculadas** na validacao: conteudo alterado ou impressao alterada dao `ImpressaoDivergente`, e nenhum hash e' atualizado para aceitar o conteudo mudado | `arquivo.rs` | `tp12_impressao_ou_conteudo_alterados_*` |
| PER-15 | O instante vai em **dois campos** (`local` + `offset_segundos`): a cadeia RFC3339 perderia os segundos do offset | `arquivo.rs` | `tp06_offset_com_segundos_e_nanossegundos_*`, `tp06_mesmo_instante_absoluto_*`, `tp06_offset_impossivel_*` |
| PER-16 | Historia: o mesmo conteudo noutro lote da' **dois** snapshots, e uma correcao a total invariado muda a impressao | `arquivo.rs` | `tp09_*` (dois testes), `tp07_duas_linhas_com_o_mesmo_payload_*` |
| PER-17 | Append-only **logico**: as declaracoes comparam-se como **multiconjunto**, logo duas ocorrencias identicas nao podem virar uma. Snapshots e execucoes comparam-se pela **codificacao exata**, nao por `PartialEq`, porque `10.00 == 10.0000` para `Decimal` e a escala e' um dado conservado | `arquivo.rs` | `tp16_*` (dois testes), `r1_acrescimo_recusa_mudanca_de_escala_*` (dois testes), `r1_candidato_identico_depois_do_round_trip_*` |
| PER-04 | Um so' `lote.id` por execucao, imposto pelo **validador** e nao so' pelo construtor: duas execucoes com o mesmo ID dariam duas identidades de plano para o mesmo lote e um referencia ambiguo | `arquivo.rs` | `r3_execucao_duplicada_e_rejeitada_pelo_validador` |
| PER-09 | Bytes nao interpretaveis tem **impressao** mesmo assim: `observar` responde `token: None`, `candidatos` continua a listar os outros, `recuperar` compara o estado e conserva os bytes corrompidos como prova | `arquivo.rs`, `arquivo_io.rs` | `r2_*` (sete testes) |
| PER-18/22 | Ordem fisica canonica e bytes deterministas; o ultimo nivel da ordem e' o **record** serializado, nao a declaracao, porque `impressao_evento` nao e' um campo dela | `arquivo.rs` | `tp17_*` (dois testes), `tp22_*` (dois testes) |
| PER-19 | Apresentacao mudada e' uma **divergencia**, nao um erro: as decisoes ficam | `arquivo.rs` | `tp18_*` (dois testes) |
| PER-21 | Um so ponto de commit (`fs::rename`); tudo o que falha antes deixa o corrente **intacto byte a byte** | `arquivo_io.rs` | `tp13_falhas_antes_do_commit_*`, `tp13_corrente_corrompido_*` |
| PER-23 | Numeracao sob lucchetto, com **base completa**: o token leva revisao **e** impressao do conteudo, porque dois arquivos diferentes podem ter o mesmo contador | `arquivo_io.rs` | `tp14_duas_publicacoes_da_mesma_base_*`, `tp14_mesmo_contador_de_outro_arquivo_nao_basta` |
| PER-24 | Lucchetto cooperativo com `create_new`, liberacao **explicita** e confirmacao do conteudo: ninguem desbloqueia por outro | `arquivo_io.rs` | `tp14_lucchetto_realmente_tomado_*` |
| PER-06/08 | Recuperacao explicita: candidatos classificados e **validados** sem carregar, estado esperado conferido **antes** de tocar em nada, corrente conservado como prova e candidato **copiado**, nao movido | `arquivo_io.rs` | `tp15_candidatos_*`, `tp15b_*` (cinco testes) |
| D7(a) | O arquivo e' autoritativo **entre processos**: quem escreve e quem le sao dois processos distintos | `tests/` | `tp19_persistencia_entre_processos` |

Superficie publica nova:

```rust
// arquivo.rs — puro
pub fn serializar(&Arquivo, revisao: u64) -> Result<String, ErroArquivo>;
pub fn desserializar(&str) -> Result<(Arquivo, u64), ErroArquivo>;
pub fn desserializar_bytes(&[u8]) -> Result<(Arquivo, u64), ErroArquivo>;
pub fn verificar_estrutura(&Arquivo) -> Result<(), ErroArquivo>;
pub fn verificar_apresentacao(&Arquivo, &PlanoContas)
    -> Result<Vec<Divergencia>, ErroArquivo>;
pub fn verificar_acrescimo(corrente: &Arquivo, candidato: &Arquivo)
    -> Result<(), ErroArquivo>;
pub fn decimal_exato(&str) -> Result<Decimal, ErroArquivo>;
impl Arquivo { novo, acrescentar_lote, acrescentar_eventos,
               contextos_do_lote, eventos }

// arquivo_io.rs — IO
pub fn observar(&Destino) -> Result<EstadoObservado, ErroIo>;
pub fn ler(&Destino) -> Result<Leitura, ErroIo>;
pub fn publicar(&Destino, &Arquivo, Option<&TokenBase>, &MetadadosLucchetto)
    -> Result<Publicacao, ErroIo>;
pub fn candidatos(&Destino) -> Result<Candidatos, ErroIo>;
pub fn recuperar(&Destino, &Path, &EstadoObservado, &MetadadosLucchetto)
    -> Result<Publicacao, ErroIo>;
pub fn liberar(&Destino, &MetadadosLucchetto) -> Result<(), ErroIo>;
```

**Diferenca tecnica a declarar.** As etiquetas dos enums no arquivo vem da
representacao `serde` dos **proprios tipos** do modelo, nao de uma tabela
paralela: ha' uma so verdade. Por isso `TipoConta` e' `snake_case` (`"custo"`)
enquanto os outros sao `SCREAMING_SNAKE_CASE`, e `StatusDado::NaoAplicavel` e'
`"N.A."`. O teste `etiquetas_dos_enums_do_arquivo` fixa cada uma. Excecao
documentada: `OperacaoRevisao` so deriva `Serialize` e `revisao.rs` e' auditado
e nao se toca, logo o arquivo tem um `OperacaoDto` local — e o mesmo teste
compara as tres etiquetas com `OperacaoRevisao::rotulo()`, de modo que uma
divergencia futura falha em vez de mudar o formato em silencio.

**Quatro achados do audit independente de 2026-10-22**, todos corrigidos com
regressao permanente:

1. **R3C3B1-01**: o confronto append-only usava `PartialEq`, e para `Decimal`
   isso e' igualdade **numerica**: `10.00 == 10.0000`. A impressao canonica
   normaliza a escala antes de comparar, logo tambem nao podia ver a mudanca —
   eram dois controles cegos para a mesma coisa. Mudar a escala original
   passava pela publicacao ordinaria. Agora snapshots e execucoes comparam-se
   pela **codificacao exata**, a mesma que vai para o disco.
2. **R3C3B1-02**: a leitura dependia de `read_to_string`, logo um corrente
   corrompido na **codificacao** (bytes nao UTF-8) fazia falhar `observar`,
   `candidatos`, `ler` e — pior — o proprio `recuperar`, que nao chegava a
   comparar o estado esperado. Aquisicao dos bytes e interpretacao do texto
   estao agora separadas: `ErroArquivo::NaoUtf8` e' um erro de **conteudo**,
   nao de IO, e nenhuma conversao com perda fabrica um texto.
3. **R3C3B1-03**: a unicidade de `lote.id` era imposta so' por
   `Arquivo::acrescentar_lote`, mas o modelo e' publico e o leitor nao passa
   por esse construtor. Duas execucoes com o mesmo ID passavam a validacao.
   Agora o validador partilhado recusa com `ExecucaoConflitante`, **sem**
   deduplicar escolhendo a primeira ou a ultima.
4. **R3C3B1-04**: `resumo` cortava a 120 **bytes**, podendo dividir um
   caractere: a funcao que devia **relatar** `HistoriaTruncada` entrava em
   panico. Agora abrevia num limite de caractere.

**Dois achados que os testes deste bloco produziram** e que valem como
registro:

1. `sync_all` sobre um handle de **leitura** e' negado no Windows
   (`PermissionDenied`, codigo 5). A copia da geracao e a prova eram abertas
   com `File::open`: a publicacao falhava sempre que havia geracao a
   conservar. Corrigido com um helper que abre em escrita antes de
   sincronizar. Nenhum teste anterior cobria o caso, porque a primeira
   publicacao nao conserva nada.
2. Depois de uma falha nos passos 6–7, o temporario escrito no passo 5
   **permanece**, e a tentativa seguinte para com `ResiduoPresente`. Nao e' um
   descuido: §9.3 da especificacao proibe a biblioteca de apagar um candidato,
   mesmo o seu proprio rascunho abandonado. A consequencia operacional e' que
   retomar exige mover o residuo — e, depois de um rollback intencional,
   tambem a geracao conservada, que D16 proibe sobrescrever. Esta' em
   `tp15b_rollback_intencional_e_declarado_nao_renumerado`.

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
| `sha2` | `hash_registro` dos lancamentos (3B) e impressoes canonicas (3C.3.a) | adicionada em 2026-10-16 com autorizacao explicita; mesmo payload de quatro campos do Python, para que os codigos continuem comparaveis com o historico |

| `serde_json` | formato do arquivo autoritativo (3C.3.b.1) | adicionada em 2026-10-22 com autorizacao explicita. Ja' estava em `Cargo.lock` como dependencia **opcional** de `rust_decimal` e na cache local, logo o `diff` do lockfile e' de **uma linha** e nada foi baixado |

A fase 3C.3.a **nao acrescentou nenhuma dependencia**: `sha2`, `chrono` (com
`DateTime<FixedOffset>` para o instante declarado) e `unicode-normalization`
(NFC das impressoes) ja' estavam no `Cargo.toml`.

A 3C.3.b.1 acrescentou **uma**, `serde_json = "1.0.154"`, autorizada no
mandato. Porque JSON e nao YAML: `serde_yaml` esta' **deprecado upstream** e
seria uma segunda verdade sobre o formato de persistencia; porque nao um
formato proprio: a garantia que o arquivo precisa — recusar uma chave
duplicada, recusar um campo desconhecido — vem de struct com
`deny_unknown_fields`, que um parser escrito a mao teria de reimplementar.

## Verificacao

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

337 testes: 319 de unidade na lib, 3 no binario, 15 de integracao (13 de CLI e
2 de persistencia entre processos). Os 70 novos sao da fase 3C.3.b.1 (47 em
`arquivo.rs`, 21 em `arquivo_io.rs`, 2 em
`tests/persistencia_entre_processos.rs`); os 267 anteriores continuam verdes
**sem nenhuma alteracao**.

Treze desses 70 sao regressoes dos quatro achados do audit de 2026-10-22
(R3C3B1-01/02/03/04), escritas **antes** da correcao e com as expectativas
corretas: falharam — tres por asserccao e uma por panico no proprio caminho de
erro —, depois passaram. Os quatro sao reproduziveis pela API publica.

Os testes de IO trabalham em arenas sob a directoria temporaria do sistema
(`%TEMP%\fantozzi_3c3b1\`), uma por teste, e **nao** apagam nada fora dela. Um
`TEMP` nao gravavel faz falhar os testes de IO sem que haja nada de errado no
codigo.

O teste entre processos reexecuta o **proprio binario de teste** com
`std::env::current_exe()` e `--exact`, passando o papel por variavel de
ambiente: um processo publica, outro le, um terceiro encontra o lucchetto e
para. Verifica tambem que o filho declarou ter executado um teste, porque um
filtro sem correspondencia sairia com sucesso **sem executar nada** e o papel
pareceria superado.

Controle fisico independente do ficheiro publicado (TP-20): os bytes sao
examinados sem passar pelo codec (BOM, CR, `}` + LF final, monetario textual,
`null` explicito, offset em segundos, contagens de ocorrencias). A contraprova
com uma **segunda** ferramenta foi feita a mao e registada no apendice da
especificacao: sobre o **arquivo de prova completo** (dois snapshots, duas
declaracoes, publicado por `publicar` num destino novo),
`Get-FileHash -Algorithm SHA256` do PowerShell deu `fb871504…a05` sobre 10557
bytes, **identico** ao valor calculado pelo crate. O ficheiro do teste **entre
processos** e' outro conteudo e tem outra medida: um hash nao se transfere de
um para o outro.

Seis desses 59 sao regressoes dos tres achados do audit de 2026-10-21
(R3C3A-01/02/03), escritos **antes** da correcao e com as expectativas
corretas: falharam, depois passaram. Os tres sao reproduziveis pela API
publica, e nenhum deles era coberto pelos testes anteriores.

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
10. **A revisao conserva-se, mas continua nao autenticada.** A 3C.3.b.1 fechou
    a parte da persistencia: os eventos e os contextos originais ficam num
    arquivo autoritativo que sobrevive ao processo. O `autor` continua a ser
    uma cadeia **declarada**, nao uma identidade verificada: o sistema registra
    quem **disse** ter aprovado, nao quem aprovou. Autenticacao e' uma decisao
    separada, que **nao** existe hoje.
11. **Mudar o texto de um aviso muda as impressoes** da 3C.3.a: os `detalhe`
    entram nos bytes canonicos. E' um limite declarado, nao um descuido —
    qualquer alteracao desse tipo exige `IMPRESSAO-CONTEUDO/v2` e a
    conservacao das impressoes `v1`, senao as aprovacoes antigas deixam de
    casar com o conteudo que descrevem.
12. **Durabilidade depois de perda de alimentacao: `NAO_VERIFICADO`.** No
    Windows nao se pode sincronizar uma **directoria** (`sync_all` sobre a
    directoria da' `PermissionDenied`, medido). O conteudo escrito e
    sincronizado nao fica lacerado, mas **quais nomes** sobrevivem — o
    corrente, o temporario, a geracao acabada de copiar — nao e' demonstravel.
    Nao se promete o contrario; o recupero lista e valida os candidatos e nao
    afirma qual deveria existir.
13. **O lucchetto e' cooperativo.** Protege contra outra copia **deste**
    programa, nao contra um editor de texto, um backup ou um script que
    escreva no ficheiro. Um escritor nao cooperante e' detectado **depois**,
    pela base desatualizada ou pela impressao divergente, nunca impedido.
14. **O arquivo cresce sem limite.** D16 decidiu nao podar: cada publicacao
    conserva a geracao anterior e nenhuma prova e' apagada. Quem opera tem de
    gerir o espaco, e a biblioteca **nao** apaga nada por iniciativa propria.
15. **Retomar depois de uma falha ou de um rollback exige um gesto humano.**
    O temporario abandonado e a geracao conservada nao sao sobrescritos nem
    apagados: a publicacao seguinte para com `ResiduoPresente` ou
    `GeracaoJaPresente` ate' que o operador os mova. E' o preco de §9.3 e de
    D16, nao um bloqueio acidental.
16. **`PublicacaoComLucchettoResiduo` nao esta' demonstrada por teste.** O
    campo `lucchetto_residuo` fica `true` quando o commit passou e a liberacao
    falhou; reproduzi-lo exigiria injectar uma falha entre os passos 7 e 8, o
    que a API publica nao permite. Os testes provam que numa publicacao normal
    o campo e' `false` e o lucchetto desaparece; o caso `true` e'
    **NAO_VERIFICADO**.
17. **O arquivo numera, mas nao datilografa a historia de um documento.**
    D17(a) deixou a sequencia historica como `NAO_DISPONIVEL`:
    `revisao_arquivo` conta publicacoes do ficheiro, nao versoes de um
    documento. Depois de um rollback o mesmo numero e' reutilizado — dado
    declarado, nao efeito colateral escondido.
