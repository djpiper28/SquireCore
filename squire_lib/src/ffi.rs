use crate::tournament::pairing_system_factory;
use crate::tournament::scoring_system_factory;
use crate::tournament::{Tournament, TournamentId, TournamentPreset, TournamentStatus};
use crate::{
    error::TournamentError,
    fluid_pairings::FluidPairings,
    operations::{OpData, OpResult, TournOp},
    pairings::Pairings,
    player::{Player, PlayerId, PlayerStatus},
    player_registry::{PlayerIdentifier, PlayerRegistry},
    round::{Round, RoundId, RoundResult, RoundStatus},
    round_registry::{RoundIdentifier, RoundRegistry},
    scoring::{Score, Standings},
    settings::{
        self, FluidPairingsSetting, PairingSetting, ScoringSetting, StandardScoringSetting,
        SwissPairingsSetting, TournamentSetting,
    },
    standard_scoring::{StandardScore, StandardScoring},
    swiss_pairings::SwissPairings,
};
use dashmap::DashMap;
use once_cell::sync::OnceCell;
use serde_json;
use std::ffi::CStr;
use std::fs::{read_to_string, remove_file, rename, write};
use std::option::Option;
use std::time::Duration;
use uuid::Uuid;

lazy_static! {
    /// NULL UUIDs are returned on errors
static ref NULL_UUID_BYTES:
    [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
}

/// A map of tournament ids to tournaments
/// this is used for allocating ffi tournaments
/// all ffi tournaments are always deeply copied
/// at the lanuage barrier
static FFI_TOURNAMENT_REGISTRY: OnceCell<DashMap<TournamentId, Tournament>> = OnceCell::new();
const BACKUP_EXT: &str = ".bak";

/// TournamentIds can be used to get data safely from
/// the Rust lib with these methods
impl TournamentId {
    #[no_mangle]
    fn close(self: Self) {
        FFI_TOURNAMENT_REGISTRY.get().unwrap().remove(&self);
    }

    /// Saves a tournament to a name
    /// Returns true if successful, false if not.
    #[no_mangle]
    fn save(self: Self, __file: &CStr) -> bool {
        let file: &str = __file.to_str().unwrap();
        let tournament: Tournament;
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(v) => tournament = v.value().clone(),
            None => {
                return false;
            }
        }

        let json: String;
        match serde_json::to_string::<Tournament>(&tournament) {
            Ok(v) => json = v,
            Err(_) => return false,
        }

        // Backup old data, do check for errors.
        let file_backup: String = file.to_string() + &BACKUP_EXT.to_string();
        std::fs::remove_file(file_backup.clone());
        std::fs::rename(file, file_backup.clone());

        match std::fs::write(file, json) {
            Ok(_) => {
                return true;
            }
            Err(e) => {
                println!("ffi-error: {}", e);
                return false;
            }
        }
    }
}

/// Loads a tournament from a file via serde
/// The tournament is then registered (stored on the heap)
/// CStr path to the tournament (alloc and, free on Cxx side)
#[no_mangle]
fn load_tournament_from_file(__file: &CStr) -> TournamentId {
    let file: &str = __file.to_str().unwrap();
    let json: String;
    match read_to_string(file) {
        Ok(v) => json = v.to_string(),
        Err(_) => {
            return TournamentId(Uuid::from_bytes(*NULL_UUID_BYTES));
        }
    };

    let tournament: Tournament;
    match serde_json::from_str::<Tournament>(&json) {
        Ok(v) => tournament = v,
        Err(_) => {
            return TournamentId(Uuid::from_bytes(*NULL_UUID_BYTES));
        }
    };

    // Cannot open the same tournament twice
    if FFI_TOURNAMENT_REGISTRY
        .get()
        .unwrap()
        .contains_key(&tournament.id)
    {
        return TournamentId(Uuid::from_bytes(*NULL_UUID_BYTES));
    }

    let tid: TournamentId = tournament.id.clone();
    FFI_TOURNAMENT_REGISTRY
        .get()
        .unwrap()
        .insert(tid, tournament.clone());

    return tournament.id;
}

/// Creates a tournament from the settings provided
#[no_mangle]
fn new_tournament_from_settings(
    __file: &CStr,
    __name: &CStr,
    __format: &CStr,
    preset: TournamentPreset,
    use_table_number: bool,
    game_size: u8,
    min_deck_count: u8,
    max_deck_count: u8,
    reg_open: bool,
    require_check_in: bool,
    require_deck_reg: bool,
) -> TournamentId {
    let tournament: Tournament = Tournament {
        id: TournamentId(Uuid::new_v4()),
        name: __name.to_str().unwrap().to_string(),
        use_table_number: use_table_number,
        format: __format.to_str().unwrap().to_string(),
        game_size: game_size,
        min_deck_count: min_deck_count,
        max_deck_count: max_deck_count,
        player_reg: PlayerRegistry::new(),
        round_reg: RoundRegistry::new(0, Duration::from_secs(3000)),
        pairing_sys: pairing_system_factory(&preset, 2),
        scoring_sys: scoring_system_factory(&preset),
        reg_open: reg_open,
        require_check_in: require_check_in,
        require_deck_reg: require_deck_reg,
        status: TournamentStatus::Planned,
    };
    let tid: TournamentId = tournament.id;

    FFI_TOURNAMENT_REGISTRY
        .get()
        .unwrap()
        .insert(tid, tournament.clone());

    tournament.id.save(__file);
    return tournament.id;
}
