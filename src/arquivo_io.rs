//! Lettura, pubblicazione e recupero dell'archivio (fase 3C.3.b.1).
//!
//! Attua §9 della specifica di fase 3C.3.b: protocollo con **un solo** punto
//! di commit, generazioni conservate, recupero vincolato allo stato osservato
//! e concorrenza con lucchetto cooperativo piu' base completa verificata.
//!
//! # I fatti su cui il protocollo si appoggia
//!
//! Misurati su Windows e registrati in §3.6 della specifica:
//!
//! - `fs::rename` **sostituisce** una destinazione esistente, anche tenuta da
//!   un handle aperto da Rust. Non serve cancellare prima;
//! - `remove_file` riesce anche su un file **aperto**, quindi "cancella poi
//!   rinomina" lascerebbe una finestra senza archivio: e' vietato;
//! - `File::create` su un file aperto lo **tronca**: i temporanei e le
//!   generazioni si creano con `create_new`, mai troncando un residuo;
//! - `create_new` ripetuto fallisce con `AlreadyExists`: e' il lucchetto
//!   **cooperativo**;
//! - `sync_all` su una **directory** e' negato: la durabilita' della voce di
//!   directory creata dal `rename` resta `NON_VERIFICATA`, e questo modulo non
//!   promette il contrario.
//!
//! # Che cosa questo modulo non fa
//!
//! Non sceglie percorsi, non legge variabili d'ambiente, non consulta
//! l'orologio: percorso, autore e istante arrivano dal chiamante. Non cancella
//! generazioni (D16) e non ripara archivi.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::arquivo::{self, Arquivo, ErroArquivo};

/// Dove vive l'archivio. **Sempre** dal chiamante.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Destino {
    pub diretorio: PathBuf,
    pub nome: String,
}

impl Destino {
    pub fn novo(diretorio: impl Into<PathBuf>, nome: impl Into<String>) -> Self {
        Destino {
            diretorio: diretorio.into(),
            nome: nome.into(),
        }
    }

    pub fn corrente(&self) -> PathBuf {
        self.diretorio.join(&self.nome)
    }

    fn lucchetto(&self) -> PathBuf {
        self.diretorio.join(format!("{}.lock", self.nome))
    }

    fn temporario(&self, revisao: u64) -> PathBuf {
        self.diretorio.join(format!("{}.novo.{revisao}", self.nome))
    }

    fn geracao(&self, revisao: u64) -> PathBuf {
        self.diretorio.join(format!("{}.ger{revisao}", self.nome))
    }

    fn prova(&self, etiqueta: &str) -> PathBuf {
        self.diretorio
            .join(format!("{}.corrupto{etiqueta}", self.nome))
    }

    fn recuperado(&self, revisao: u64) -> PathBuf {
        self.diretorio
            .join(format!("{}.recuperado.{revisao}", self.nome))
    }
}

/// Che cosa il chiamante ha letto: **non** solo il contatore, perche' due
/// archivi diversi possono condividerlo (§8.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenBase {
    pub revisao: u64,
    pub sha256_conteudo: String,
}

/// Stato della destinazione come il chiamante l'ha **osservato**.
///
/// L'impronta dei byte funziona anche su un file che non si riesce a
/// deserializzare: e' cio' che rende confrontabile uno stato corrotto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EstadoObservado {
    DestinoAusente,
    Corrente {
        sha256_bytes: String,
        token: Option<TokenBase>,
    },
}

/// Metadati del lucchetto: **dichiarati**, non dedotti dall'ambiente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadadosLucchetto {
    pub operador: String,
    pub instante: DateTime<FixedOffset>,
    pub motivo: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LucchettoDto {
    operador: String,
    instante_local: String,
    offset_segundos: i32,
    motivo: String,
}

