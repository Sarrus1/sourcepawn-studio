import { CompletionItem, CompletionItemKind } from "vscode";

const detail: string = "Black Mesa Event";
const kind: CompletionItemKind = CompletionItemKind.Keyword;

export const bmEvents: CompletionItem[] = [
	{
		label: "freezecam_started",
		kind: kind,
		detail: detail,
	},
	{
		label: "spec_target_updated",
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
		label: "player_dominated",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_revenge",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_death",
		kind: kind,
		detail: detail,
	},
	{
		label: "broadcast_killstreak",
		kind: kind,
		detail: detail,
	},
	{
		label: "broadcast_teamsound",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_soda_machine",
		kind: kind,
		detail: detail,
	},
	{
		label: "item_pickup",
		kind: kind,
		detail: detail,
	},
	{
		label: "weapon_pickup",
		kind: kind,
		detail: detail,
	},
	{
		label: "tram_accelerate",
		kind: kind,
		detail: detail,
	},
	{
		label: "fire_mortar",
		kind: kind,
		detail: detail,
	},
	{
		label: "tram_client_state_change",
		kind: kind,
		detail: detail,
	},
	{
		label: "tram_control_state",
		kind: kind,
		detail: detail,
	},
	{
		label: "weapon_tau_overcharged",
		kind: kind,
		detail: detail,
	},
	{
		label: "weapon_gluon_fired",
		kind: kind,
		detail: detail,
	},
	{
		label: "npc_barnacle_grab_victim",
		kind: kind,
		detail: detail,
	},
	{
		label: "damage_indicator",
		kind: kind,
		detail: detail,
	}
]