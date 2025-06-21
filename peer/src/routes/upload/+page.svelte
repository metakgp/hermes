<script lang="ts">
  import { buttonVariants } from "$lib/components/ui/button/index.js";
  import * as Table from "$lib/components/ui/table/index.js";
  import Menu from "@lucide/svelte/icons/menu";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import Share2 from "@lucide/svelte/icons/share-2";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import { open } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";
  import DirectoryTree, {
    type TreeNode,
  } from "$lib/components/custom/directorytree.svelte";
  type File = {
    name: string;
    size: string;
    hash: string;
  };
  let sharedPath: string[] = $state([]);
  let files: File[] = $state([]);
  let rootNode: TreeNode[] = $state([]);
  loadFiles();
  loadSharedPaths();

  async function loadSharedPaths() {
    await invoke("get_shared_paths", {})
      .then((res) => {
        sharedPath = res as string[];
      })
      .catch((e) => console.error("What ", e));
  }

  async function pickFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select a folder",
    });
    if (typeof selected === "string") {
      const folderPath = selected;
      await invoke("add_path", { path: folderPath })
        .then((res) => {
          loadSharedPaths();
          loadFiles();
        })
        .catch((e) => console.error("What ", e));
    }
  }
  async function clearAll() {
    await invoke("clear_files").then((res) => {
      files = [];
      sharedPath = [];
    });
  }
  async function loadFiles() {
    await invoke<TreeNode>("get_uploaded_files_tree", {})
      .then((res) => {
        rootNode = res;
        console.log(rootNode);
      })
      .catch((e) => console.error("What ", e));
  }
</script>

<h1 class="text-2xl font-bold m-8 mb-4">Shared Files</h1>
<!-- Folder Picker Input -->
<Label class="mb-2 inline-flex">Add a folder:</Label>
<Button onclick={pickFolder} class="">Pick a folder</Button>

<!-- Folders List UI -->
<h2 class="font-semibold mb-2">Selected Folders</h2>
{#if sharedPath.length === 0}
  <p class="text-muted-foreground">No folders selected.</p>
{:else}
  <ul class="list-disc pl-5 space-y-1">
    {#each sharedPath as folder}
      <li class="truncate">{folder}</li>
    {/each}
  </ul>
{/if}

<!-- Clear All Button -->
<div class="flex justify-end m-8 mb-4">
  <Button class="" onclick={clearAll}>Clear All</Button>
</div>

<div class="space-y-4 m-8 max-w-full">
  <DirectoryTree
    data={rootNode}
    selectable={true}
    searchable={true}
  />

</div>
