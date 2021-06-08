import { CompletionItem, CompletionItemKind } from "vscode";

const detail: string = "CS:GO Event";
const kind: CompletionItemKind = CompletionItemKind.Keyword;

export const csgoEvents: CompletionItem[] = [
	{
		label: "player_death",
		kind: kind,
		detail: detail,
	},
	{
		label: "other_death",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_hurt",
		kind: kind,
		detail: detail,
	},
	{
		label: "item_purchase",
		kind: kind,
		detail: detail,
	},
	{
		label: "bomb_beginplant",
		kind: kind,
		detail: detail,
	},
	{
		label: "bomb_abortplant",
		kind: kind,
		detail: detail,
	},
	{
		label: "bomb_planted",
		kind: kind,
		detail: detail,
	},
	{
		label: "bomb_defused",
		kind: kind,
		detail: detail,
	},
	{
		label: "bomb_exploded",
		kind: kind,
		detail: detail,
	},
	{
		label: "bomb_dropped",
		kind: kind,
		detail: detail,
	},
	{
		label: "bomb_pickup",
		kind: kind,
		detail: detail,
	},
	{
		label: "defuser_dropped",
		kind: kind,
		detail: detail,
	},
	{
		label: "defuser_pickup",
		kind: kind,
		detail: detail,
	},
	{
		label: "announce_phase_end",
		kind: kind,
		detail: detail,
	},
	{
		label: "cs_intermission",
		kind: kind,
		detail: detail,
	},
	{
		label: "bomb_begindefuse",
		kind: kind,
		detail: detail,
	},
	{
		label: "bomb_abortdefuse",
		kind: kind,
		detail: detail,
	},
	{
		label: "hostage_follows",
		kind: kind,
		detail: detail,
	},
	{
		label: "hostage_hurt",
		kind: kind,
		detail: detail,
	},
	{
		label: "hostage_killed",
		kind: kind,
		detail: detail,
	},
	{
		label: "hostage_rescued",
		kind: kind,
		detail: detail,
	},
	{
		label: "hostage_stops_following",
		kind: kind,
		detail: detail,
	},
	{
		label: "hostage_rescued_all",
		kind: kind,
		detail: detail,
	},
	{
		label: "hostage_call_for_help",
		kind: kind,
		detail: detail,
	},
	{
		label: "vip_escaped",
		kind: kind,
		detail: detail,
	},
	{
		label: "vip_killed",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_radio",
		kind: kind,
		detail: detail,
	},
	{
		label: "bomb_beep",
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
		label: "grenade_thrown",
		kind: kind,
		detail: detail,
	},
	{
		label: "weapon_outofammo",
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
		label: "silencer_detach",
		kind: kind,
		detail: detail,
	},
	{
		label: "inspect_weapon",
		kind: kind,
		detail: detail,
	},
	{
		label: "weapon_zoom_rifle",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_spawned",
		kind: kind,
		detail: detail,
	},
	{
		label: "item_pickup",
		kind: kind,
		detail: detail,
	},
	{
		label: "item_pickup_slerp",
		kind: kind,
		detail: detail,
	},
	{
		label: "item_pickup_failed",
		kind: kind,
		detail: detail,
	},
	{
		label: "item_remove",
		kind: kind,
		detail: detail,
	},
	{
		label: "ammo_pickup",
		kind: kind,
		detail: detail,
	},
	{
		label: "item_equip",
		kind: kind,
		detail: detail,
	},
	{
		label: "enter_buyzone",
		kind: kind,
		detail: detail,
	},
	{
		label: "exit_buyzone",
		kind: kind,
		detail: detail,
	},
	{
		label: "buytime_ended",
		kind: kind,
		detail: detail,
	},
	{
		label: "enter_bombzone",
		kind: kind,
		detail: detail,
	},
	{
		label: "exit_bombzone",
		kind: kind,
		detail: detail,
	},
	{
		label: "enter_rescue_zone",
		kind: kind,
		detail: detail,
	},
	{
		label: "exit_rescue_zone",
		kind: kind,
		detail: detail,
	},
	{
		label: "silencer_off",
		kind: kind,
		detail: detail,
	},
	{
		label: "silencer_on",
		kind: kind,
		detail: detail,
	},
	{
		label: "buymenu_open",
		kind: kind,
		detail: detail,
	},
	{
		label: "buymenu_close",
		kind: kind,
		detail: detail,
	},
	{
		label: "round_prestart",
		kind: kind,
		detail: detail,
	},
	{
		label: "round_poststart",
		kind: kind,
		detail: detail,
	},
	{
		label: "round_start",
		kind: kind,
		detail: detail,
	},
	{
		label: "round_end",
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
		label: "flashbang_detonate",
		kind: kind,
		detail: detail,
	},
	{
		label: "smokegrenade_detonate",
		kind: kind,
		detail: detail,
	},
	{
		label: "smokegrenade_expired",
		kind: kind,
		detail: detail,
	},
	{
		label: "molotov_detonate",
		kind: kind,
		detail: detail,
	},
	{
		label: "decoy_detonate",
		kind: kind,
		detail: detail,
	},
	{
		label: "decoy_started",
		kind: kind,
		detail: detail,
	},
	{
		label: "tagrenade_detonate",
		kind: kind,
		detail: detail,
	},
	{
		label: "inferno_startburn",
		kind: kind,
		detail: detail,
	},
	{
		label: "inferno_expire",
		kind: kind,
		detail: detail,
	},
	{
		label: "inferno_extinguish",
		kind: kind,
		detail: detail,
	},
	{
		label: "decoy_firing",
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
		label: "door_moving",
		kind: kind,
		detail: detail,
	},
	{
		label: "round_freeze_end",
		kind: kind,
		detail: detail,
	},
	{
		label: "mb_input_lock_success",
		kind: kind,
		detail: detail,
	},
	{
		label: "mb_input_lock_cancel",
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
		label: "player_stats_updated",
		kind: kind,
		detail: detail,
	},
	{
		label: "achievement_info_loaded",
		kind: kind,
		detail: detail,
	},
	{
		label: "spec_target_updated",
		kind: kind,
		detail: detail,
	},
	{
		label: "spec_mode_updated",
		kind: kind,
		detail: detail,
	},
	{
		label: "hltv_changed_mode",
		kind: kind,
		detail: detail,
	},
	{
		label: "cs_game_disconnected",
		kind: kind,
		detail: detail,
	},
	{
		label: "cs_win_panel_round",
		kind: kind,
		detail: detail,
	},
	{
		label: "cs_win_panel_match",
		kind: kind,
		detail: detail,
	},
	{
		label: "cs_match_end_restart",
		kind: kind,
		detail: detail,
	},
	{
		label: "cs_pre_restart",
		kind: kind,
		detail: detail,
	},
	{
		label: "show_freezepanel",
		kind: kind,
		detail: detail,
	},
	{
		label: "hide_freezepanel",
		kind: kind,
		detail: detail,
	},
	{
		label: "freezecam_started",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_avenged_teammate",
		kind: kind,
		detail: detail,
	},
	{
		label: "achievement_earned",
		kind: kind,
		detail: detail,
	},
	{
		label: "achievement_earned_local",
		kind: kind,
		detail: detail,
	},
	{
		label: "item_found",
		kind: kind,
		detail: detail,
	},
	{
		label: "items_gifted",
		kind: kind,
		detail: detail,
	},
	{
		label: "repost_xbox_achievements",
		kind: kind,
		detail: detail,
	},
	{
		label: "match_end_conditions",
		kind: kind,
		detail: detail,
	},
	{
		label: "round_mvp",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_decal",
		kind: kind,
		detail: detail,
	},
	{
		label: "teamplay_round_start",
		kind: kind,
		detail: detail,
	},
	{
		label: "show_survival_respawn_status",
		kind: kind,
		detail: detail,
	},
	{
		label: "client_disconnect",
		kind: kind,
		detail: detail,
	},
	{
		label: "gg_player_levelup",
		kind: kind,
		detail: detail,
	},
	{
		label: "ggtr_player_levelup",
		kind: kind,
		detail: detail,
	},
	{
		label: "assassination_target_killed",
		kind: kind,
		detail: detail,
	},
	{
		label: "ggprogressive_player_levelup",
		kind: kind,
		detail: detail,
	},
	{
		label: "gg_killed_enemy",
		kind: kind,
		detail: detail,
	},
	{
		label: "gg_final_weapon_achieved",
		kind: kind,
		detail: detail,
	},
	{
		label: "gg_bonus_grenade_achieved",
		kind: kind,
		detail: detail,
	},
	{
		label: "switch_team",
		kind: kind,
		detail: detail,
	},
	{
		label: "gg_leader",
		kind: kind,
		detail: detail,
	},
	{
		label: "gg_team_leader",
		kind: kind,
		detail: detail,
	},
	{
		label: "gg_player_impending_upgrade",
		kind: kind,
		detail: detail,
	},
	{
		label: "write_profile_data",
		kind: kind,
		detail: detail,
	},
	{
		label: "trial_time_expired",
		kind: kind,
		detail: detail,
	},
	{
		label: "update_matchmaking_stats",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_reset_vote",
		kind: kind,
		detail: detail,
	},
	{
		label: "enable_restart_voting",
		kind: kind,
		detail: detail,
	},
	{
		label: "sfuievent",
		kind: kind,
		detail: detail,
	},
	{
		label: "start_vote",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_given_c4",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_become_ghost",
		kind: kind,
		detail: detail,
	},
	{
		label: "gg_reset_round_start_sounds",
		kind: kind,
		detail: detail,
	},
	{
		label: "tr_player_flashbanged",
		kind: kind,
		detail: detail,
	},
	{
		label: "tr_mark_complete",
		kind: kind,
		detail: detail,
	},
	{
		label: "tr_mark_best_time",
		kind: kind,
		detail: detail,
	},
	{
		label: "tr_exit_hint_trigger",
		kind: kind,
		detail: detail,
	},
	{
		label: "bot_takeover",
		kind: kind,
		detail: detail,
	},
	{
		label: "tr_show_finish_msgbox",
		kind: kind,
		detail: detail,
	},
	{
		label: "tr_show_exit_msgbox",
		kind: kind,
		detail: detail,
	},
	{
		label: "reset_player_controls",
		kind: kind,
		detail: detail,
	},
	{
		label: "jointeam_failed",
		kind: kind,
		detail: detail,
	},
	{
		label: "teamchange_pending",
		kind: kind,
		detail: detail,
	},
	{
		label: "material_default_complete",
		kind: kind,
		detail: detail,
	},
	{
		label: "cs_prev_next_spectator",
		kind: kind,
		detail: detail,
	},
	{
		label: "cs_handle_ime_event",
		kind: kind,
		detail: detail,
	},
	{
		label: "nextlevel_changed",
		kind: kind,
		detail: detail,
	},
	{
		label: "seasoncoin_levelup",
		kind: kind,
		detail: detail,
	},
	{
		label: "tournament_reward",
		kind: kind,
		detail: detail,
	},
	{
		label: "start_halftime",
		kind: kind,
		detail: detail,
	},
	{
		label: "ammo_refill",
		kind: kind,
		detail: detail,
	},
	{
		label: "parachute_pickup",
		kind: kind,
		detail: detail,
	},
	{
		label: "dronegun_attack",
		kind: kind,
		detail: detail,
	},
	{
		label: "drone_dispatched",
		kind: kind,
		detail: detail,
	},
	{
		label: "loot_crate_visible",
		kind: kind,
		detail: detail,
	},
	{
		label: "loot_crate_opened",
		kind: kind,
		detail: detail,
	},
	{
		label: "open_crate_instr",
		kind: kind,
		detail: detail,
	},
	{
		label: "smoke_beacon_paradrop",
		kind: kind,
		detail: detail,
	},
	{
		label: "survival_paradrop_spawn",
		kind: kind,
		detail: detail,
	},
	{
		label: "survival_paradrop_break",
		kind: kind,
		detail: detail,
	},
	{
		label: "drone_cargo_detached",
		kind: kind,
		detail: detail,
	},
	{
		label: "drone_above_roof",
		kind: kind,
		detail: detail,
	},
	{
		label: "choppers_incoming_warning",
		kind: kind,
		detail: detail,
	},
	{
		label: "firstbombs_incoming_warning",
		kind: kind,
		detail: detail,
	},
	{
		label: "dz_item_interaction",
		kind: kind,
		detail: detail,
	},
	{
		label: "snowball_hit_player_face",
		kind: kind,
		detail: detail,
	},
	{
		label: "survival_teammate_respawn",
		kind: kind,
		detail: detail,
	},
	{
		label: "survival_no_respawns_warning",
		kind: kind,
		detail: detail,
	},
	{
		label: "survival_no_respawns_final",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_ping",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_ping_stop",
		kind: kind,
		detail: detail,
	},
	{
		label: "guardian_wave_restart",
		kind: kind,
		detail: detail,
	}
]