impl MetadadosLucchetto {
    fn texto(&self) -> String {
        let dto = LucchettoDto {
            operador: self.operador.clone(),
            instante_local: self
                .instante
                .naive_local()
                .format("%Y-%m-%dT%H:%M:%S%.9f")
                .to_string(),
            offset_segundos: self.instante.offset().local_minus_utc(),
            motivo: self.motivo.clone(),
        };
        // Il lucchetto non e' l'archivio: la sua forma e' locale a questo
        // modulo e non passa per il codec di `arquivo.rs`.
        serde_json::to_string(&dto).unwrap_or_else(|_| "{}".to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Leitura {
    pub arquivo: Arquivo,
    pub base: TokenBase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Publicacao {
    pub revisao_publicada: u64,
    /// `None` alla prima pubblicazione: non c'e' niente da conservare.
    pub geracao_anterior: Option<PathBuf>,
    /// `true` se il commit e' avvenuto ma il rilascio del lucchetto e'
    /// fallito: la pubblicazione **e' valida**.
    pub lucchetto_residuo: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PapelCandidato {
    Corrente,
    Geracao,
    TemporarioRestante,
    Prova,
}

impl PapelCandidato {
    pub const fn rotulo(self) -> &'static str {
        match self {
            PapelCandidato::Corrente => "CORRENTE",
            PapelCandidato::Geracao => "GERACAO",
            PapelCandidato::TemporarioRestante => "TEMPORARIO_RESTANTE",
            PapelCandidato::Prova => "PROVA",
        }
    }
}

/// Un candidato al recupero, **validato** ma non caricato.
#[derive(Debug)]
pub struct CandidatoRecuperacao {
    pub caminho: PathBuf,
    pub papel: PapelCandidato,
    pub revisao_arquivo: Option<u64>,
    pub validacao: Result<(), ErroArquivo>,
    pub snapshots: usize,
    pub eventos: usize,
}

/// Elenco dei candidati **piu'** lo stato osservato, da ripassare a
/// [`recuperar`] senza ricostruirlo.
#[derive(Debug)]
pub struct Candidatos {
    pub estado: EstadoObservado,
    pub candidatos: Vec<CandidatoRecuperacao>,
}

/// Errore dello strato IO. Ogni variante nomina l'operazione e la fonte.
#[derive(Debug)]
pub enum ErroIo {
    DestinoAusente {
        caminho: PathBuf,
    },
    DestinoJaExistente {
        caminho: PathBuf,
    },
    ArquivoCorrenteInvalido {
        caminho: PathBuf,
        causa: ErroArquivo,
    },
    ResiduoPresente {
        caminho: PathBuf,
    },
    GeracaoJaPresente {
        caminho: PathBuf,
    },
    ProvaJaPresente {
        caminho: PathBuf,
    },
    ArquivoOcupado {
        lucchetto: PathBuf,
        conteudo_declarado: String,
    },
    LucchettoAusente {
        caminho: PathBuf,
    },
    ConfirmacaoNaoCoincide {
        lucchetto: PathBuf,
        conteudo_declarado: String,
    },
    BaseDesatualizada {
        esperada: TokenBase,
        encontrada: TokenBase,
    },
    EstadoMudou {
        esperado: EstadoObservado,
        encontrado: EstadoObservado,
    },
    ContadorEsgotado {
        revisao_corrente: u64,
    },
    CandidatoInvalido {
        caminho: PathBuf,
        causa: ErroArquivo,
    },
    Conteudo(ErroArquivo),
    Io {
        operacao: &'static str,
        caminho: PathBuf,
        erro: std::io::Error,
    },
}

impl std::fmt::Display for ErroIo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErroIo::DestinoAusente { caminho } => {
                write!(f, "destino ausente: {}", caminho.display())
            }
            ErroIo::DestinoJaExistente { caminho } => write!(
                f,
                "destino ja' existente: {}. A inicializacao nao escreve sobre nada",
                caminho.display()
            ),
            ErroIo::ArquivoCorrenteInvalido { caminho, causa } => write!(
                f,
                "arquivo corrente invalido ({}): {causa}. Nao se publica sobre um \
                 arquivo que nao se sabe ler; o recuperar e' outra chamada",
                caminho.display()
            ),
            ErroIo::ResiduoPresente { caminho } => write!(
                f,
                "residuo presente: {}. Nao e' truncado: examine-o com candidatos()",
                caminho.display()
            ),
            ErroIo::GeracaoJaPresente { caminho } => write!(
                f,
                "geracao ja' presente: {}. Nenhuma copia conservada e' sobrescrita",
                caminho.display()
            ),
            ErroIo::ProvaJaPresente { caminho } => write!(
                f,
                "prova ja' presente: {}. Um arquivo corrompido e' uma prova e nao \
                 e' sobrescrito",
                caminho.display()
            ),
            ErroIo::ArquivoOcupado {
                lucchetto,
                conteudo_declarado,
            } => write!(
                f,
                "arquivo ocupado, lucchetto {}: {conteudo_declarado}",
                lucchetto.display()
            ),
            ErroIo::LucchettoAusente { caminho } => {
                write!(f, "nenhum lucchetto para liberar em {}", caminho.display())
            }
            ErroIo::ConfirmacaoNaoCoincide {
                lucchetto,
                conteudo_declarado,
            } => write!(
                f,
                "confirmacao nao coincide com o lucchetto {}: {conteudo_declarado}. \
                 Nenhum lucchetto e' removido sem que o chamante diga qual",
                lucchetto.display()
            ),
            ErroIo::BaseDesatualizada {
                esperada,
                encontrada,
            } => write!(
                f,
                "base desatualizada: declarada revisao {} impressao {}, encontrada \
                 revisao {} impressao {}",
                esperada.revisao,
                resumo(&esperada.sha256_conteudo),
                encontrada.revisao,
                resumo(&encontrada.sha256_conteudo)
            ),
            ErroIo::EstadoMudou {
                esperado,
                encontrado,
            } => write!(
                f,
                "o estado do destino mudou entre a observacao e o recuperar: \
                 esperado {}, encontrado {}. Nada foi tocado: observe de novo e \
                 confirme",
                descrever(esperado),
                descrever(encontrado)
            ),
            ErroIo::ContadorEsgotado { revisao_corrente } => write!(
                f,
                "contador de revisao esgotado em {revisao_corrente}: nenhum numero \
                 e' reutilizado"
            ),
            ErroIo::CandidatoInvalido { caminho, causa } => write!(
                f,
                "candidato invalido ({}): {causa}. Nada foi tocado",
                caminho.display()
            ),
            ErroIo::Conteudo(erro) => write!(f, "conteudo invalido: {erro}"),
            ErroIo::Io {
                operacao,
                caminho,
                erro,
            } => write!(
                f,
                "falha de io em {operacao} sobre {}: {erro}",
                caminho.display()
            ),
        }
    }
}

impl std::error::Error for ErroIo {}

fn resumo(hex: &str) -> String {
    hex.chars().take(16).collect()
}

fn descrever(estado: &EstadoObservado) -> String {
    match estado {
        EstadoObservado::DestinoAusente => "destino ausente".to_string(),
        EstadoObservado::Corrente {
            sha256_bytes,
            token,
        } => match token {
            Some(t) => format!(
                "corrente valido revisao {} bytes {}",
                t.revisao,
                resumo(sha256_bytes)
            ),
            None => format!("corrente ilegivel bytes {}", resumo(sha256_bytes)),
        },
    }
}

fn io(operacao: &'static str, caminho: &Path, erro: std::io::Error) -> ErroIo {
    ErroIo::Io {
        operacao,
        caminho: caminho.to_path_buf(),
        erro,
    }
}

/// Legge i **byte** di un file, senza interpretarli.
///
/// Riscontro R3C3B1-02 del 2026-10-22: la lettura era subordinata a
/// `read_to_string`, quindi un corrente corrotto nella **codifica** faceva
/// fallire l'osservazione e bloccava il protocollo di recupero prima che
/// potesse partire. Acquisizione e interpretazione sono due passi distinti: un
/// errore qui e' un errore di IO vero, e l'interpretazione dei byte e' un
/// problema di contenuto, nominato da [`arquivo::ErroArquivo`].
fn ler_bytes(caminho: &Path) -> Result<Vec<u8>, ErroIo> {
    let mut bytes = Vec::new();
    File::open(caminho)
        .map_err(|e| io("abrir", caminho, e))?
        .read_to_end(&mut bytes)
        .map_err(|e| io("ler", caminho, e))?;
    Ok(bytes)
}

/// Legge un file che **deve** essere testo: il lucchetto, che questo modulo
/// scrive da sé.
fn ler_texto(caminho: &Path) -> Result<String, ErroIo> {
    let mut texto = String::new();
    File::open(caminho)
        .map_err(|e| io("abrir", caminho, e))?
        .read_to_string(&mut texto)
        .map_err(|e| io("ler", caminho, e))?;
    Ok(texto)
}

/// Forza su disco un file **gia' scritto**, per esempio una copia fatta con
/// [`fs::copy`].
///
/// L'handle va aperto in scrittura: su Windows `sync_all` su un handle di sola
/// lettura e' negato con `PermissionDenied` (`Acesso negado`, codice 5). Il
/// fatto e' stato misurato dai test di questo modulo, non supposto.
fn sincronizar(
    operacao_abrir: &'static str,
    operacao_sync: &'static str,
    caminho: &Path,
) -> Result<(), ErroIo> {
    OpenOptions::new()
        .write(true)
        .open(caminho)
        .map_err(|e| io(operacao_abrir, caminho, e))?
        .sync_all()
        .map_err(|e| io(operacao_sync, caminho, e))
}

/// Scrive un file **nuovo**: `create_new`, quindi mai un troncamento.
///
/// Prende **byte**: il recupero promuove il candidato cosi come sta sul
/// disco, senza passare per una conversione di testo.
fn escrever_novo(caminho: &Path, conteudo: &[u8]) -> Result<(), ErroIo> {
    let mut arquivo = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(caminho)
        .map_err(|e| io("criar", caminho, e))?;
    arquivo
        .write_all(conteudo)
        .map_err(|e| io("escrever", caminho, e))?;
    arquivo
        .sync_all()
        .map_err(|e| io("sincronizar", caminho, e))
}

// ---------------------------------------------------------------------------
// Lucchetto cooperativo
// ---------------------------------------------------------------------------

struct Trava {
    caminho: PathBuf,
    liberada: bool,
}

impl Trava {
    fn tomar(destino: &Destino, metadados: &MetadadosLucchetto) -> Result<Self, ErroIo> {
        let caminho = destino.lucchetto();
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&caminho)
        {
            Ok(mut arquivo) => {
                arquivo
                    .write_all(metadados.texto().as_bytes())
                    .map_err(|e| io("escrever lucchetto", &caminho, e))?;
                let _ = arquivo.sync_all();
                Ok(Trava {
                    caminho,
                    liberada: false,
                })
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let conteudo = ler_texto(&caminho).unwrap_or_else(|_| "(ilegivel)".to_string());
                Err(ErroIo::ArquivoOcupado {
                    lucchetto: caminho,
                    conteudo_declarado: conteudo,
                })
            }
            Err(e) => Err(io("criar lucchetto", &caminho, e)),
        }
    }

    /// Rilascio **esplicito**: l'esito e' un dato, non un dettaglio.
    fn liberar(mut self) -> bool {
        self.liberada = true;
        fs::remove_file(&self.caminho).is_ok()
    }
}

impl Drop for Trava {
    fn drop(&mut self) {
        if !self.liberada {
            // Percorso d'errore: rilascio di cortesia. L'esito interessa solo
            // dopo il commit, e la' il rilascio e' esplicito.
            let _ = fs::remove_file(&self.caminho);
        }
    }
}

// ---------------------------------------------------------------------------
// Osservazione e lettura
// ---------------------------------------------------------------------------

