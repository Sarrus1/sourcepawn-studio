import { CompletionItem, CompletionItemKind } from "vscode";

const detail: string = "Left 4 Dead 2 Event";
const kind: CompletionItemKind = CompletionItemKind.Keyword;

export const l4d2Events: CompletionItem[] = [
	{
		label: "player_death",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_hurt",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_team",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_bot_replace",
		kind: kind,
		detail: detail,
	},
	{
		label: "bot_player_replace",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_afk",
		kind: kind,
		detail: detail,
	},
	{
		label: "weapon_fire",
		kind: kind,
		detail: detail,
	},
	{
		label: "weapon_fire_on_empty",
		kind: kind,
		detail: detail,
	},
	{
		label: "weapon_reload",
		kind: kind,
		detail: detail,
	},
	{
		label: "weapon_zoom",
		kind: kind,
		detail: detail,
	},
	{
		label: "ability_use",
		kind: kind,
		detail: detail,
	},
	{
		label: "ammo_pickup",
		kind: kind,
		detail: detail,
	},
	{
		label: "item_pickup",
		kind: kind,
		detail: detail,
	},
	{
		label: "grenade_bounce",
		kind: kind,
		detail: detail,
	},
	{
		label: "hegrenade_detonate",
		kind: kind,
		detail: detail,
	},
	{
		label: "bullet_impact",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_footstep",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_jump",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_blind",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_falldamage",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_ledge_grab",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_ledge_release",
		kind: kind,
		detail: detail,
	},
	{
		label: "door_moving",
		kind: kind,
		detail: detail,
	},
	{
		label: "door_open",
		kind: kind,
		detail: detail,
	},
	{
		label: "door_close",
		kind: kind,
		detail: detail,
	},
	{
		label: "door_unlocked",
		kind: kind,
		detail: detail,
	},
	{
		label: "rescue_door_open",
		kind: kind,
		detail: detail,
	},
	{
		label: "waiting_checkpoint_door_used",
		kind: kind,
		detail: detail,
	},
	{
		label: "waiting_door_used_versus",
		kind: kind,
		detail: detail,
	},
	{
		label: "waiting_checkpoint_button_used",
		kind: kind,
		detail: detail,
	},
	{
		label: "success_checkpoint_button_used",
		kind: kind,
		detail: detail,
	},
	{
		label: "round_freeze_end",
		kind: kind,
		detail: detail,
	},
	{
		label: "round_start_pre_entity",
		kind: kind,
		detail: detail,
	},
	{
		label: "round_start_post_nav",
		kind: kind,
		detail: detail,
	},
	{
		label: "nav_blocked",
		kind: kind,
		detail: detail,
	},
	{
		label: "nav_generate",
		kind: kind,
		detail: detail,
	},
	{
		label: "round_end_message",
		kind: kind,
		detail: detail,
	},
	{
		label: "round_end",
		kind: kind,
		detail: detail,
	},
	{
		label: "vote_ended",
		kind: kind,
		detail: detail,
	},
	{
		label: "vote_started",
		kind: kind,
		detail: detail,
	},
	{
		label: "vote_changed",
		kind: kind,
		detail: detail,
	},
	{
		label: "vote_passed",
		kind: kind,
		detail: detail,
	},
	{
		label: "vote_failed",
		kind: kind,
		detail: detail,
	},
	{
		label: "vote_cast_yes",
		kind: kind,
		detail: detail,
	},
	{
		label: "vote_cast_no",
		kind: kind,
		detail: detail,
	},
	{
		label: "infected_hurt",
		kind: kind,
		detail: detail,
	},
	{
		label: "infected_death",
		kind: kind,
		detail: detail,
	},
	{
		label: "hostname_changed",
		kind: kind,
		detail: detail,
	},
	{
		label: "difficulty_changed",
		kind: kind,
		detail: detail,
	},
	{
		label: "finale_start",
		kind: kind,
		detail: detail,
	},
	{
		label: "finale_rush",
		kind: kind,
		detail: detail,
	},
	{
		label: "finale_escape_start",
		kind: kind,
		detail: detail,
	},
	{
		label: "finale_vehicle_incoming",
		kind: kind,
		detail: detail,
	},
	{
		label: "finale_vehicle_ready",
		kind: kind,
		detail: detail,
	},
	{
		label: "finale_vehicle_leaving",
		kind: kind,
		detail: detail,
	},
	{
		label: "finale_win",
		kind: kind,
		detail: detail,
	},
	{
		label: "mission_lost",
		kind: kind,
		detail: detail,
	},
	{
		label: "finale_radio_start",
		kind: kind,
		detail: detail,
	},
	{
		label: "finale_radio_damaged",
		kind: kind,
		detail: detail,
	},
	{
		label: "final_reportscreen",
		kind: kind,
		detail: detail,
	},
	{
		label: "map_transition",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_transitioned",
		kind: kind,
		detail: detail,
	},
	{
		label: "heal_begin",
		kind: kind,
		detail: detail,
	},
	{
		label: "heal_success",
		kind: kind,
		detail: detail,
	},
	{
		label: "heal_end",
		kind: kind,
		detail: detail,
	},
	{
		label: "heal_interrupted",
		kind: kind,
		detail: detail,
	},
	{
		label: "ammo_pack_used",
		kind: kind,
		detail: detail,
	},
	{
		label: "give_weapon",
		kind: kind,
		detail: detail,
	},
	{
		label: "pills_used",
		kind: kind,
		detail: detail,
	},
	{
		label: "pills_used_fail",
		kind: kind,
		detail: detail,
	},
	{
		label: "ammo_pack_used_fail_no_weapon",
		kind: kind,
		detail: detail,
	},
	{
		label: "ammo_pack_used_fail_full",
		kind: kind,
		detail: detail,
	},
	{
		label: "ammo_pack_used_fail_doesnt_use_ammo",
		kind: kind,
		detail: detail,
	},
	{
		label: "ammo_pile_weapon_cant_use_ammo",
		kind: kind,
		detail: detail,
	},
	{
		label: "defibrillator_begin",
		kind: kind,
		detail: detail,
	},
	{
		label: "defibrillator_used",
		kind: kind,
		detail: detail,
	},
	{
		label: "defibrillator_used_fail",
		kind: kind,
		detail: detail,
	},
	{
		label: "defibrillator_interrupted",
		kind: kind,
		detail: detail,
	},
	{
		label: "upgrade_pack_begin",
		kind: kind,
		detail: detail,
	},
	{
		label: "upgrade_pack_used",
		kind: kind,
		detail: detail,
	},
	{
		label: "upgrade_item_already_used",
		kind: kind,
		detail: detail,
	},
	{
		label: "upgrade_failed_no_primary",
		kind: kind,
		detail: detail,
	},
	{
		label: "dead_survivor_visible",
		kind: kind,
		detail: detail,
	},
	{
		label: "adrenaline_used",
		kind: kind,
		detail: detail,
	},
	{
		label: "revive_begin",
		kind: kind,
		detail: detail,
	},
	{
		label: "revive_success",
		kind: kind,
		detail: detail,
	},
	{
		label: "revive_end",
		kind: kind,
		detail: detail,
	},
	{
		label: "drag_begin",
		kind: kind,
		detail: detail,
	},
	{
		label: "drag_end",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_incapacitated",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_incapacitated_start",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_entered_start_area",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_first_spawn",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_left_start_area",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_entered_checkpoint",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_left_checkpoint",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_shoved",
		kind: kind,
		detail: detail,
	},
	{
		label: "entity_shoved",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_jump_apex",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_blocked",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_now_it",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_no_longer_it",
		kind: kind,
		detail: detail,
	},
	{
		label: "witch_harasser_set",
		kind: kind,
		detail: detail,
	},
	{
		label: "witch_spawn",
		kind: kind,
		detail: detail,
	},
	{
		label: "witch_killed",
		kind: kind,
		detail: detail,
	},
	{
		label: "tank_spawn",
		kind: kind,
		detail: detail,
	},
	{
		label: "melee_kill",
		kind: kind,
		detail: detail,
	},
	{
		label: "area_cleared",
		kind: kind,
		detail: detail,
	},
	{
		label: "award_earned",
		kind: kind,
		detail: detail,
	},
	{
		label: "tongue_grab",
		kind: kind,
		detail: detail,
	},
	{
		label: "tongue_broke_bent",
		kind: kind,
		detail: detail,
	},
	{
		label: "tongue_broke_victim_died",
		kind: kind,
		detail: detail,
	},
	{
		label: "tongue_release",
		kind: kind,
		detail: detail,
	},
	{
		label: "choke_start",
		kind: kind,
		detail: detail,
	},
	{
		label: "choke_end",
		kind: kind,
		detail: detail,
	},
	{
		label: "choke_stopped",
		kind: kind,
		detail: detail,
	},
	{
		label: "tongue_pull_stopped",
		kind: kind,
		detail: detail,
	},
	{
		label: "lunge_shove",
		kind: kind,
		detail: detail,
	},
	{
		label: "lunge_pounce",
		kind: kind,
		detail: detail,
	},
	{
		label: "pounce_end",
		kind: kind,
		detail: detail,
	},
	{
		label: "pounce_stopped",
		kind: kind,
		detail: detail,
	},
	{
		label: "fatal_vomit",
		kind: kind,
		detail: detail,
	},
	{
		label: "survivor_call_for_help",
		kind: kind,
		detail: detail,
	},
	{
		label: "survivor_rescued",
		kind: kind,
		detail: detail,
	},
	{
		label: "survivor_rescue_abandoned",
		kind: kind,
		detail: detail,
	},
	{
		label: "relocated",
		kind: kind,
		detail: detail,
	},
	{
		label: "respawning",
		kind: kind,
		detail: detail,
	},
	{
		label: "tank_frustrated",
		kind: kind,
		detail: detail,
	},
	{
		label: "weapon_given",
		kind: kind,
		detail: detail,
	},
	{
		label: "weapon_drop",
		kind: kind,
		detail: detail,
	},
	{
		label: "break_breakable",
		kind: kind,
		detail: detail,
	},
	{
		label: "achievement_earned",
		kind: kind,
		detail: detail,
	},
	{
		label: "spec_target_updated",
		kind: kind,
		detail: detail,
	},
	{
		label: "spawner_give_item",
		kind: kind,
		detail: detail,
	},
	{
		label: "create_panic_event",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_pills",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_weapons",
		kind: kind,
		detail: detail,
	},
	{
		label: "entity_visible",
		kind: kind,
		detail: detail,
	},
	{
		label: "weapon_spawn_visible",
		kind: kind,
		detail: detail,
	},
	{
		label: "boomer_near",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_pre_radio",
		kind: kind,
		detail: detail,
	},
	{
		label: "started_pre_radio",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_radio",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_gas_truck",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_panic_button",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_elevator_button",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_lift_button",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_church_door",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_emergency_door",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_crane",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_bridge",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_gas_can_panic",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_van_panic",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_mainstreet",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_train_lever",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_disturbance",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_scavenge_goal",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_scavenge_leave_area",
		kind: kind,
		detail: detail,
	},
	{
		label: "begin_scavenge_overtime",
		kind: kind,
		detail: detail,
	},
	{
		label: "scavenge_round_start",
		kind: kind,
		detail: detail,
	},
	{
		label: "scavenge_round_halftime",
		kind: kind,
		detail: detail,
	},
	{
		label: "scavenge_round_finished",
		kind: kind,
		detail: detail,
	},
	{
		label: "scavenge_score_tied",
		kind: kind,
		detail: detail,
	},
	{
		label: "versus_round_start",
		kind: kind,
		detail: detail,
	},
	{
		label: "gascan_pour_blocked",
		kind: kind,
		detail: detail,
	},
	{
		label: "gascan_pour_completed",
		kind: kind,
		detail: detail,
	},
	{
		label: "gascan_dropped",
		kind: kind,
		detail: detail,
	},
	{
		label: "gascan_pour_interrupted",
		kind: kind,
		detail: detail,
	},
	{
		label: "scavenge_match_finished",
		kind: kind,
		detail: detail,
	},
	{
		label: "versus_match_finished",
		kind: kind,
		detail: detail,
	},
	{
		label: "use_target",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_use",
		kind: kind,
		detail: detail,
	},
	{
		label: "friendly_fire",
		kind: kind,
		detail: detail,
	},
	{
		label: "gameinstructor_draw",
		kind: kind,
		detail: detail,
	},
	{
		label: "gameinstructor_nodraw",
		kind: kind,
		detail: detail,
	},
	{
		label: "request_weapon_stats",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_talking_state",
		kind: kind,
		detail: detail,
	},
	{
		label: "weapon_pickup",
		kind: kind,
		detail: detail,
	},
	{
		label: "hunter_punched",
		kind: kind,
		detail: detail,
	},
	{
		label: "hunter_headshot",
		kind: kind,
		detail: detail,
	},
	{
		label: "zombie_ignited",
		kind: kind,
		detail: detail,
	},
	{
		label: "boomer_exploded",
		kind: kind,
		detail: detail,
	},
	{
		label: "non_pistol_fired",
		kind: kind,
		detail: detail,
	},
	{
		label: "weapon_fire_at_40",
		kind: kind,
		detail: detail,
	},
	{
		label: "total_ammo_below_40",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_hurt_concise",
		kind: kind,
		detail: detail,
	},
	{
		label: "tank_killed",
		kind: kind,
		detail: detail,
	},
	{
		label: "achievement_write_failed",
		kind: kind,
		detail: detail,
	},
	{
		label: "ghost_spawn_time",
		kind: kind,
		detail: detail,
	},
	{
		label: "survival_at_30min",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_pre_drawbridge",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_drawbridge",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_perimeter",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_deactivate_alarm",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_impound_lot",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_decon",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_mall_window",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_mall_alarm",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_coaster",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_coaster_stop",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_decon_wait",
		kind: kind,
		detail: detail,
	},
	{
		label: "gauntlet_finale_start",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_float",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_ferry_button",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_hatch_button",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_shack_button",
		kind: kind,
		detail: detail,
	},
	{
		label: "upgrade_incendiary_ammo",
		kind: kind,
		detail: detail,
	},
	{
		label: "upgrade_explosive_ammo",
		kind: kind,
		detail: detail,
	},
	{
		label: "receive_upgrade",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_vehicle_arrival",
		kind: kind,
		detail: detail,
	},
	{
		label: "mounted_gun_start",
		kind: kind,
		detail: detail,
	},
	{
		label: "mounted_gun_overheated",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_burger_sign",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_carousel_button",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_carousel_destination",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_stage_lighting",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_stage_finale_start",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_stage_survival_start",
		kind: kind,
		detail: detail,
	},
	{
		label: "ability_out_of_range",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_stage_pyrotechnics",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_c3m4_radio1",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_c3m4_radio2",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_gates_are_open",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_c2m4_ticketbooth",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_c3m4_rescue",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_hotel_elevator_doors",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_gun_shop_tanker",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_gun_shop",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_store_alarm",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_store_item",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_store_item_stop",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_survival_generic",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_survival_alarm",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_survival_radio",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_survival_carousel",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_return_item",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_save_items",
		kind: kind,
		detail: detail,
	},
	{
		label: "spit_burst",
		kind: kind,
		detail: detail,
	},
	{
		label: "entered_spit",
		kind: kind,
		detail: detail,
	},
	{
		label: "temp_c4m1_getgas",
		kind: kind,
		detail: detail,
	},
	{
		label: "temp_c4m3_return_to_boat",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_c1m4_finale",
		kind: kind,
		detail: detail,
	},
	{
		label: "c1m4_scavenge_instructions",
		kind: kind,
		detail: detail,
	},
	{
		label: "punched_clown",
		kind: kind,
		detail: detail,
	},
	{
		label: "charger_killed",
		kind: kind,
		detail: detail,
	},
	{
		label: "spitter_killed",
		kind: kind,
		detail: detail,
	},
	{
		label: "jockey_ride",
		kind: kind,
		detail: detail,
	},
	{
		label: "jockey_ride_end",
		kind: kind,
		detail: detail,
	},
	{
		label: "jockey_killed",
		kind: kind,
		detail: detail,
	},
	{
		label: "non_melee_fired",
		kind: kind,
		detail: detail,
	},
	{
		label: "infected_decapitated",
		kind: kind,
		detail: detail,
	},
	{
		label: "upgrade_pack_added",
		kind: kind,
		detail: detail,
	},
	{
		label: "vomit_bomb_tank",
		kind: kind,
		detail: detail,
	},
	{
		label: "triggered_car_alarm",
		kind: kind,
		detail: detail,
	},
	{
		label: "panic_event_finished",
		kind: kind,
		detail: detail,
	},
	{
		label: "charger_charge_start",
		kind: kind,
		detail: detail,
	},
	{
		label: "charger_charge_end",
		kind: kind,
		detail: detail,
	},
	{
		label: "charger_carry_start",
		kind: kind,
		detail: detail,
	},
	{
		label: "charger_carry_end",
		kind: kind,
		detail: detail,
	},
	{
		label: "charger_impact",
		kind: kind,
		detail: detail,
	},
	{
		label: "charger_pummel_start",
		kind: kind,
		detail: detail,
	},
	{
		label: "charger_pummel_end",
		kind: kind,
		detail: detail,
	},
	{
		label: "strongman_bell_knocked_off",
		kind: kind,
		detail: detail,
	},
	{
		label: "molotov_thrown",
		kind: kind,
		detail: detail,
	},
	{
		label: "gas_can_forced_drop",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_need_gnome_to_continue",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_survivor_glows_disabled",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_item_glows_disabled",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_rescue_disabled",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_bodyshots_reduced",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_witch_instant_kill",
		kind: kind,
		detail: detail,
	},
	{
		label: "set_instructor_group_enabled",
		kind: kind,
		detail: detail,
	},
	{
		label: "stashwhacker_game_won",
		kind: kind,
		detail: detail,
	},
	{
		label: "versus_marker_reached",
		kind: kind,
		detail: detail,
	},
	{
		label: "start_score_animation",
		kind: kind,
		detail: detail,
	},
	{
		label: "survival_round_start",
		kind: kind,
		detail: detail,
	},
	{
		label: "scavenge_gas_can_destroyed",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_sewer_gate",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_sewer_run",
		kind: kind,
		detail: detail,
	},
	{
		label: "explain_c6m3_finale",
		kind: kind,
		detail: detail,
	},
	{
		label: "finale_bridge_lowering",
		kind: kind,
		detail: detail,
	},
	{
		label: "m60_streak_ended",
		kind: kind,
		detail: detail,
	},
	{
		label: "chair_charged",
		kind: kind,
		detail: detail,
	},
	{
		label: "song_played",
		kind: kind,
		detail: detail,
	},
	{
		label: "foot_locker_opened",
		kind: kind,
		detail: detail,
	}
]