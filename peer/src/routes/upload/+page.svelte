<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { open } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { Trash2 } from "@lucide/svelte";
  import DirectoryTree, {
    type TreeNode,
  } from "$lib/components/custom/directorytree.svelte";
  import { toast } from "svelte-sonner";
  let rootNode: TreeNode[] = $state([]);
  onMount(() => {
    // This will run when the component is mounted
    loadFiles();
  });

  async function pickFolder() {
    const selected = await open({
      directory: false,
      multiple: true,
      title: "Select a folder",
    });
    if (!selected) {
      toast.info("No folders selected.");
      return;
    }
    const folderPaths = Array.isArray(selected) ? selected : [selected];
    const importPromises = folderPaths.map(async (folderPath) => {
      try {
        await invoke("add_path", { path: folderPath });
        return { folderPath };
      } catch (error) {
        throw { folderPath, error };
      }
    });

    toast.promise(Promise.all(importPromises), {
      loading: `Adding ${folderPaths.length} folder(s)...`,
      success: (results) => {
        const len = results.length;
        return `Successfully added ${len} folder(s).`;
      },
      error: (e: { folderPath: string; error: any }) => {
        return `While adding folder: ${e.folderPath} encountered an error: ${e.error}`;
      },
    });
    Promise.all(importPromises).then((res) => {
      loadFiles();
    });
  }
  async function clearAll() {
    await invoke("clear_files").then((res) => {
      loadFiles();
    });
  }
  async function loadFiles() {
    await invoke("get_uploaded_files_tree", {})
      .then((res) => {
        rootNode = res as TreeNode[];
      })
      .catch((e) => toast.error(`Error loading files: ${e}`));
  }
  function handleRemove(selectedNodesList: TreeNode[]) {
    // TODO
  }
</script>

<h1 class="text-2xl font-bold m-8 mb-4">Shared Files</h1>
<!-- Folder Picker Input -->
<Label class="mb-2 inline-flex">Add a folder:</Label>
<Button onclick={pickFolder} class="">Pick a folder</Button>

<!-- Clear All Button -->
<div class="flex justify-end m-8 mb-4">
  <Button class="" onclick={clearAll}>Clear All</Button>
</div>

<div class="space-y-4 m-8 max-w-full">
  <DirectoryTree data={rootNode} selectable={true} searchable={true}>
    {#snippet selectedItemsActions(selectedNodesList: TreeNode[])}
      <Button
        variant="outline"
        size="sm"
        onclick={() => handleRemove(selectedNodesList)}
        disabled={selectedNodesList.length === 0}
      >
        <Trash2 class="h-4 w-4 mr-1" />
        Remove
      </Button>
    {/snippet}
  </DirectoryTree>
</div>