/// Osserva la destinazione **senza interpretarla**: funziona anche su un file
/// che non si riesce a deserializzare.
pub fn observar(destino: &Destino) -> Result<EstadoObservado, ErroIo> {
    let caminho = destino.corrente();
    if !caminho.exists() {
        return Ok(EstadoObservado::DestinoAusente);
    }
    let bytes = ler_bytes(&caminho)?;
    let sha256_bytes = arquivo::sha256_dos_bytes(&bytes);
    let token = match arquivo::desserializar_bytes(&bytes) {
        Ok((modelo, revisao)) if arquivo::verificar_estrutura(&modelo).is_ok() => Some(TokenBase {
            revisao,
            sha256_conteudo: sha256_bytes.clone(),
        }),
        _ => None,
    };
    Ok(EstadoObservado::Corrente {
        sha256_bytes,
        token,
    })
}

/// Legge e **valida** l'archivio corrente. Non esiste un percorso che
/// restituisca un archivio non validato.
pub fn ler(destino: &Destino) -> Result<Leitura, ErroIo> {
    let caminho = destino.corrente();
    if !caminho.exists() {
        return Err(ErroIo::DestinoAusente { caminho });
    }
    let bytes = ler_bytes(&caminho)?;
    let (modelo, revisao) =
        arquivo::desserializar_bytes(&bytes).map_err(|causa| ErroIo::ArquivoCorrenteInvalido {
            caminho: caminho.clone(),
            causa,
        })?;
    arquivo::verificar_estrutura(&modelo).map_err(|causa| ErroIo::ArquivoCorrenteInvalido {
        caminho: caminho.clone(),
        causa,
    })?;
    Ok(Leitura {
        base: TokenBase {
            revisao,
            sha256_conteudo: arquivo::sha256_dos_bytes(&bytes),
        },
        arquivo: modelo,
    })
}

// ---------------------------------------------------------------------------
// Pubblicazione
// ---------------------------------------------------------------------------

/// Esegue i passi 1–8 di §9.1.
///
/// `base = None` significa **inizializzazione** (§9.1.1): nessun numero
/// inventato e nessuna generazione da copiare.
pub fn publicar(
    destino: &Destino,
    candidato: &Arquivo,
    base: Option<&TokenBase>,
    lucchetto: &MetadadosLucchetto,
) -> Result<Publicacao, ErroIo> {
    // Passo 0: valida la struttura. Nessun file e' stato toccato.
    arquivo::verificar_estrutura(candidato).map_err(ErroIo::Conteudo)?;

    // Passo 1: lucchetto.
    let trava = Trava::tomar(destino, lucchetto)?;
    let corrente = destino.corrente();

    // Passi 2-4: rileggi, confronta, verifica l'accrescimento, numera.
    let (revisao_base, nova_revisao) = match base {
        None => {
            if corrente.exists() {
                return Err(ErroIo::DestinoJaExistente { caminho: corrente });
            }
            (None, 1u64)
        }
        Some(token) => {
            if !corrente.exists() {
                return Err(ErroIo::DestinoAusente { caminho: corrente });
            }
            let bytes = ler_bytes(&corrente)?;
            let sha = arquivo::sha256_dos_bytes(&bytes);
            let (modelo, revisao) = arquivo::desserializar_bytes(&bytes).map_err(|causa| {
                ErroIo::ArquivoCorrenteInvalido {
                    caminho: corrente.clone(),
                    causa,
                }
            })?;
            arquivo::verificar_estrutura(&modelo).map_err(|causa| {
                ErroIo::ArquivoCorrenteInvalido {
                    caminho: corrente.clone(),
                    causa,
                }
            })?;
            if sha != token.sha256_conteudo || revisao != token.revisao {
                return Err(ErroIo::BaseDesatualizada {
                    esperada: token.clone(),
                    encontrada: TokenBase {
                        revisao,
                        sha256_conteudo: sha,
                    },
                });
            }
            arquivo::verificar_acrescimo(&modelo, candidato).map_err(ErroIo::Conteudo)?;
            let nova = revisao.checked_add(1).ok_or(ErroIo::ContadorEsgotado {
                revisao_corrente: revisao,
            })?;
            (Some(revisao), nova)
        }
    };

    // Passo 4 (seconda parte): solo ora si serializza, con il numero
    // verificato sotto lucchetto.
    let texto = arquivo::serializar(candidato, nova_revisao).map_err(ErroIo::Conteudo)?;

    // Passo 5: temporaneo, senza troncare un residuo.
    let temporario = destino.temporario(nova_revisao);
    if temporario.exists() {
        return Err(ErroIo::ResiduoPresente {
            caminho: temporario,
        });
    }
    escrever_novo(&temporario, texto.as_bytes())?;

    // Passo 6: generazione precedente, mai sovrascritta.
    let geracao = match revisao_base {
        None => None,
        Some(revisao) => {
            let destino_geracao = destino.geracao(revisao);
            if destino_geracao.exists() {
                return Err(ErroIo::GeracaoJaPresente {
                    caminho: destino_geracao,
                });
            }
            fs::copy(&corrente, &destino_geracao)
                .map_err(|e| io("copiar geracao", &destino_geracao, e))?;
            sincronizar("abrir geracao", "sincronizar geracao", &destino_geracao)?;
            Some(destino_geracao)
        }
    };

    // Passo 7: PUNTO DI COMMIT.
    fs::rename(&temporario, &corrente).map_err(|e| io("publicar", &corrente, e))?;

    // Passo 8: rilascio. Un fallimento qui non annulla la pubblicazione.
    let liberado = trava.liberar();
    Ok(Publicacao {
        revisao_publicada: nova_revisao,
        geracao_anterior: geracao,
        lucchetto_residuo: !liberado,
    })
}

// ---------------------------------------------------------------------------
// Candidati e recupero
// ---------------------------------------------------------------------------

/// Elenca e **valida** i candidati. Non carica niente in automatico, non
/// cancella niente (D16).
pub fn candidatos(destino: &Destino) -> Result<Candidatos, ErroIo> {
    let estado = observar(destino)?;
    let mut encontrados: Vec<CandidatoRecuperacao> = Vec::new();
    let entradas =
        fs::read_dir(&destino.diretorio).map_err(|e| io("listar", &destino.diretorio, e))?;
    for entrada in entradas {
        let entrada = entrada.map_err(|e| io("listar", &destino.diretorio, e))?;
        let caminho = entrada.path();
        let Some(nome) = caminho.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let papel = if nome == destino.nome {
            PapelCandidato::Corrente
        } else if nome.starts_with(&format!("{}.ger", destino.nome)) {
            PapelCandidato::Geracao
        } else if nome.starts_with(&format!("{}.novo.", destino.nome))
            || nome.starts_with(&format!("{}.recuperado.", destino.nome))
        {
            PapelCandidato::TemporarioRestante
        } else if nome.starts_with(&format!("{}.corrupto", destino.nome)) {
            PapelCandidato::Prova
        } else {
            continue;
        };
        // I byte si acquisiscono sempre; interpretarli puo fallire, e allora il
        // candidato resta elencato come invalido invece di interrompere
        // l'elenco degli altri (riscontro R3C3B1-02).
        let bytes = ler_bytes(&caminho)?;
        let (validacao, revisao, snapshots, eventos) = match arquivo::desserializar_bytes(&bytes) {
            Ok((modelo, revisao)) => {
                let validacao = arquivo::verificar_estrutura(&modelo);
                (
                    validacao,
                    Some(revisao),
                    modelo.snapshots.len(),
                    modelo.eventos.len(),
                )
            }
            Err(erro) => (Err(erro), None, 0, 0),
        };
        encontrados.push(CandidatoRecuperacao {
            caminho,
            papel,
            revisao_arquivo: revisao,
            validacao,
            snapshots,
            eventos,
        });
    }
    encontrados.sort_by(|a, b| a.caminho.cmp(&b.caminho));
    Ok(Candidatos {
        estado,
        candidatos: encontrados,
    })
}

