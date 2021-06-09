import { CompletionItem, CompletionItemKind } from "vscode";

const detail: string = "Left 4 Dead Event";
const kind: CompletionItemKind = CompletionItemKind.Keyword;

export const l4dEvents: CompletionItem[] = [
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
		label: "weapon_give_duplicate_fail",
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
	}
]