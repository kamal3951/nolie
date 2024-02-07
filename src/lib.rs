#![doc = include_str!("../README.md")]
#![allow(unused_variables)]

#[macro_use]
extern crate pbc_contract_codegen;
extern crate pbc_contract_common;

use pbc_contract_common::address::Address;
use pbc_contract_common::avl_tree_map::AvlTreeMap;
use pbc_contract_common::context::ContractContext;

#[state]
struct ContractState {
    pub size_of_matching: u8,

    pub number_of_men: u8,
    pub number_of_women: u8,

    pub admin: Address,

    pub men: AvlTreeMap<Address, u8>,
    pub women: AvlTreeMap<Address, u8>,

    women_preferences: Vec<Vec<u32>>, //make them private
    men_preferences: Vec<Vec<u32>>,
}

#[init]
fn initialize(_ctx: ContractContext, _size_of_matching: u8) -> ContractState {
    ContractState {
        size_of_matching: _size_of_matching,
        number_of_men: 0,
        number_of_women: 0,
        admin: _ctx.sender,
        men: AvlTreeMap::new(), //whosoever pushes first here, becomes the first woman, update this value accordingly
        women: AvlTreeMap::new(), //same here
        women_preferences: Vec::new(),
        men_preferences: Vec::new(),
    }
}

#[action(shortname = 0x01)]
fn join_matching_man(_ctx: ContractContext, mut state: ContractState) -> ContractState {
    state.men.insert(_ctx.sender, 0);
    state
}

#[action(shortname = 0x02)]
fn join_matching_woman(_ctx: ContractContext, mut state: ContractState) -> ContractState {
    state.women.insert(_ctx.sender, 0);
    state
}

#[action(shortname = 0x03)]
fn add_preferences(
    _ctx: ContractContext,
    mut state: ContractState,
    prefs: Vec<u32>,
) -> ContractState {
    let joined_man: bool = state.men.contains_key(&_ctx.sender);
    let joined_woman: bool = state.women.contains_key(&_ctx.sender);
    assert!(
        joined_man || joined_woman,
        "You've not joined any matching, join a matching first to add your preferences!"
    );
    assert_eq!(
        prefs.len(),
        state.size_of_matching as usize,
        "Number of preferences must be equal to the size of the matching"
    );
    if joined_man {
        state.men_preferences.push(prefs.clone()); //cloning to avoid prefs moving out of scope for line76 to use
        state.men.insert(_ctx.sender, state.number_of_men + 1);
    }
    if joined_woman {
        state.women_preferences.push(prefs);
        state.women.insert(_ctx.sender, state.number_of_women + 1);
    }

    state
}

#[action(shortname = 0x04)]
fn solve_matching(_ctx: ContractContext, state: ContractState) -> Vec<u32> {
    assert!(state.admin == _ctx.sender, "You're not allowed to solve!");
    assert!(
        state.number_of_men == state.number_of_women,
        "The matching has unequal number of sexes, Cannot match!"
    );
    let size: usize = state.men_preferences[0].len();

    let mut woman_partner: Vec<u32> = vec![0; size];
    let mut man_engaged: Vec<bool> = vec![false; size];

    let mut free_men_count: usize = size;

    let upper_bound: u32 = size as u32;

    while free_men_count > 0 {
        for man in 1..=upper_bound {
            for &woman in &state.men_preferences[(man - 1) as usize] {
                if !man_engaged[(man - 1) as usize] {
                    if woman_partner[(woman - 1) as usize] == 0 {
                        woman_partner[(woman - 1) as usize] = man;
                        man_engaged[(man - 1) as usize] = true;
                        free_men_count = free_men_count - 1;
                        break;
                    } else {
                        for &man_pref in &state.women_preferences[(man - 1) as usize] {
                            let mut need_to_break: bool = false;
                            if {
                                if man == man_pref {
                                    true
                                } else if man == woman_partner[(man - 1) as usize] {
                                    need_to_break = true;
                                    true
                                } else {
                                    false
                                }
                            } {
                                if need_to_break {
                                    break;
                                }
                                man_engaged[(woman_partner[(woman - 1) as usize] - 1) as usize] =
                                    false;
                                woman_partner[(woman - 1) as usize] = man;
                                man_engaged[(woman - 1) as usize] = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    woman_partner
}
