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
  type File = {
    name: string;
    size: string;
    hash: string;
  };
  let folders: string[] = $state([]);
  let files: File[] = $state([]);
  loadFiles();
  loadFolders();

  async function loadFolders() {
    await invoke("get_paths", { }).then((res) => {
      folders = res as string[];
      console.log(folders);
    }).catch(e => console.error("What ", e));
  }
  
  async function pickFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select a folder"
    });
    if (typeof selected === "string") {
      const folderPath = selected;
      await invoke("add_path", {path: folderPath}).then((res) => {
        folders = res as string[];
        loadFiles();
      })
      .catch(e => console.error("What ", e));
    }
  }
  async function clearAll() {
    await invoke("clear_files").then((res) => {
        files = [];
        folders = [];
    })    
  } 
  async function loadFiles() {
      await invoke("get_files", { }).then((res) => {
        files = res; 
        console.log(files);
      }).catch(e => console.error("What ", e));
  }
</script>

<h1 class="text-2xl font-bold m-8 mb-4">Shared Files</h1>
  <!-- Folder Picker Input -->
<Label class="mb-2 inline-flex">
    Add a folder:
</Label>
<Button onclick={pickFolder} class="">
  Pick a folder
</Button>

<!-- Folders List UI -->
  <h2 class="font-semibold mb-2">Selected Folders</h2>
  {#if folders.length === 0}
    <p class="text-muted-foreground">No folders selected.</p>
  {:else}
    <ul class="list-disc pl-5 space-y-1">
      {#each folders as folder}
        <li class="truncate">{folder}</li>
      {/each}
    </ul>
  {/if}

<!-- Clear All Button -->
<div class="flex justify-end m-8 mb-4">
  <Button class="" onclick={clearAll}>
    Clear All
  </Button>
</div>



<div class="space-y-4 m-8 max-w-full">
  <Table.Root>
    <Table.Caption>A list of Added files</Table.Caption>
    <Table.Header>
      <Table.Row>
        <Table.Head class="w-[100px]">File Hash</Table.Head>
        <Table.Head>Name</Table.Head>
        <Table.Head class="text-right">Size</Table.Head>
        <Table.Head class="text-right">Actions</Table.Head>
      </Table.Row>
    </Table.Header>
    <Table.Body>
      {#each files as file}
        <Table.Row>
          <Table.Cell>{file.hash}</Table.Cell>
          <Table.Cell class="font-medium">{file.name}</Table.Cell>
          <Table.Cell class="text-right">{file.size}</Table.Cell>
          <Table.Cell class="text-right">
            <DropdownMenu.Root>
              <DropdownMenu.Trigger
                class={buttonVariants({ variant: "outline" })}
              >
                <Menu />
              </DropdownMenu.Trigger>
              <DropdownMenu.Content>
                <DropdownMenu.Group>
                  <DropdownMenu.Item><Share2 /> Share</DropdownMenu.Item>
                  <DropdownMenu.Item><Trash2 /> Delete</DropdownMenu.Item>
                </DropdownMenu.Group>
              </DropdownMenu.Content>
            </DropdownMenu.Root>
          </Table.Cell>
        </Table.Row>
      {/each}
    </Table.Body>
  </Table.Root>
</div>
