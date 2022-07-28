use crate::tournament::pairing_system_factory;
use crate::tournament::scoring_system_factory;
use crate::tournament::{Tournament, TournamentId, TournamentPreset, TournamentStatus};
use crate::tournament::PairingSystem::{Swiss, Fluid};
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
use std::ffi::CString;
use std::ptr::null;
use std::vec::Vec;
use std::os::raw::c_char;
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

#[no_mangle]
pub extern "C" fn init_squire_ffi() {
    let map: DashMap<TournamentId, Tournament> = DashMap::new();
    FFI_TOURNAMENT_REGISTRY.set(map);
}

/// Helper function for cloning strings
fn clone_string_to_c_string(s: String) -> *const c_char {
    let len: usize = s.len() + 1;
    let mut v: Vec<u8> = Vec::with_capacity(len);
    let s_str = s.as_bytes();
    v[s.len()] = 0;

    let mut i: usize = 0;
    while i < s.len() {
        v[i] = s_str[i];
        i += 1;
    }
    return v.as_ptr() as *const c_char;
}

/// TournamentIds can be used to get data safely from
/// the Rust lib with these methods
impl TournamentId {
    /// Returns the name of a tournament
    /// Returns NULL if an error happens
    /// This is heap allocated, please free it
    #[no_mangle]
    pub extern "C" fn name(self: Self) -> *const c_char {
        let tourn: Tournament;
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => { tourn = t.value().clone() },
            None => { return std::ptr::null(); }
        }
        return clone_string_to_c_string(tourn.name);
    }
    
    /// Returns the format of a tournament
    /// Returns NULL if an error happens
    /// This is heap allocated, please free it
    #[no_mangle]
    pub extern "C" fn format(self: Self) -> *const c_char {
        let tourn: Tournament;
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => { tourn = t.value().clone() },
            None => { return std::ptr::null(); }
        }
        return clone_string_to_c_string(tourn.format)
    }

    /// Returns whether table numbers are being used for this tournament
    /// false, is the error value (kinda sketchy)
    #[no_mangle]
    pub extern "C" fn use_table_number(self: Self) -> bool {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => { return t.value().use_table_number },
            None => { 
                println!("Cannot find tournament in tourn_id.use_table_number();");
                return false;
            }
        }
    }
    
    /// Returns the game size
    /// -1 is the error value
    #[no_mangle]
    pub extern "C" fn game_size(self: Self) -> i32 {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => { return t.value().game_size as i32 },
            None => { 
                println!("Cannot find tournament in tourn_id.game_size();");
                return -1;
            }
        }
    }
    
    /// Returns the min deck count
    /// -1 is the error value
    #[no_mangle]
    pub extern "C" fn min_deck_count(self: Self) -> i32 {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => { return t.value().min_deck_count as i32 },
            None => { 
                println!("Cannot find tournament in tourn_id.min_deck_count();");
                return -1;
            }
        }
    }

    /// Returns the max deck count
    /// -1 is the error value
    #[no_mangle]
    pub extern "C" fn max_deck_count(self: Self) -> i32 {
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => { return t.value().max_deck_count as i32 },
            None => { 
                println!("Cannot find tournament in tourn_id.max_deck_count();");
                return -1;
            }
        }
    }

    /// Returns the pairing type
    /// This is of type TournamentPreset, but an int to let me return error values
    /// -1 is error value
    #[no_mangle]
    pub extern "C" fn pairing_type(self: Self) -> i32 {
        let tourn: Tournament;
        match FFI_TOURNAMENT_REGISTRY.get().unwrap().get(&self) {
            Some(t) => { tourn = t.value().clone() },
            None => { 
                println!("Cannot find tournament in tourn_id.pairing_type();");
                return -1;
            }
        }
        
        match tourn.pairing_sys {
            Swiss(_) => { return TournamentPreset::Swiss as i32; }
            Fluid(_) => { return TournamentPreset::Fluid as i32; }
        }
    }

    // End of getters

    /// Closes a tournament removing it from the internal FFI state
    #[no_mangle]
    pub extern "C" fn close_tourn(self: Self) {
        FFI_TOURNAMENT_REGISTRY.get().unwrap().remove(&self);
    }

    /// Saves a tournament to a name
    /// Returns true if successful, false if not.
    #[no_mangle]
    pub extern "C" fn save_tourn(self: Self, __file: *const c_char) -> bool {
        let file: &str = unsafe { CStr::from_ptr(__file).to_str().unwrap() };
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
/// Returns a NULL UUID (all 0s) if there is an error
#[no_mangle]
pub extern "C" fn load_tournament_from_file(__file: * const c_char) -> TournamentId {
    let file: &str = unsafe { CStr::from_ptr(__file).to_str().unwrap() };
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
/// Returns a NULL UUID (all 0s) if there is an error
#[no_mangle]
pub extern "C" fn new_tournament_from_settings(
    __file: *const c_char,
    __name: *const c_char,
    __format: *const c_char,
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
        name: String::from(unsafe { CStr::from_ptr(__name).to_str().unwrap().to_string() }),
        use_table_number: use_table_number,
        format: String::from(unsafe { CStr::from_ptr(__format).to_str().unwrap().to_string() }),
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

    if !tournament.id.save_tourn(__file) {
        return TournamentId(Uuid::from_bytes(*NULL_UUID_BYTES));
    }
    return tournament.id;
}
