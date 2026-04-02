<script lang="ts">
  import type { Component } from "svelte";
  import type { MumbleFolderIconKey } from "./folder-icon-keys";
  import { MUMBLE_FOLDER_ICON_KEYS } from "./folder-icon-keys";
  import BellIcon from "@lucide/svelte/icons/bell";
  import BriefcaseIcon from "@lucide/svelte/icons/briefcase";
  import CogIcon from "@lucide/svelte/icons/cog";
  import FolderIcon from "@lucide/svelte/icons/folder";
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";
  import Gamepad2Icon from "@lucide/svelte/icons/gamepad-2";
  import HeadphonesIcon from "@lucide/svelte/icons/headphones";
  import HeadsetIcon from "@lucide/svelte/icons/headset";
  import HeartIcon from "@lucide/svelte/icons/heart";
  import HomeIcon from "@lucide/svelte/icons/home";
  import MapIcon from "@lucide/svelte/icons/map";
  import MegaphoneIcon from "@lucide/svelte/icons/megaphone";
  import MessageCircleIcon from "@lucide/svelte/icons/message-circle";
  import MicIcon from "@lucide/svelte/icons/mic";
  import PhoneIcon from "@lucide/svelte/icons/phone";
  import RadioIcon from "@lucide/svelte/icons/radio";
  import SatelliteDishIcon from "@lucide/svelte/icons/satellite-dish";
  import ShieldIcon from "@lucide/svelte/icons/shield";
  import StarIcon from "@lucide/svelte/icons/star";
  import UserIcon from "@lucide/svelte/icons/user";
  import UsersIcon from "@lucide/svelte/icons/users";
  import Volume2Icon from "@lucide/svelte/icons/volume-2";
  import WrenchIcon from "@lucide/svelte/icons/wrench";
  import ZapIcon from "@lucide/svelte/icons/zap";

  const ICON_MAP = {
    headphones: HeadphonesIcon,
    folder: FolderIcon,
    "folder-open": FolderOpenIcon,
    users: UsersIcon,
    user: UserIcon,
    radio: RadioIcon,
    mic: MicIcon,
    "message-circle": MessageCircleIcon,
    shield: ShieldIcon,
    star: StarIcon,
    heart: HeartIcon,
    home: HomeIcon,
    briefcase: BriefcaseIcon,
    "gamepad-2": Gamepad2Icon,
    map: MapIcon,
    zap: ZapIcon,
    "satellite-dish": SatelliteDishIcon,
    "volume-2": Volume2Icon,
    headset: HeadsetIcon,
    phone: PhoneIcon,
    megaphone: MegaphoneIcon,
    wrench: WrenchIcon,
    bell: BellIcon,
    cog: CogIcon
  } satisfies Record<MumbleFolderIconKey, Component>;

  let { iconKey = null, class: className = "" }: { iconKey?: string | null; class?: string } = $props();

  function resolvedKey(raw: string | null | undefined): MumbleFolderIconKey {
    if (raw == null || raw === "") return "headphones";
    const t = raw.trim().toLowerCase();
    return (MUMBLE_FOLDER_ICON_KEYS as readonly string[]).includes(t)
      ? (t as MumbleFolderIconKey)
      : "headphones";
  }

  let Icon = $derived(ICON_MAP[resolvedKey(iconKey)]);
</script>

<Icon class={className} aria-hidden="true" />
