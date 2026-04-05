import { backend } from "$services/backend";
import type { MumbleFolder, MumbleLink, MumbleTreeSnapshot } from "$models/domain";
import { deriveMumbleLinkName, isAllowedMumbleLinkUrl } from "$lib/utils/mumble-url";
import { finiteDisplayOrder } from "$lib/utils/mumble-display-order";
import { toast } from "svelte-sonner";

const MUMBLE_LINK_HOTKEY_CAPTURE = "mumbleLink";

export class MumbleLinksController {
  tree = $state<MumbleTreeSnapshot | null>(null);
  captureHotkeyLinkId = $state<number | null>(null);
  linkDraft = $state<{ serverGroupId: number; folderId: number | null } | null>(null);
  linkDraftUrl = $state("");
  linkDraftName = $state("");
  folderDraft = $state<{ serverGroupId: number; parentFolderId: number | null } | null>(null);
  folderDraftName = $state("");
  folderDraftIconKey = $state("");
  confirmDelete = $state<{
    kind: "folder" | "link";
    id: number;
    label: string;
  } | null>(null);
  confirmOpen = $state(false);
  /** Folder id -> expanded; omitted ids default to expanded (true). */
  folderExpandedById = $state<Record<number, boolean>>({});

  sortedGroups = $derived(
    this.tree
      ? [...this.tree.groups].sort(
          (a, b) => a.displayOrder - b.displayOrder || a.name.localeCompare(b.name)
        )
      : []
  );

  multipleServerGroups = $derived(this.sortedGroups.length > 1);

  private userSafeMumbleErrorMessage(): string {
    return "Unable to save Mumble link changes right now. Please try again.";
  }

  isFolderExpanded(folderId: number): boolean {
    return this.folderExpandedById[folderId] !== false;
  }

  setFolderExpanded(folderId: number, expanded: boolean) {
    this.folderExpandedById = { ...this.folderExpandedById, [folderId]: expanded };
  }

  onEnterSubmit(
    e: KeyboardEvent,
    action: () => void | Promise<void>,
    options?: { stopBubble?: boolean }
  ) {
    if (e.key !== "Enter") return;
    e.preventDefault();
    if (options?.stopBubble) e.stopPropagation();
    void action();
  }

  async refresh(): Promise<boolean> {
    try {
      this.tree = await backend.getMumbleTree();
      return true;
    } catch {
      toast.error(this.userSafeMumbleErrorMessage());
      return false;
    }
  }

  foldersForParent(gid: number, parentId: number | null): MumbleFolder[] {
    if (!this.tree) return [];
    return this.tree.folders
      .filter((f) => f.serverGroupId === gid && (f.parentFolderId ?? null) === parentId)
      .sort((a, b) => a.displayOrder - b.displayOrder || a.name.localeCompare(b.name));
  }

  linksForFolder(gid: number, folderId: number | null): MumbleLink[] {
    if (!this.tree) return [];
    return this.tree.links
      .filter((l) => l.serverGroupId === gid && (l.folderId ?? null) === folderId)
      .sort((a, b) => a.displayOrder - b.displayOrder || a.name.localeCompare(b.name));
  }

  maxFolderOrder(gid: number, parentId: number | null): number {
    const list = this.foldersForParent(gid, parentId);
    return list.reduce((m, f) => Math.max(m, finiteDisplayOrder(f.displayOrder)), -1);
  }

  maxLinkOrder(gid: number, folderId: number | null): number {
    const list = this.linksForFolder(gid, folderId);
    return list.reduce((m, l) => Math.max(m, finiteDisplayOrder(l.displayOrder)), -1);
  }

  openDeleteFolder(f: MumbleFolder) {
    this.confirmDelete = { kind: "folder", id: f.id, label: f.name };
    this.confirmOpen = true;
  }

  openDeleteLink(l: MumbleLink) {
    this.confirmDelete = { kind: "link", id: l.id, label: l.name };
    this.confirmOpen = true;
  }

  async executeDelete() {
    const c = this.confirmDelete;
    if (!c) return;
    try {
      if (c.kind === "folder") await backend.deleteMumbleFolder(c.id);
      else await backend.deleteMumbleLink(c.id);
      toast.success("Deleted");
      this.confirmOpen = false;
      this.confirmDelete = null;
      await this.refresh();
    } catch {
      toast.error(this.userSafeMumbleErrorMessage());
    }
  }

  cancelConfirmDelete() {
    this.confirmOpen = false;
    this.confirmDelete = null;
  }

  async saveFolder(f: MumbleFolder, opts?: { silent?: boolean }) {
    try {
      await backend.updateMumbleFolder(
        f.id,
        f.name.trim(),
        finiteDisplayOrder(f.displayOrder),
        f.iconKey ?? null
      );
      if (!opts?.silent) toast.success("Folder saved");
      await this.refresh();
    } catch {
      toast.error(this.userSafeMumbleErrorMessage());
    }
  }