/// Recupero **esplicito** di un candidato (§9.4.2).
///
/// Confronta lo stato effettivo con `esperado` **prima** di conservare il
/// corrente o promuovere il candidato: il lucchetto garantisce la mutua
/// esclusione al commit, non copre il tempo fra l'osservazione del chiamante e
/// l'acquisizione del lucchetto.
pub fn recuperar(
    destino: &Destino,
    candidato: &Path,
    esperado: &EstadoObservado,
    lucchetto: &MetadadosLucchetto,
) -> Result<Publicacao, ErroIo> {
    let trava = Trava::tomar(destino, lucchetto)?;

    // Passo 2: lo stato atteso deve essere ancora quello.
    let encontrado = observar(destino)?;
    if &encontrado != esperado {
        return Err(ErroIo::EstadoMudou {
            esperado: esperado.clone(),
            encontrado,
        });
    }

    // Passo 3: valida il candidato. Niente e' stato toccato.
    let bytes = ler_bytes(candidato)?;
    let (modelo, revisao) =
        arquivo::desserializar_bytes(&bytes).map_err(|causa| ErroIo::CandidatoInvalido {
            caminho: candidato.to_path_buf(),
            causa,
        })?;
    arquivo::verificar_estrutura(&modelo).map_err(|causa| ErroIo::CandidatoInvalido {
        caminho: candidato.to_path_buf(),
        causa,
    })?;

    // Passo 4: il corrente, anche illeggibile, diventa una prova.
    let corrente = destino.corrente();
    let prova = match &encontrado {
        EstadoObservado::DestinoAusente => None,
        EstadoObservado::Corrente { token, .. } => {
            let etiqueta = match token {
                Some(t) => format!("{}", t.revisao),
                None => lucchetto
                    .instante
                    .format("%Y%m%dT%H%M%S%.9f")
                    .to_string()
                    .replace('.', ""),
            };
            let caminho = destino.prova(&etiqueta);
            if caminho.exists() {
                return Err(ErroIo::ProvaJaPresente { caminho });
            }
            fs::copy(&corrente, &caminho).map_err(|e| io("conservar prova", &caminho, e))?;
            sincronizar("abrir prova", "sincronizar prova", &caminho)?;
            Some(caminho)
        }
    };

    // Passo 5: il candidato viene **copiato**, non spostato: resta dov'e'
    // come prova di recupero (D16).
    let temporario = destino.recuperado(revisao);
    if temporario.exists() {
        return Err(ErroIo::ResiduoPresente {
            caminho: temporario,
        });
    }
    escrever_novo(&temporario, &bytes)?;
    fs::rename(&temporario, &corrente).map_err(|e| io("recuperar", &corrente, e))?;

    let liberado = trava.liberar();
    Ok(Publicacao {
        revisao_publicada: revisao,
        geracao_anterior: prova,
        lucchetto_residuo: !liberado,
    })
}

