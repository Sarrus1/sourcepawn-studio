import { CompletionItem, CompletionItemKind } from "vscode";

const detail: string = "Dota 2 Event";
const kind: CompletionItemKind = CompletionItemKind.Keyword;

export const dota2Events: CompletionItem[] = [
	{
		label: "modifier_event",
		kind: kind,
		detail: detail,
	},
	{
		label: "dota_player_kill",
		kind: kind,
		detail: detail,
	},
	{
		label: "dota_player_deny",
		kind: kind,
		detail: detail,
	},
	{
		label: "dota_barracks_kill",
		kind: kind,
		detail: detail,
	},
	{
		label: "dota_tower_kill",
		kind: kind,
		detail: detail,
	},
	{
		label: "dota_roshan_kill",
		kind: kind,
		detail: detail,
	},
	{
		label: "dota_courier_lost",
		kind: kind,
		detail: detail,
	},
	{
		label: "dota_courier_respawned",
		kind: kind,
		detail: detail,
	},
	{
		label: "dota_glyph_used",
		kind: kind,
		detail: detail,
	},
	{
		label: "dota_super_creeps",
		kind: kind,
		detail: detail,
	},
	{
		label: "dota_item_purchase",
		kind: kind,
		detail: detail,
	},
	{
		label: "dota_item_gifted",
		kind: kind,
		detail: detail,
	},
	{
		label: "dota_rune_pickup",
		kind: kind,
		detail: detail,
	},
	{
		label: "dota_rune_spotted",
		kind: kind,
		detail: detail,
	},
	{
		label: "dota_item_spotted",
		kind: kind,
		detail: detail,
	},
	{
		label: "dota_no_battle_points",
		kind: kind,
		detail: detail,
	},
	{
		label: "dota_chat_informational",
		kind: kind,
		detail: detail,
	},
	{
		label: "dota_action_item",
		kind: kind,
		detail: detail,
	},
	{
		label: "dota_match_done",
		kind: kind,
		detail: detail,
	}
]