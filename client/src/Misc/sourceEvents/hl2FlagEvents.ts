import { CompletionItem, CompletionItemKind } from "vscode";

const detail: string = "Half-Life 2: Capture the Flag Event";
const kind: CompletionItemKind = CompletionItemKind.Keyword;

export const hl2FlagEvents: CompletionItem[] = [
	{
		label: "player_death",
		kind: kind,
		detail: detail,
	},
	{
		label: "ctf_flag_capture",
		kind: kind,
		detail: detail,
	},
	{
		label: "ctf_flag_stolen",
		kind: kind,
		detail: detail,
	},
	{
		label: "ctf_flag_return",
		kind: kind,
		detail: detail,
	},
	{
		label: "ctf_flag_assist",
		kind: kind,
		detail: detail,
	},
	{
		label: "ctf_flag_defend",
		kind: kind,
		detail: detail,
	},
	{
		label: "ctf_flag_dominate",
		kind: kind,
		detail: detail,
	},
	{
		label: "ctf_protect_carrier",
		kind: kind,
		detail: detail,
	},
	{
		label: "ctf_kill_carrier",
		kind: kind,
		detail: detail,
	},
	{
		label: "ctf_map_end",
		kind: kind,
		detail: detail,
	},
	{
		label: "ctf_round_start",
		kind: kind,
		detail: detail,
	}
]