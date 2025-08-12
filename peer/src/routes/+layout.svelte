<script lang="ts">
  import { ModeWatcher } from "mode-watcher";
  import { AppSidebar } from "$lib/components/custom/index.js";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import { Toaster } from "$lib/components/ui/sonner/index.js";
  import { toast } from "svelte-sonner";
  import { listen, once, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  let { children } = $props();
  import "../app.css";

  type Peer = {
    username: string;
    node_id: string;
  };
  const unlisteners: Array<UnlistenFn> = [];

  onMount(() => {
    listen<Peer>("peer::added", (event) => {
      toast.info(`New peer added: ${event.payload.username}`, {
        description: `Node ID: ${event.payload.node_id}`,
      });
    }).then((unlisten) => {
      unlisteners.push(unlisten);
    });
    listen<[Peer, Peer]>("peer::username_changed", (event) => {
      toast.info(
        `Peer username changed: ${event.payload[0].username} to ${event.payload[1].username}`,
        {
          description: `Node ID: ${event.payload[0].node_id}`,
        },
      );
    }).then((unlisten) => {
      unlisteners.push(unlisten);
    });
    listen<Peer>("peer::left", (event) => {
      toast.info(`Peer left: ${event.payload.username}`, {
        description: `Node ID: ${event.payload.node_id}`,
      });
    }).then((unlisten) => {
      unlisteners.push(unlisten);
    });
  });

  onDestroy(() => {
    unlisteners.forEach((unlisten) => unlisten());
  });
</script>

<ModeWatcher defaultMode="dark" />
<div class="flex h-screen">
  <Sidebar.Provider>
    <AppSidebar />
    <main class="w-full">
      <Sidebar.Trigger />
      {@render children?.()}
    </main>
  </Sidebar.Provider>
  <main class="flex-1 overflow-y-auto"></main>
</div>
<Toaster position="top-right" />
