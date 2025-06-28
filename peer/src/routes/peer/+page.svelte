<script lang="ts">
  import DirectoryTree, {
    type TreeNode,
  } from "$lib/components/custom/directorytree.svelte";
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import { invoke } from "@tauri-apps/api/core";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Download } from "@lucide/svelte";

  let treeData: TreeNode[] = $state([]);
  let nodeid: string;
  onMount(() => {
    // This will run when the component is mounted
    nodeid = page.url.searchParams.get("nodeid") || "";
    if (nodeid) {
      console.log("Node ID from URL:", nodeid);
    } else {
      console.error("No Node ID provided in URL");
    }
    invoke<TreeNode[]>("get_remote_files", { nodeId: nodeid })
      .then((res) => {
        treeData = res as TreeNode[];
        console.log("Data loaded:", treeData);
      })
      .catch((e) => console.error("Error loading data:", e));
  });
  function handleDownload(selectedNodesList: TreeNode[]) {
    console.log("Downloading selected nodes:", selectedNodesList);
    // TODO implement download logic here
  }
</script>

<div class="container mx-auto py-8">
  <h1 class="text-2xl font-bold mb-6">Directory Tree</h1>

  <DirectoryTree data={treeData} selectable={true} searchable={true}>
    {#snippet selectedItemsActions(selectedNodesList: TreeNode[])}
      <Button
        variant="outline"
        size="sm"
        onclick={() => handleDownload(selectedNodesList)}
        disabled={selectedNodesList.length === 0}
      >
        <Download class="h-4 w-4 mr-1" />
        Download
      </Button>
    {/snippet}
  </DirectoryTree>
</div>
