import type { ClientGroupDetail, ThumbnailSetting } from "$models/domain";

export function orderedMemberTitles(group: ClientGroupDetail): string[] {
  return [...group.members]
    .sort(
      (a, b) =>
        a.displayOrder - b.displayOrder || a.windowTitle.localeCompare(b.windowTitle),
    )
    .map((member) => member.windowTitle);
}

export function availableToAdd(
  group: ClientGroupDetail,
  thumbnailSettings: ThumbnailSetting[],
): string[] {
  const inGroup = new Set(group.members.map((member) => member.windowTitle));
  return thumbnailSettings.map((setting) => setting.windowTitle).filter((title) => !inGroup.has(title));
}

export function reorderTitles(list: string[], fromIndex: number, toBeforeIndex: number): string[] {
  const next = [...list];
  const [item] = next.splice(fromIndex, 1);
  let destinationIndex = toBeforeIndex;
  if (fromIndex < destinationIndex) {
    destinationIndex -= 1;
  }
  next.splice(destinationIndex, 0, item);
  return next;
}
