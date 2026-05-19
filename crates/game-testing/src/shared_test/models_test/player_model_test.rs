use game_shared::models::player::{PartySlot, PlayerPartyInputConfig};

#[test]
fn player_party_input_config_has_expected_defaults() {
    let input = PlayerPartyInputConfig::default();
    assert_eq!(input.party_slot_01, "1");
    assert_eq!(input.party_slot_02, "2");
    assert_eq!(input.party_slot_03, "3");
    assert_eq!(input.party_slot_04, "4");
    assert_eq!(input.party_next_slot, "Q");
}

#[test]
fn party_slot_component_stores_index() {
    let slot = PartySlot { index: 3 };
    assert_eq!(slot.index, 3);
}
