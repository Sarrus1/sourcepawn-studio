import { CompletionItem } from "vscode";

import { sourceGenericEvents } from "./sourceEvents/sourceGeneric";
import { csgoEvents } from "./sourceEvents/csgoEvents";
import { bmEvents } from "./sourceEvents/bmEvents";
import { ccEvents } from "./sourceEvents/ccEvents";
import { dota2Events } from "./sourceEvents/dota2Events";
import { dystopiaEvents } from "./sourceEvents/dystopiaEvents";
import { gmEvents } from "./sourceEvents/gmEvents";
import { hl2DmEvents } from "./sourceEvents/hl2DmEvents";
import { hl2FlagEvents } from "./sourceEvents/hl2FlagEvents";
import { hsEvents } from "./sourceEvents/hsEvents";
import { igEvents } from "./sourceEvents/igEvents";
import { insurgencyEvents } from "./sourceEvents/insurgencyEvents";
import { l4d2Events } from "./sourceEvents/l4d2Events";
import { l4dEvents } from "./sourceEvents/l4dEvents";
import { neotokyoEvents } from "./sourceEvents/neotokyoEvents";
import { pdsEvents } from "./sourceEvents/pdsEvents";
import { sfEvents } from "./sourceEvents/sfEvents";

export const events: CompletionItem[] = [].concat(
  sourceGenericEvents,
  csgoEvents,
  bmEvents,
  ccEvents,
  dota2Events,
  dystopiaEvents,
  gmEvents,
  hl2DmEvents,
  hl2FlagEvents,
  hsEvents,
  igEvents,
  insurgencyEvents,
  l4d2Events,
  l4dEvents,
  neotokyoEvents,
  pdsEvents,
  sfEvents
);