  async saveLink(link: MumbleLink, opts?: { silent?: boolean }) {
    const url = link.url.trim();
    if (!isAllowedMumbleLinkUrl(url)) {
      toast.error("Link URL must start with mumble:// or https://");
      return;
    }
    try {
      await backend.updateMumbleLink(
        link.id,
        link.name.trim(),
        url,
        finiteDisplayOrder(link.displayOrder),
        link.hotkey.trim(),
        link.serverGroupId,
        link.folderId ?? null
      );
      if (!opts?.silent) toast.success("Link saved");
      await this.refresh();
    } catch {
      toast.error(this.userSafeMumbleErrorMessage());
    }
  }

  beginRootFolderDraft(gid: number) {
    this.folderDraft = { serverGroupId: gid, parentFolderId: null };
    this.folderDraftName = "";
    this.folderDraftIconKey = "";
  }

  beginSubfolderDraft(gid: number, parentFolderId: number) {
    this.folderDraft = { serverGroupId: gid, parentFolderId };
    this.folderDraftName = "";
    this.folderDraftIconKey = "";
  }

  cancelFolderDraft() {
    this.folderDraft = null;
    this.folderDraftName = "";
    this.folderDraftIconKey = "";
  }

  async submitFolderDraft() {
    if (!this.folderDraft || !this.folderDraftName.trim()) return;
    const ord = this.maxFolderOrder(this.folderDraft.serverGroupId, this.folderDraft.parentFolderId) + 1;
    try {
      await backend.createMumbleFolder(
        this.folderDraft.serverGroupId,
        this.folderDraft.parentFolderId,
        this.folderDraftName.trim(),
        ord,
        this.folderDraftIconKey === "" ? null : this.folderDraftIconKey
      );
      toast.success("Folder created");
      const ok = await this.refresh();
      if (ok) this.cancelFolderDraft();
    } catch {
      toast.error(this.userSafeMumbleErrorMessage());
    }
  }

  beginLinkDraft(gid: number, folderId: number | null) {
    this.linkDraft = { serverGroupId: gid, folderId };
    this.linkDraftUrl = "";
    this.linkDraftName = "";
  }

  cancelLinkDraft() {
    this.linkDraft = null;
    this.linkDraftUrl = "";
    this.linkDraftName = "";
  }

  onLinkDraftUrlInput(v: string) {
    this.linkDraftUrl = v;
    const d = deriveMumbleLinkName(v);
    if (d) this.linkDraftName = d;
  }

  async submitLinkDraft() {
    if (!this.linkDraft) return;
    const url = this.linkDraftUrl.trim();
    const name = this.linkDraftName.trim();
    if (!name || !isAllowedMumbleLinkUrl(url)) {
      toast.error("Enter a valid URL and name");
      return;
    }
    const ord = this.maxLinkOrder(this.linkDraft.serverGroupId, this.linkDraft.folderId) + 1;
    if (!Number.isFinite(ord)) {
      toast.error(this.userSafeMumbleErrorMessage());
      return;
    }
    try {
      await backend.createMumbleLink(
        name,
        url,
        ord,
        "",
        this.linkDraft.serverGroupId,
        this.linkDraft.folderId
      );
      toast.success("Link created");
      const ok = await this.refresh();
      if (ok) this.cancelLinkDraft();
    } catch {
      toast.error(this.userSafeMumbleErrorMessage());
    }
  }

  stopLinkHotkeyCapture() {
    this.captureHotkeyLinkId = null;
    void backend.hotkeysCaptureStop();
  }

  async startLinkHotkeyCapture(linkId: number) {
    this.captureHotkeyLinkId = linkId;
    try {
      await backend.hotkeysCaptureStart(MUMBLE_LINK_HOTKEY_CAPTURE, linkId);
    } catch {
      toast.error(this.userSafeMumbleErrorMessage());
      this.captureHotkeyLinkId = null;
    }
  }

  readonly hotkeyCaptureType = MUMBLE_LINK_HOTKEY_CAPTURE;

  async applyCapturedHotkey(linkId: number, hotkeyValue: string) {
    const link = this.tree?.links.find((l) => l.id === linkId);
    if (!link) {
      void this.refresh();
      return;
    }
    try {
      await backend.updateMumbleLink(
        linkId,
        link.name.trim(),
        link.url.trim(),
        finiteDisplayOrder(link.displayOrder),
        hotkeyValue.trim(),
        link.serverGroupId,
        link.folderId ?? null
      );
      toast.success("Hotkey saved");
      await this.refresh();
    } catch {
      toast.error(this.userSafeMumbleErrorMessage());
    }
  }
}
