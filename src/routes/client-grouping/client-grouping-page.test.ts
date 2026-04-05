import { describe, expect, it } from "vitest";
import type { ClientGroupDetail, ThumbnailSetting } from "$models/domain";
import {
  availableToAdd,
  orderedMemberTitles,
  reorderTitles,
} from "$lib/client-grouping/client-grouping-helpers";

function makeGroup(memberSpecs: Array<{ title: string; order: number }>): ClientGroupDetail {
  return {
    id: 1,
    profileId: 1,
    name: "Test Group",
    cycleForwardHotkey: "",
    cycleBackwardHotkey: "",
    members: memberSpecs.map((member, index) => ({
      id: index + 1,
      groupId: 1,
      windowTitle: member.title,
      displayOrder: member.order,
    })),
  } as unknown as ClientGroupDetail;
}

function makeThumbnailSettings(titles: string[]): ThumbnailSetting[] {
  return titles.map((title, index) => ({
    id: index + 1,
    profileId: 1,
    windowTitle: title,
    config: {
      x: 0,
      y: 0,
      width: 100,
      height: 100,
      clickThru: false,
      alwaysOnTop: false,
      opacity: 1,
      hiddenAltTab: false,
      cropLeft: 0,
      cropTop: 0,
      cropRight: 0,
      cropBottom: 0,
      scale: 1,
      hideWhenInactive: false,
    },
  })) as unknown as ThumbnailSetting[];
}

describe("client-grouping helpers", () => {
  describe("orderedMemberTitles", () => {
    it("sorts member titles by display order and title", () => {
      const group = makeGroup([
        { title: "Bravo", order: 2 },
        { title: "Alpha", order: 1 },
        { title: "Charlie", order: 2 },
      ]);

      expect(orderedMemberTitles(group)).toEqual(["Alpha", "Bravo", "Charlie"]);
    });

    it("returns an empty list for an empty group", () => {
      const group = makeGroup([]);
      expect(orderedMemberTitles(group)).toEqual([]);
    });

    it("does not mutate source member order", () => {
      const group = makeGroup([
        { title: "Zulu", order: 2 },
        { title: "Alpha", order: 1 },
      ]);
      const before = group.members.map((member) => member.windowTitle);

      orderedMemberTitles(group);

      expect(group.members.map((member) => member.windowTitle)).toEqual(before);
    });
  });

  describe("availableToAdd", () => {
    it("returns only titles that are not already in the group", () => {
      const group = makeGroup([
        { title: "Client A", order: 1 },
        { title: "Client B", order: 2 },
      ]);
      const settings = makeThumbnailSettings(["Client A", "Client B", "Client C"]);

      expect(availableToAdd(group, settings)).toEqual(["Client C"]);
    });

    it("keeps thumbnail settings order for available titles", () => {
      const group = makeGroup([{ title: "Client C", order: 1 }]);
      const settings = makeThumbnailSettings(["Client B", "Client A", "Client C", "Client D"]);

      expect(availableToAdd(group, settings)).toEqual(["Client B", "Client A", "Client D"]);
    });

    it("returns all thumbnail titles when the group has no members", () => {
      const group = makeGroup([]);
      const settings = makeThumbnailSettings(["Client A", "Client B"]);

      expect(availableToAdd(group, settings)).toEqual(["Client A", "Client B"]);
    });

    it("returns empty when every thumbnail client is already in the group", () => {
      const group = makeGroup([
        { title: "Client A", order: 1 },
        { title: "Client B", order: 2 },
      ]);
      const settings = makeThumbnailSettings(["Client A", "Client B"]);

      expect(availableToAdd(group, settings)).toEqual([]);
    });
  });

  describe("reorderTitles", () => {
    it("reorders titles when dropping before a target index", () => {
      const titles = ["A", "B", "C", "D"];

      expect(reorderTitles(titles, 1, 3)).toEqual(["A", "C", "B", "D"]);
      expect(reorderTitles(titles, 3, 1)).toEqual(["A", "D", "B", "C"]);
    });

    it("moves an item to the beginning", () => {
      const titles = ["A", "B", "C", "D"];
      expect(reorderTitles(titles, 2, 0)).toEqual(["C", "A", "B", "D"]);
    });

    it("moves an item to the end when drop index equals list length", () => {
      const titles = ["A", "B", "C", "D"];
      expect(reorderTitles(titles, 1, titles.length)).toEqual(["A", "C", "D", "B"]);
    });

    it("keeps ordering unchanged when dropping directly after itself", () => {
      const titles = ["A", "B", "C", "D"];
      expect(reorderTitles(titles, 1, 2)).toEqual(["A", "B", "C", "D"]);
    });

    it("does not mutate the original list", () => {
      const titles = ["A", "B", "C", "D"];
      const original = [...titles];

      reorderTitles(titles, 0, 3);

      expect(titles).toEqual(original);
    });
  });
});
