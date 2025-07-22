<script lang="ts">
    import { Button } from "$lib/components/ui/button/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import { open } from "@tauri-apps/plugin-dialog";
    import { invoke } from "@tauri-apps/api/core";
    import { onDestroy, onMount } from "svelte";
    import { Trash2 } from "@lucide/svelte";
    import DirectoryTree, {
        type TreeNode,
    } from "$lib/components/custom/directorytree.svelte";
    import { toast } from "svelte-sonner";
    import type { UnlistenFn } from "@tauri-apps/api/event";
    import { getCurrentWebview } from "@tauri-apps/api/webview";
    let rootNode: TreeNode[] = $state([]);
    const unlisteners: Array<UnlistenFn> = [];
    onMount(() => {
        // This will run when the component is mounted
        loadFiles();
        getCurrentWebview()
            .onDragDropEvent(async (event) => {
                if (event.payload.type === "drop") {
                    const paths = event.payload.paths;
                    await addPaths(paths);
                }
            })
            .then((unlisten) => {
                unlisteners.push(unlisten);
            });
    });
    onDestroy(() => {
        unlisteners.forEach((unlisten) => unlisten());
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
        await addPaths(folderPaths);
    }
    async function addPaths(folderPaths: string[]) {
        const importPromises = folderPaths.map(async (folderPath) => {
            try {
                await invoke("add_path", { path: folderPath });
                return { folderPath };
            } catch (error) {
                throw { folderPath, error };
            }
        });

        const allPromises = Promise.all(importPromises);
        toast.promise(allPromises, {
            loading: `Adding ${folderPaths.length} folder(s)...`,
            success: (results) => {
                const len = results.length;
                return `Successfully added ${len} folder(s).`;
            },
            error: (e: { folderPath: string; error: any }) => {
                return `While adding folder: ${e.folderPath} encountered an error: ${e.error}`;
            },
        });
        await allPromises;
        loadFiles();
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
    async function handleRemove(selectedNodesList: TreeNode[]) {
        if (!selectedNodesList.length) {
            toast.error("No files selected.");
            return;
        }

        const nodeHashes = selectedNodesList.map((node) => node.hash);
        await toast.promise(
            invoke("remove_files", { nodeHashes }).then(() => loadFiles()),
            {
                loading: "Removing...",
                success: "Successfully removed files!",
                error: (e) => `Error removing: ${e}`,
            },
        );
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
