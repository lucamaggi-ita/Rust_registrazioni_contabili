//! Porte Rust de `Programma_registrazioni-contabili` (agente Fantozzi).
//!
//! Fase 1: modelo de dados de dominio ([`modelos`]) e leitura contratualizada
//! das regras versionadas em `config/` ([`regras`]).
//!
//! Fase 2: leitura de NF-e a partir de XML ([`nfe`]), normalizacao de texto
//! ([`normalizacao`]) e classificacao de produtos e despesas
//! ([`classificacao`]).
//!
//! Fase 3 (pendente): alocacao em lancamentos, pre-livro diario, DRE gerencial.

pub mod classificacao;
pub mod modelos;
pub mod nfe;
pub mod normalizacao;
pub mod regras;
