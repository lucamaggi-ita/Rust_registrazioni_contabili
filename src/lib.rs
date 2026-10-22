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
//! Fase 3C/3D (pendentes): pre-livro diario, revisao, exportacao e DRE.

pub mod alocacao;
pub mod arquivo;
pub mod arquivo_io;
pub mod classificacao;
pub(crate) mod dinheiro;
pub mod impressao;
pub mod modelos;
pub mod nfe;
pub mod normalizacao;
pub mod prelivro;
pub mod regras;
pub mod revisao;
