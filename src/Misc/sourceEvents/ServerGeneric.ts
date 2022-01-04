import { CompletionItem, CompletionItemKind } from "vscode";

const detail: string = "Generic Source Server Event";
const kind: CompletionItemKind = CompletionItemKind.Keyword;

export const serverGenericEvents: CompletionItem[] = [
	{
		label: "server_spawn",
		kind: kind,
		detail: detail,
	},
	{
		label: "server_shutdown",
		kind: kind,
		detail: detail,
	},
	{
		label: "server_cvar",
		kind: kind,
		detail: detail,
	},
	{
		label: "server_message",
		kind: kind,
		detail: detail,
	},
	{
		label: "server_addban",
		kind: kind,
		detail: detail,
	},
	{
		label: "server_removeban",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_connect",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_connect_client",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_info",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_disconnect",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_activate",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_say",
		kind: kind,
		detail: detail,
	}
]