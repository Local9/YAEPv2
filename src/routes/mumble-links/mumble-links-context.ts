import { getContext, setContext } from "svelte";
import type { MumbleLinksController } from "./mumble-links-controller.svelte.js";

const MUMBLE_LINKS_PAGE_CTX = Symbol("mumbleLinksPage");

export function setMumbleLinksPageContext(ctrl: MumbleLinksController): void {
  setContext(MUMBLE_LINKS_PAGE_CTX, ctrl);
}

export function getMumbleLinksPageContext(): MumbleLinksController {
  const ctrl = getContext<MumbleLinksController | undefined>(MUMBLE_LINKS_PAGE_CTX);
  if (!ctrl) {
    throw new Error("Mumble links page context is not available");
  }
  return ctrl;
}
