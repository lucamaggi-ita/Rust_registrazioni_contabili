//! Porte Rust de `Programma_registrazioni-contabili` (agente Fantozzi).
//!
//! Fase 1: modelo de dados de dominio ([`modelos`]) e leitura contratualizada
//! das regras versionadas em `config/` ([`regras`]).
//!
//! Fase 2 (pendente): parser de NF-e com `quick-xml`, normalizacao de texto e
//! classificacao de produtos/despesas.

pub mod modelos;
pub mod regras;