/// Sblocco **esplicito**: il chiamante deve dire quale lucchetto rimuove.
pub fn liberar(destino: &Destino, confirmacao: &MetadadosLucchetto) -> Result<(), ErroIo> {
    let caminho = destino.lucchetto();
    if !caminho.exists() {
        return Err(ErroIo::LucchettoAusente { caminho });
    }
    let conteudo = ler_texto(&caminho)?;
    if conteudo != confirmacao.texto() {
        return Err(ErroIo::ConfirmacaoNaoCoincide {
            lucchetto: caminho,
            conteudo_declarado: conteudo,
        });
    }
    fs::remove_file(&caminho).map_err(|e| io("liberar", &caminho, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arquivo::testes_comuns::*;
    use crate::arquivo::{IdentidadePlano, serializar};
    use crate::revisao::OperacaoRevisao;
    use std::str::FromStr;
    use std::sync::atomic::{AtomicU32, Ordering};

    static SEQUENCIA: AtomicU32 = AtomicU32::new(0);

    /// Directory di lavoro isolata, una per chiamata. Sta sotto la temporanea
    /// di sistema e **non** tocca nessuna cartella del progetto.
    fn arena(nome: &str) -> Destino {
        let unica = format!(
            "{nome}_{}_{}",
            std::process::id(),
            SEQUENCIA.fetch_add(1, Ordering::SeqCst)
        );
        let diretorio = std::env::temp_dir().join("fantozzi_3c3b1").join(unica);
        if diretorio.exists() {
            fs::remove_dir_all(&diretorio).unwrap();
        }
        fs::create_dir_all(&diretorio).unwrap();
        Destino::novo(diretorio, "arquivo.json")
    }

    fn lucchetto(motivo: &str) -> MetadadosLucchetto {
        MetadadosLucchetto {
            operador: "Jos\u{00E9}".to_string(),
            instante: instante(9, -3 * 3600),
            motivo: motivo.to_string(),
        }
    }

    fn bytes_do_corrente(destino: &Destino) -> String {
        arquivo::sha256_dos_bytes(&ler_bytes(&destino.corrente()).unwrap())
    }

    fn com_mais_um_evento(base: &Arquivo, hora: u32) -> Arquivo {
        let mut novo = base.clone();
        novo.acrescentar_eventos(&[evento(
            &um_documento(),
            "2.01.004",
            0,
            OperacaoRevisao::Anotar,
            hora,
            -3 * 3600,
        )]);
        novo
    }

    /// Primo archivio pubblicato: inizializzazione piu' una lettura.
    fn inicializar(destino: &Destino) -> Leitura {
        let publicacao = publicar(destino, &arquivo_valido(), None, &lucchetto("inicio")).unwrap();
        assert_eq!(publicacao.revisao_publicada, 1);
        assert!(publicacao.geracao_anterior.is_none());
        assert!(!publicacao.lucchetto_residuo);
        ler(destino).unwrap()
    }

    // =================================================================
    // TP-13: ciclo di vita reale
    // =================================================================

    #[test]
    fn tp13_destino_ausente_e_primeira_publicacao() {
        let destino = arena("tp13_inicio");
        assert_eq!(observar(&destino).unwrap(), EstadoObservado::DestinoAusente);
        match ler(&destino).unwrap_err() {
            ErroIo::DestinoAusente { caminho } => assert_eq!(caminho, destino.corrente()),
            outro => panic!("erro inesperado: {outro}"),
        }

        let leitura = inicializar(&destino);
        assert_eq!(leitura.base.revisao, 1);
        assert_eq!(
            leitura.arquivo,
            arquivo_valido(),
            "nessuna perdita sul disco"
        );

        // Lo stato osservato porta l'impronta dei byte e il token.
        match observar(&destino).unwrap() {
            EstadoObservado::Corrente {
                sha256_bytes,
                token,
            } => {
                assert_eq!(sha256_bytes, bytes_do_corrente(&destino));
                assert_eq!(token.unwrap(), leitura.base);
            }
            outro => panic!("estado inesperado: {outro:?}"),
        }

        // Una seconda inizializzazione non scrive sopra niente.
        let antes = bytes_do_corrente(&destino);
        match publicar(&destino, &arquivo_valido(), None, &lucchetto("de novo")).unwrap_err() {
            ErroIo::DestinoJaExistente { caminho } => assert_eq!(caminho, destino.corrente()),
            outro => panic!("erro inesperado: {outro}"),
        }
        assert_eq!(bytes_do_corrente(&destino), antes, "corrente intatto");
        assert!(!destino.lucchetto().exists(), "lucchetto rilasciato");
    }

    #[test]
    fn tp13_segunda_publicacao_conserva_a_geracao() {
        let destino = arena("tp13_segunda");
        let leitura = inicializar(&destino);
        let bytes_da_primeira = bytes_do_corrente(&destino);

        let candidato = com_mais_um_evento(&leitura.arquivo, 18);
        let publicacao = publicar(
            &destino,
            &candidato,
            Some(&leitura.base),
            &lucchetto("segunda"),
        )
        .unwrap();
        assert_eq!(publicacao.revisao_publicada, 2);
        assert!(!publicacao.lucchetto_residuo);

        // La generazione precedente e' conservata byte per byte (D16).
        let geracao = publicacao.geracao_anterior.unwrap();
        assert_eq!(geracao, destino.diretorio.join("arquivo.json.ger1"));
        assert_eq!(
            arquivo::sha256_dos_bytes(&ler_bytes(&geracao).unwrap()),
            bytes_da_primeira
        );

        // Il corrente e' il nuovo, e il temporaneo non resta.
        let relida = ler(&destino).unwrap();
        assert_eq!(relida.base.revisao, 2);
        assert_eq!(relida.arquivo, candidato);
        assert!(!destino.diretorio.join("arquivo.json.novo.2").exists());
        assert!(!destino.lucchetto().exists());
    }

    #[test]
    fn tp13_falhas_antes_do_commit_deixam_o_corrente_intacto() {
        let destino = arena("tp13_falhas");
        let leitura = inicializar(&destino);
        let antes = bytes_do_corrente(&destino);
        let candidato = com_mais_um_evento(&leitura.arquivo, 18);

        // Residuo di un tentativo precedente: non si tronca, si segnala.
        let residuo = destino.diretorio.join("arquivo.json.novo.2");
        fs::write(&residuo, "residuo de outra tentativa").unwrap();
        match publicar(&destino, &candidato, Some(&leitura.base), &lucchetto("r")).unwrap_err() {
            ErroIo::ResiduoPresente { caminho } => assert_eq!(caminho, residuo),
            outro => panic!("erro inesperado: {outro}"),
        }
        assert_eq!(
            fs::read_to_string(&residuo).unwrap(),
            "residuo de outra tentativa",
            "il residuo resta come prova"
        );
        assert_eq!(bytes_do_corrente(&destino), antes);
        fs::remove_file(&residuo).unwrap();

        // Generazione gia' presente: nessuna copia conservata si sovrascrive.
        let geracao = destino.diretorio.join("arquivo.json.ger1");
        fs::write(&geracao, "geracao de outra execucao").unwrap();
        match publicar(&destino, &candidato, Some(&leitura.base), &lucchetto("g")).unwrap_err() {
            ErroIo::GeracaoJaPresente { caminho } => assert_eq!(caminho, geracao),
            outro => panic!("erro inesperado: {outro}"),
        }
        assert_eq!(
            fs::read_to_string(&geracao).unwrap(),
            "geracao de outra execucao"
        );
        assert_eq!(bytes_do_corrente(&destino), antes);
        assert!(!destino.lucchetto().exists(), "lucchetto rilasciato");

        // Il temporaneo scritto al passo 5 **resta**: §9.3 vieta alla libreria
        // di cancellare un candidato, anche se e' il proprio tentativo
        // abbandonato. Chi riprende lo esamina e lo sposta: e' una scelta
        // dell'operatore, non un ripulisci automatico.
        assert!(residuo.exists(), "il tentativo abbandonato resta nominato");
        fs::remove_file(&geracao).unwrap();
        fs::rename(&residuo, destino.diretorio.join("exame.novo.2")).unwrap();

        // Candidato strutturalmente invalido: si cade al passo 0, senza
        // nemmeno prendere il lucchetto.
        let mut invalido = candidato.clone();
        invalido.snapshots[0].impressao_conteudo = "0".repeat(64);
        match publicar(&destino, &invalido, Some(&leitura.base), &lucchetto("i")).unwrap_err() {
            ErroIo::Conteudo(ErroArquivo::ImpressaoDivergente { o_que, .. }) => {
                assert_eq!(o_que, "conteudo")
            }
            outro => panic!("erro inesperado: {outro}"),
        }
        assert_eq!(bytes_do_corrente(&destino), antes);
        assert!(!destino.lucchetto().exists());

        // Dopo tutte le rinunce, la pubblicazione legittima riesce.
        let publicacao = publicar(
            &destino,
            &candidato,
            Some(&leitura.base),
            &lucchetto("finalmente"),
        )
        .unwrap();
        assert_eq!(publicacao.revisao_publicada, 2);
    }

    #[test]
    fn tp13_corrente_corrompido_para_a_publicacao_mas_nao_o_exame() {
        let destino = arena("tp13_corrompido");
        let leitura = inicializar(&destino);
        // Scrittore non cooperante: sovrascrive il corrente con spazzatura.
        fs::write(destino.corrente(), "{ isto nao e' json").unwrap();

        // `observar` funziona: l'impronta dei byte non dipende dal codec.
        match observar(&destino).unwrap() {
            EstadoObservado::Corrente { token, .. } => {
                assert!(token.is_none(), "nessun token da un file illeggibile")
            }
            outro => panic!("estado inesperado: {outro:?}"),
        }
        match ler(&destino).unwrap_err() {
            ErroIo::ArquivoCorrenteInvalido { caminho, .. } => {
                assert_eq!(caminho, destino.corrente())
            }
            outro => panic!("erro inesperado: {outro}"),
        }
        // E non si pubblica sopra un archivio che non si sa leggere.
        let candidato = com_mais_um_evento(&leitura.arquivo, 18);
        match publicar(&destino, &candidato, Some(&leitura.base), &lucchetto("c")).unwrap_err() {
            ErroIo::ArquivoCorrenteInvalido { .. } => {}
            outro => panic!("erro inesperado: {outro}"),
        }
        assert_eq!(
            fs::read_to_string(destino.corrente()).unwrap(),
            "{ isto nao e' json",
            "il file corrotto non viene riparato ne' sovrascritto"
        );
        assert!(!destino.lucchetto().exists());
    }

    #[test]
    fn tp13_publicar_com_base_sobre_destino_ausente_e_recusado() {
        let destino = arena("tp13_sem_corrente");
        let base = TokenBase {
            revisao: 7,
            sha256_conteudo: "0".repeat(64),
        };
        match publicar(&destino, &arquivo_valido(), Some(&base), &lucchetto("x")).unwrap_err() {
            ErroIo::DestinoAusente { caminho } => assert_eq!(caminho, destino.corrente()),
            outro => panic!("erro inesperado: {outro}"),
        }
        assert!(!destino.corrente().exists(), "niente creato per errore");
        assert!(!destino.lucchetto().exists());
    }

    // =================================================================
    // TP-14: base, contatore e concorrenza
    // =================================================================

    #[test]
    fn tp14_duas_publicacoes_da_mesma_base_nao_passam_as_duas() {
        let destino = arena("tp14_base");
        let leitura = inicializar(&destino);
        let primeiro = com_mais_um_evento(&leitura.arquivo, 18);
        publicar(
            &destino,
            &primeiro,
            Some(&leitura.base),
            &lucchetto("primeiro"),
        )
        .unwrap();

        // Il secondo pubblicatore ha ancora la base vecchia.
        let segundo = com_mais_um_evento(&leitura.arquivo, 19);
        match publicar(
            &destino,
            &segundo,
            Some(&leitura.base),
            &lucchetto("segundo"),
        )
        .unwrap_err()
        {
            ErroIo::BaseDesatualizada {
                esperada,
                encontrada,
            } => {
                assert_eq!(esperada, leitura.base);
                assert_eq!(encontrada.revisao, 2);
            }
            outro => panic!("erro inesperado: {outro}"),
        }
        // La prima pubblicazione resta quella sul disco.
        assert_eq!(ler(&destino).unwrap().arquivo, primeiro);
    }

    #[test]
    fn tp14_mesmo_contador_de_outro_arquivo_nao_basta() {
        let destino = arena("tp14_outro");
        let leitura = inicializar(&destino);

        // Un altro archivio, con lo **stesso** contatore. Il token di base
        // porta anche l'impronta del contenuto, per questo non passa (§8.3).
        let plano = plano();
        let mut outro = Arquivo::novo();
        let mut documento = um_documento();
        documento.lote.id = "lote-de-outra-maquina".to_string();
        outro
            .acrescentar_lote(
                &[documento],
                &plano,
                IdentidadePlano::from_str("1.0").unwrap(),
            )
            .unwrap();
        fs::write(destino.corrente(), serializar(&outro, 1).unwrap()).unwrap();

        let candidato = com_mais_um_evento(&leitura.arquivo, 18);
        match publicar(&destino, &candidato, Some(&leitura.base), &lucchetto("o")).unwrap_err() {
            ErroIo::BaseDesatualizada {
                esperada,
                encontrada,
            } => {
                assert_eq!(esperada.revisao, encontrada.revisao, "contatore uguale");
                assert_ne!(
                    esperada.sha256_conteudo, encontrada.sha256_conteudo,
                    "contenuto diverso: e' questo che lo blocca"
                );
            }
            outro => panic!("erro inesperado: {outro}"),
        }
    }

    #[test]
    fn tp14_candidato_que_perde_historia_e_recusado_tambem_no_io() {
        let destino = arena("tp14_historia");
        let leitura = inicializar(&destino);
        let mut amputado = leitura.arquivo.clone();
        amputado.eventos.pop();
        let antes = bytes_do_corrente(&destino);
        match publicar(&destino, &amputado, Some(&leitura.base), &lucchetto("h")).unwrap_err() {
            ErroIo::Conteudo(ErroArquivo::HistoriaTruncada { o_que }) => {
                assert!(o_que.contains("declaracao"), "{o_que}")
            }
            outro => panic!("erro inesperado: {outro}"),
        }
        assert_eq!(bytes_do_corrente(&destino), antes);
        assert!(!destino.lucchetto().exists());
    }

    #[test]
    fn tp14_lucchetto_realmente_tomado_bloqueia_e_depois_libera() {
        let destino = arena("tp14_lucchetto");
        let leitura = inicializar(&destino);
        let candidato = com_mais_um_evento(&leitura.arquivo, 18);

        let metadados = lucchetto("manutencao em curso");
        let trava = Trava::tomar(&destino, &metadados).unwrap();
        assert!(destino.lucchetto().exists());

        // Chi arriva dopo non entra, e sa chi e' dentro.
        match publicar(&destino, &candidato, Some(&leitura.base), &lucchetto("b")).unwrap_err() {
            ErroIo::ArquivoOcupado {
                lucchetto: caminho,
                conteudo_declarado,
            } => {
                assert_eq!(caminho, destino.lucchetto());
                assert!(conteudo_declarado.contains("manutencao em curso"));
                assert!(conteudo_declarado.contains("Jos\u{00E9}"));
            }
            outro => panic!("erro inesperado: {outro}"),
        }

        // Nessuno sblocca al posto d'altri: la confermazione deve coincidere.
        match liberar(&destino, &lucchetto("outro motivo")).unwrap_err() {
            ErroIo::ConfirmacaoNaoCoincide { lucchetto: c, .. } => {
                assert_eq!(c, destino.lucchetto())
            }
            outro => panic!("erro inesperado: {outro}"),
        }
        assert!(destino.lucchetto().exists(), "non rimosso per errore");

        // Con la confermazione giusta si libera, e poi non c'e' piu' niente.
        liberar(&destino, &metadados).unwrap();
        assert!(!destino.lucchetto().exists());
        match liberar(&destino, &metadados).unwrap_err() {
            ErroIo::LucchettoAusente { caminho } => assert_eq!(caminho, destino.lucchetto()),
            outro => panic!("erro inesperado: {outro}"),
        }
        // La guardia non rinasce: il suo `Drop` non ricrea niente.
        drop(trava);
        assert!(!destino.lucchetto().exists());

        // Liberato il passaggio, la pubblicazione riesce.
        assert_eq!(
            publicar(&destino, &candidato, Some(&leitura.base), &lucchetto("ok"))
                .unwrap()
                .revisao_publicada,
            2
        );
    }

    // =================================================================
    // TP-15: residui e candidati
    // =================================================================

    #[test]
    fn tp15_candidatos_classifica_cada_papel_e_valida_sem_carregar() {
        let destino = arena("tp15_candidatos");
        let leitura = inicializar(&destino);
        let candidato = com_mais_um_evento(&leitura.arquivo, 18);
        publicar(&destino, &candidato, Some(&leitura.base), &lucchetto("s")).unwrap();

        // Residuo e prova fabbricati a mano, piu' un file estraneo.
        let residuo = destino.diretorio.join("arquivo.json.novo.9");
        fs::write(&residuo, serializar(&candidato, 9).unwrap()).unwrap();
        let prova = destino.diretorio.join("arquivo.json.corrupto7");
        fs::write(&prova, "{ nao e' json").unwrap();
        fs::write(destino.diretorio.join("leia-me.txt"), "anotacao").unwrap();

        let achados = candidatos(&destino).unwrap();
        assert!(matches!(
            achados.estado,
            EstadoObservado::Corrente { token: Some(_), .. }
        ));
        let por_papel = |papel: PapelCandidato| {
            achados
                .candidatos
                .iter()
                .filter(|c| c.papel == papel)
                .collect::<Vec<_>>()
        };
        assert_eq!(por_papel(PapelCandidato::Corrente).len(), 1);
        assert_eq!(por_papel(PapelCandidato::Geracao).len(), 1);
        assert_eq!(por_papel(PapelCandidato::TemporarioRestante).len(), 1);
        assert_eq!(por_papel(PapelCandidato::Prova).len(), 1);
        assert_eq!(
            achados.candidatos.len(),
            4,
            "il file estraneo non e' un candidato"
        );

        let corrente = por_papel(PapelCandidato::Corrente)[0];
        assert_eq!(corrente.revisao_arquivo, Some(2));
        assert!(corrente.validacao.is_ok());
        assert_eq!(corrente.snapshots, 2);
        assert_eq!(corrente.eventos, 3);

        let geracao = por_papel(PapelCandidato::Geracao)[0];
        assert_eq!(geracao.revisao_arquivo, Some(1));
        assert_eq!(geracao.eventos, 2, "la generazione e' lo stato precedente");

        // La prova illeggibile viene elencata con il suo errore, non nascosta.
        let prova_achada = por_papel(PapelCandidato::Prova)[0];
        assert!(prova_achada.validacao.is_err());
        assert_eq!(prova_achada.revisao_arquivo, None);
        assert_eq!(prova_achada.snapshots, 0, "zero tecnico, non un conteggio");

        // Niente e' stato cancellato (D16).
        assert!(residuo.exists() && prova.exists());
        assert!(destino.diretorio.join("arquivo.json.ger1").exists());
    }

    // =================================================================
    // TP-15b: recupero
    // =================================================================

    #[test]
    fn tp15b_recuperar_faz_as_tres_transicoes() {
        let destino = arena("tp15b_recuperar");
        let leitura = inicializar(&destino);
        let segunda = com_mais_um_evento(&leitura.arquivo, 18);
        let publicacao =
            publicar(&destino, &segunda, Some(&leitura.base), &lucchetto("s")).unwrap();
        let geracao = publicacao.geracao_anterior.unwrap();

        // Transizione 1: il corrente si corrompe.
        fs::write(destino.corrente(), "{ corrompido").unwrap();
        let estado = observar(&destino).unwrap();
        assert!(matches!(
            estado,
            EstadoObservado::Corrente { token: None, .. }
        ));

        // Transizione 2: recupero esplicito dalla generazione 1.
        let recuperacao = recuperar(&destino, &geracao, &estado, &lucchetto("recupero")).unwrap();
        assert_eq!(
            recuperacao.revisao_publicada, 1,
            "la revisione e' quella del candidato: nessun numero inventato"
        );
        let prova = recuperacao.geracao_anterior.unwrap();
        assert_eq!(
            fs::read_to_string(&prova).unwrap(),
            "{ corrompido",
            "il file corrotto e' conservato come prova"
        );
        assert!(geracao.exists(), "il candidato resta dov'era: e' copiato");

        // Transizione 3: il corrente e' di nuovo leggibile.
        let relida = ler(&destino).unwrap();
        assert_eq!(relida.base.revisao, 1);
        assert_eq!(relida.arquivo, leitura.arquivo);
        assert!(!destino.lucchetto().exists());
        assert!(
            !destino.diretorio.join("arquivo.json.recuperado.1").exists(),
            "il temporaneo del recupero non resta"
        );

        // Una seconda prova con la stessa etichetta non si sovrascrive.
        fs::write(destino.corrente(), "{ corrompido de novo").unwrap();
        let estado = observar(&destino).unwrap();
        fs::write(&prova, "prova anterior").unwrap();
        match recuperar(&destino, &geracao, &estado, &lucchetto("r2")).unwrap_err() {
            ErroIo::ProvaJaPresente { caminho } => assert_eq!(caminho, prova),
            outro => panic!("erro inesperado: {outro}"),
        }
        assert_eq!(fs::read_to_string(&prova).unwrap(), "prova anterior");
    }

    #[test]
    fn tp15b_recuperar_nao_toca_nada_se_o_estado_mudou() {
        let destino = arena("tp15b_estado");
        let leitura = inicializar(&destino);
        let segunda = com_mais_um_evento(&leitura.arquivo, 18);
        let publicacao =
            publicar(&destino, &segunda, Some(&leitura.base), &lucchetto("s")).unwrap();
        let geracao = publicacao.geracao_anterior.unwrap();

        // Stato osservato prima, poi qualcuno pubblica di nuovo.
        let estado_antigo = EstadoObservado::DestinoAusente;
        let antes = bytes_do_corrente(&destino);
        match recuperar(&destino, &geracao, &estado_antigo, &lucchetto("r")).unwrap_err() {
            ErroIo::EstadoMudou {
                esperado,
                encontrado,
            } => {
                assert_eq!(esperado, EstadoObservado::DestinoAusente);
                assert!(matches!(
                    encontrado,
                    EstadoObservado::Corrente { token: Some(_), .. }
                ));
            }
            outro => panic!("erro inesperado: {outro}"),
        }
        assert_eq!(bytes_do_corrente(&destino), antes, "niente toccato");
        assert_eq!(
            fs::read_dir(&destino.diretorio).unwrap().count(),
            2,
            "nessuna prova e nessun temporaneo creati"
        );
        assert!(!destino.lucchetto().exists());
    }

    #[test]
    fn tp15b_candidato_invalido_nao_e_promovido() {
        let destino = arena("tp15b_invalido");
        inicializar(&destino);
        let estado = observar(&destino).unwrap();
        let antes = bytes_do_corrente(&destino);

        let ruim = destino.diretorio.join("arquivo.json.ger9");
        fs::write(&ruim, "{ nem isto e' json").unwrap();
        match recuperar(&destino, &ruim, &estado, &lucchetto("r")).unwrap_err() {
            ErroIo::CandidatoInvalido { caminho, .. } => assert_eq!(caminho, ruim),
            outro => panic!("erro inesperado: {outro}"),
        }
        assert_eq!(bytes_do_corrente(&destino), antes);
        assert!(!destino.lucchetto().exists());

        // Candidato sintatticamente valido ma strutturalmente rotto.
        let mut adulterado = arquivo_valido();
        adulterado.snapshots[0].impressao_conteudo = "0".repeat(64);
        fs::write(&ruim, serializar(&adulterado, 3).unwrap()).unwrap();
        match recuperar(&destino, &ruim, &estado, &lucchetto("r")).unwrap_err() {
            ErroIo::CandidatoInvalido { causa, .. } => assert!(matches!(
                causa,
                ErroArquivo::ImpressaoDivergente {
                    o_que: "conteudo",
                    ..
                }
            )),
            outro => panic!("erro inesperado: {outro}"),
        }
        assert_eq!(bytes_do_corrente(&destino), antes);
    }

    #[test]
    fn tp15b_recuperar_sobre_destino_ausente_nao_fabrica_prova() {
        let destino = arena("tp15b_ausente");
        let fonte = destino.diretorio.join("arquivo.json.ger1");
        fs::write(&fonte, serializar(&arquivo_valido(), 1).unwrap()).unwrap();

        let recuperacao = recuperar(
            &destino,
            &fonte,
            &EstadoObservado::DestinoAusente,
            &lucchetto("r"),
        )
        .unwrap();
        assert_eq!(recuperacao.revisao_publicada, 1);
        assert!(
            recuperacao.geracao_anterior.is_none(),
            "niente da conservare: nessuna prova inventata"
        );
        assert_eq!(ler(&destino).unwrap().arquivo, arquivo_valido());
    }

    #[test]
    fn tp15b_rollback_intencional_e_declarado_nao_renumerado() {
        let destino = arena("tp15b_rollback");
        let leitura = inicializar(&destino);
        let segunda = com_mais_um_evento(&leitura.arquivo, 18);
        let publicacao =
            publicar(&destino, &segunda, Some(&leitura.base), &lucchetto("s")).unwrap();
        let geracao = publicacao.geracao_anterior.unwrap();
        let estado = observar(&destino).unwrap();

        // Tornare indietro e' una decisione, non un errore: il recupero la
        // esegue e **non** inventa un numero nuovo. Il contatore torna a 1, e
        // questo e' un dato dichiarato, non un effetto collaterale nascosto.
        let recuperacao = recuperar(&destino, &geracao, &estado, &lucchetto("rollback")).unwrap();
        assert_eq!(recuperacao.revisao_publicada, 1);
        let relida = ler(&destino).unwrap();
        assert_eq!(relida.base.revisao, 1);
        assert_eq!(relida.arquivo.eventos.len(), 2, "lo stato anteriore");
        // La revisione 2 non e' persa: e' conservata come prova.
        let prova = recuperacao.geracao_anterior.unwrap();
        let (conservado, revisao) =
            arquivo::desserializar_bytes(&ler_bytes(&prova).unwrap()).unwrap();
        assert_eq!(revisao, 2);
        assert_eq!(conservado, segunda);

        // Conseguenza **dichiarata** del rollback: ripubblicare dalla revisione
        // 1 vorrebbe conservare di nuovo la generazione 1, che esiste gia'.
        // D16 vieta di sovrascriverla e il codice non la sovrascrive: la
        // ripresa dopo un rollback esige un intervento esplicito
        // dell'operatore sulla generazione conservata. Non e' un difetto
        // nascosto, e' il prezzo di D16 piu' D17(a).
        let terceira = com_mais_um_evento(&relida.arquivo, 20);
        let ger1 = destino.diretorio.join("arquivo.json.ger1");
        match publicar(&destino, &terceira, Some(&relida.base), &lucchetto("t")).unwrap_err() {
            ErroIo::GeracaoJaPresente { caminho } => assert_eq!(caminho, ger1),
            outro => panic!("erro inesperado: {outro}"),
        }
        assert!(ger1.exists(), "la generazione conservata resta intatta");
        assert_eq!(ler(&destino).unwrap().base.revisao, 1, "corrente intatto");

        // Spostati la generazione conservata **e** il temporaneo del tentativo
        // abbandonato, la ripresa riesce e il numero 2 viene riusato: questo
        // e' il dato dichiarato, non un dettaglio.
        fs::rename(&ger1, destino.diretorio.join("exame.ger1")).unwrap();
        fs::rename(
            destino.diretorio.join("arquivo.json.novo.2"),
            destino.diretorio.join("exame.novo.2"),
        )
        .unwrap();
        assert_eq!(
            publicar(&destino, &terceira, Some(&relida.base), &lucchetto("t2"))
                .unwrap()
                .revisao_publicada,
            2
        );
    }

    // =================================================================
    // Determinismo fra due pubblicazioni dello stesso contenuto
    // =================================================================

    #[test]
    fn dois_destinos_com_o_mesmo_conteudo_dao_os_mesmos_bytes() {
        let um = arena("det_um");
        let dois = arena("det_dois");
        inicializar(&um);
        inicializar(&dois);
        assert_eq!(
            bytes_do_corrente(&um),
            bytes_do_corrente(&dois),
            "stesso contenuto, stessa revisione, stessi byte"
        );
    }

    // =================================================================
    // Regressioni dell'audit del 2026-10-22 (R3C3B1-02)
    // =================================================================

    /// Byte che **non** sono UTF-8: un corrente corrotto a livello di
    /// codifica, non di sintassi JSON.
    const NAO_UTF8: [u8; 3] = [0xFF, 0xFE, 0x80];

    /// R3C3B1-02: l'impronta dei byte deve essere calcolabile anche quando il
    /// file non si puo' interpretare. Era subordinata a `read_to_string`,
    /// quindi su byte non UTF-8 l'osservazione falliva e il protocollo di
    /// recupero non partiva nemmeno.
    #[test]
    fn r2_observar_corrente_nao_utf8_da_impressao_dos_bytes() {
        let destino = arena("r2_observar");
        inicializar(&destino);
        fs::write(destino.corrente(), NAO_UTF8).unwrap();

        match observar(&destino).unwrap() {
            EstadoObservado::Corrente {
                sha256_bytes,
                token,
            } => {
                assert!(token.is_none(), "nessun token da byte non interpretabili");
                assert_eq!(sha256_bytes.len(), 64);
                assert!(sha256_bytes.chars().all(|c| c.is_ascii_hexdigit()));
                // L'impronta e' dei byte: cambiarli la cambia.
                fs::write(destino.corrente(), [0xFF, 0xFE, 0x81]).unwrap();
                let outra = observar(&destino).unwrap();
                assert_ne!(
                    outra,
                    EstadoObservado::Corrente {
                        sha256_bytes,
                        token: None
                    }
                );
            }
            outro => panic!("estado inesperado: {outro:?}"),
        }
    }

    #[test]
    fn r2_ler_e_publicar_nomeiam_o_corrente_nao_utf8() {
        let destino = arena("r2_ler");
        let leitura = inicializar(&destino);
        fs::write(destino.corrente(), NAO_UTF8).unwrap();

        match ler(&destino).unwrap_err() {
            ErroIo::ArquivoCorrenteInvalido { caminho, causa } => {
                assert_eq!(caminho, destino.corrente());
                assert!(
                    matches!(causa, ErroArquivo::NaoUtf8 { .. }),
                    "causa inesperada: {causa}"
                );
            }
            outro => panic!("erro inesperado: {outro}"),
        }

        let candidato = com_mais_um_evento(&leitura.arquivo, 18);
        match publicar(&destino, &candidato, Some(&leitura.base), &lucchetto("u")).unwrap_err() {
            ErroIo::ArquivoCorrenteInvalido { causa, .. } => {
                assert!(matches!(causa, ErroArquivo::NaoUtf8 { .. }))
            }
            outro => panic!("erro inesperado: {outro}"),
        }
        assert_eq!(
            fs::read(destino.corrente()).unwrap(),
            NAO_UTF8,
            "i byte corrotti non vengono riparati ne' sovrascritti"
        );
        assert!(!destino.lucchetto().exists());
    }

    #[test]
    fn r2_candidatos_elenca_os_outros_mesmo_com_residuo_nao_utf8() {
        let destino = arena("r2_candidatos");
        let leitura = inicializar(&destino);
        let segunda = com_mais_um_evento(&leitura.arquivo, 18);
        publicar(&destino, &segunda, Some(&leitura.base), &lucchetto("s")).unwrap();

        // Corrente corrotto nella codifica, generazione valida accanto, piu'
        // un residuo altrettanto illeggibile.
        fs::write(destino.corrente(), NAO_UTF8).unwrap();
        let residuo = destino.diretorio.join("arquivo.json.novo.9");
        fs::write(&residuo, NAO_UTF8).unwrap();

        let achados = candidatos(&destino).expect("l'elenco non si blocca sui byte illeggibili");
        assert!(matches!(
            achados.estado,
            EstadoObservado::Corrente { token: None, .. }
        ));
        assert_eq!(achados.candidatos.len(), 3);

        let validos: Vec<_> = achados
            .candidatos
            .iter()
            .filter(|c| c.validacao.is_ok())
            .collect();
        assert_eq!(validos.len(), 1, "la generazione valida resta elencabile");
        assert_eq!(validos[0].papel, PapelCandidato::Geracao);
        assert_eq!(validos[0].revisao_arquivo, Some(1));

        for ilegivel in achados.candidatos.iter().filter(|c| c.validacao.is_err()) {
            assert_eq!(ilegivel.revisao_arquivo, None);
            assert_eq!(ilegivel.snapshots, 0, "zero tecnico, non un conteggio");
        }
    }

    #[test]
    fn r2_recuperar_de_corrente_nao_utf8_conserva_a_prova_byte_a_byte() {
        let destino = arena("r2_recuperar");
        let leitura = inicializar(&destino);
        let segunda = com_mais_um_evento(&leitura.arquivo, 18);
        let publicacao =
            publicar(&destino, &segunda, Some(&leitura.base), &lucchetto("s")).unwrap();
        let geracao = publicacao.geracao_anterior.unwrap();

        fs::write(destino.corrente(), NAO_UTF8).unwrap();
        let estado = observar(&destino).unwrap();

        let recuperacao = recuperar(&destino, &geracao, &estado, &lucchetto("recupero")).unwrap();
        assert_eq!(recuperacao.revisao_publicada, 1);
        let prova = recuperacao.geracao_anterior.unwrap();
        assert_eq!(
            fs::read(&prova).unwrap(),
            NAO_UTF8,
            "la prova conserva i byte, non una loro versione leggibile"
        );
        assert_eq!(ler(&destino).unwrap().arquivo, leitura.arquivo);
        assert!(geracao.exists(), "il candidato resta dov'era");
        assert!(!destino.lucchetto().exists());
    }

    #[test]
    fn r2_estado_mudou_se_os_bytes_corrompidos_mudam() {
        let destino = arena("r2_estado");
        let leitura = inicializar(&destino);
        let segunda = com_mais_um_evento(&leitura.arquivo, 18);
        let publicacao =
            publicar(&destino, &segunda, Some(&leitura.base), &lucchetto("s")).unwrap();
        let geracao = publicacao.geracao_anterior.unwrap();

        fs::write(destino.corrente(), NAO_UTF8).unwrap();
        let estado = observar(&destino).unwrap();
        // Qualcuno corrompe il corrente in un **altro** modo dopo
        // l'osservazione: il recupero non deve procedere alla cieca.
        fs::write(destino.corrente(), [0x00, 0xC0, 0xAF]).unwrap();

        match recuperar(&destino, &geracao, &estado, &lucchetto("r")).unwrap_err() {
            ErroIo::EstadoMudou { .. } => {}
            outro => panic!("erro inesperado: {outro}"),
        }
        assert_eq!(
            fs::read(destino.corrente()).unwrap(),
            [0x00, 0xC0, 0xAF],
            "niente toccato"
        );
        assert_eq!(
            fs::read_dir(&destino.diretorio).unwrap().count(),
            2,
            "nessuna prova e nessun temporaneo creati"
        );
        assert!(!destino.lucchetto().exists());
    }
}
