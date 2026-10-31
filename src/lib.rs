//! Porte Rust de `Programma_registrazioni-contabili` (agente Fantozzi).
//!
//! Fase 1: modelo de dados de dominio ([`modelos`]) e leitura contratualizada
//! das regras versionadas em `config/` ([`regras`]).
//!
//! Fase 2: leitura de NF-e a partir de XML ([`nfe`]), normalizacao de texto
//! ([`normalizacao`]) e classificacao de produtos e despesas
//! ([`classificacao`]).
//!
//! Fase 3B: alocacao de NF-e em lancamentos com reconciliacao ([`alocacao`]).
//!
//! Fase 3C.2: pre-livro diario em memoria ([`prelivro`]).
//!
//! Fase 3C.3.a: impressoes canonicas ([`impressao`]) e revisao humana
//! ([`revisao`]).
//!
//! Fase 3C.3.b: arquivo autoritativo ([`arquivo`]) e seu protocolo de IO
//! ([`arquivo_io`]).
//!
//! Fase 3C.4.1: export derivado ([`export`]), adaptador do formato ([`xlsx`])
//! e escrita do arquivo derivado ([`export_io`]).
//!
//! Fase 3C.5.a: composicao da importacao usada pela CLI ([`importacao`]).
//!
//! Fase 3C.5.b/3D (pendentes): resto do uso pela CLI e DRE gerencial.

pub mod alocacao;
pub mod arquivo;
pub mod arquivo_io;
pub mod classificacao;
pub(crate) mod dinheiro;
pub mod export;
pub mod export_io;
pub mod importacao;
pub mod impressao;
pub mod modelos;
pub mod nfe;
pub mod normalizacao;
pub mod prelivro;
pub mod regras;
pub mod revisao;
pub mod xlsx;
