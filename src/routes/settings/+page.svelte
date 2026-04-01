<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import { setMode } from "mode-watcher";
  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import * as Select from "$lib/components/ui/select";
  import { toast } from "svelte-sonner";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Field, FieldContent, FieldLabel } from "$lib/components/ui/field";
  import SaveIcon from "@lucide/svelte/icons/save";
  import SettingsIcon from "@lucide/svelte/icons/settings";

  let enableThumbnailDragging = $state(true);
  let startHidden = $state(false);
  let theme = $state("Dark");
  let saveStatus = $state("");
  let error = $state("");

  function userSafeErrorMessage(): string {
    return "Unable to save settings right now. Please try again.";
  }

  async function refresh() {
    try {
      const dragging = await backend.getAppSetting("EnableThumbnailDragging");
      const hidden = await backend.getAppSetting("StartHidden");
      const currentTheme = await backend.getAppSetting("Theme");
      enableThumbnailDragging = dragging == null ? true : dragging === "true";
      startHidden = hidden == null ? false : hidden === "true";
      theme = currentTheme ?? "Dark";
      error = "";
    } catch (e) {
      error = userSafeErrorMessage();
    }
  }

  async function save() {
    try {
      await backend.setAppSetting("EnableThumbnailDragging", String(enableThumbnailDragging));
      await backend.setAppSetting("StartHidden", String(startHidden));
      await backend.setAppSetting("Theme", theme);
      setMode(theme === "Light" ? "light" : "dark");
      saveStatus = "Settings saved";
      error = "";
    } catch (e) {
      error = userSafeErrorMessage();
    }
  }

  onMount(refresh);

  $effect(() => {
    if (saveStatus) toast.success(saveStatus);
  });

  $effect(() => {
    if (error) toast.error(error);
  });
</script>

<Card class="shadow-sm">
  <CardHeader>
    <div class="flex items-start gap-3">
      <SettingsIcon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
      <div>
        <CardTitle class="text-lg font-semibold tracking-tight">Settings</CardTitle>
        <CardDescription>App-level settings with persisted values.</CardDescription>
      </div>
    </div>
  </CardHeader>
  <CardContent>
    <div class="mt-4 grid max-w-3xl gap-3">
      <Field>
        <FieldContent class="flex w-full flex-row items-center gap-2 flex-initial!">
          <Checkbox
            id="settings-enable-thumbnail-dragging"
            bind:checked={enableThumbnailDragging}
            class="cursor-pointer"
          />
          <FieldLabel
            for="settings-enable-thumbnail-dragging"
            class="text-muted-foreground mb-0 cursor-pointer leading-snug font-normal"
          >
            Enable Thumbnail Dragging
          </FieldLabel>
        </FieldContent>
      </Field>
      <Field>
        <FieldContent class="flex w-full flex-row items-center gap-2 flex-initial!">
          <Checkbox id="settings-start-hidden" bind:checked={startHidden} class="cursor-pointer" />
          <FieldLabel
            for="settings-start-hidden"
            class="text-muted-foreground mb-0 cursor-pointer leading-snug font-normal"
          >
            Start Hidden
          </FieldLabel>
        </FieldContent>
      </Field>
      <Field class="max-w-md">
        <FieldLabel class="text-muted-foreground">Theme</FieldLabel>
        <FieldContent>
          <Select.Root
            type="single"
            bind:value={theme}
            items={[
              { value: "Dark", label: "Dark" },
              { value: "Light", label: "Light" },
            ]}
          >
            <Select.Trigger class="w-full">
              <span data-slot="select-value">{theme}</span>
            </Select.Trigger>
            <Select.Content>
              <Select.Item value="Dark">Dark</Select.Item>
              <Select.Item value="Light">Light</Select.Item>
            </Select.Content>
          </Select.Root>
        </FieldContent>
      </Field>
      <div>
        <Button onclick={save} class="gap-2">
          <SaveIcon class="size-4 shrink-0" aria-hidden="true" />
          Save settings
        </Button>
      </div>
    </div>
  </CardContent>
</Card>
