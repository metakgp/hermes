<script lang="ts">
  import {
    Folder,
    FolderOpen,
    File,
    Search,
    ChevronRight,
    ChevronDown,
    ArrowUpDown,
    ArrowUp,
    ArrowDown,
  } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Checkbox } from "$lib/components/ui/checkbox/index.js";
  import { cn } from "$lib/utils.js";
  import type { Snippet } from "svelte";

  export interface TreeNode {
    id: string;
    name: string;
    hash: string;
    children?: TreeNode[];
    path: string;
    size?: number;
    modified?: Date;
  }

  interface DirectoryTreeProps {
    data: TreeNode[];
    selectable?: boolean;
    searchable?: boolean;
    onNodeClick?: (node: TreeNode) => void;
    selectedItemsActions?: Snippet<TreeNode[]>;
  }

  let {
    data,
    selectable = false,
    searchable = true,
    selectedItemsActions = undefined,
  }: DirectoryTreeProps = $props();

  type SortField = "name" | "size" | "modified";
  type SortDirection = "asc" | "desc";

  // State management using Svelte 5 syntax
  let searchTerm = $state<string>("");
  let expandedNodes = $state<Set<string>>(new Set());
  let selectedNodes = $state<Set<string>>(new Set());
  let sortField = $state<SortField>("name");
  let sortDirection = $state<SortDirection>("asc");

  let totalFilteredFilesAndFolders = $derived.by(() => {
    let count = 0;
    countFilesAndFolders(filteredData).forEach((c) => {
      count += c;
    });
    return count;
  });

  // function to count files and folders in the tree returns a (number, number)
  function countFilesAndFolders(nodes: Array<TreeNode>): [number, number] {
    let fileCount = 0;
    let folderCount = 0;
    for (const node of nodes) {
      if (!node.children) {
        fileCount++;
      } else {
        folderCount++;
        const [childFiles, childFolders] = countFilesAndFolders(node.children);
        fileCount += childFiles;
        folderCount += childFolders;
      }
    }
    return [fileCount, folderCount];
  }

  // Derived state for filtered and flattened tree
  let filteredData: TreeNode[] = $derived.by(() => {
    if (!searchTerm.trim()) return data;
    return filterTree(data, searchTerm.toLowerCase());
  });

  let flattenedExpandedNodes: Array<TreeNode & { level: number }> = $derived.by(
    () => {
      const flattened = flattenTree(filteredData, expandedNodes);
      return sortNodes(flattened, sortField, sortDirection);
    },
  );
  // Get selected nodes for actions
  let selectedNodesList: TreeNode[] = $derived.by(() => {
    return [...selectedNodes]
      .map((id) => findNodeById(data, id))
      .filter((node) => node !== null);
  });

  function sortNodes(
    nodes: Array<TreeNode & { level: number }>,
    field: SortField,
    direction: SortDirection,
  ): Array<TreeNode & { level: number }> {
    // Group nodes by their parent and level
    const nodesByParent = new Map();

    for (const node of nodes) {
      const parentPath = node.path.split("/").slice(0, -1).join("/") || "root";
      const key = `${parentPath}-${node.level}`;

      if (!nodesByParent.has(key)) {
        nodesByParent.set(key, []);
      }
      nodesByParent.get(key).push(node);
    }

    // Sort each group separately
    for (const [key, group] of nodesByParent) {
      group.sort((a: TreeNode, b: TreeNode) => {
        // Folders first, then files
        const aIsFolder = !!a.children;
        const bIsFolder = !!b.children;

        if (aIsFolder && !bIsFolder) return -1;
        if (!aIsFolder && bIsFolder) return 1;

        let comparison = 0;

        switch (field) {
          case "name":
            comparison = a.name.localeCompare(b.name);
            break;
          case "size":
            comparison = (a.size || 0) - (b.size || 0);
            break;
          case "modified":
            comparison =
              (a.modified?.getTime() || 0) - (b.modified?.getTime() || 0);
            break;
        }

        return direction === "asc" ? comparison : -comparison;
      });
    }

    // Rebuild the flat array maintaining tree order
    const result: Array<TreeNode & { level: number }> = [];
    const processed = new Set();

    function addNodeAndChildren(node: TreeNode & { level: number }) {
      if (processed.has(node.id)) return;

      result.push(node);
      processed.add(node.id);

      // Find and add immediate children in sorted order
      const childKey = `${node.path}-${node.level + 1}`;
      const children = nodesByParent.get(childKey) || [];

      for (const child of children) {
        if (
          child.path.startsWith(node.path + "/") &&
          child.path.split("/").length === node.path.split("/").length + 1
        ) {
          addNodeAndChildren(child);
        }
      }
    }

    // Start with root level nodes
    const rootKey = "root-0";
    const rootNodes = nodesByParent.get(rootKey) || [];

    for (const rootNode of rootNodes) {
      addNodeAndChildren(rootNode);
    }

    return result;
  }
  function filterTree(nodes: TreeNode[], search: string): TreeNode[] {
    return nodes.reduce((acc, node) => {
      const matchesSearch =
        node.name.toLowerCase().includes(search) ||
        node.path.toLowerCase().includes(search);

      if (node.children) {
        const filteredChildren = filterTree(
          node.children as TreeNode[],
          search,
        );
        if (matchesSearch || filteredChildren.length > 0) {
          acc.push({
            ...node,
            children: filteredChildren,
          } as TreeNode);
        }
      } else if (matchesSearch) {
        acc.push(node);
      }

      return acc;
    }, [] as TreeNode[]);
  }

  function handleSort(field: SortField) {
    if (sortField === field) {
      sortDirection = sortDirection === "asc" ? "desc" : "asc";
    } else {
      sortField = field;
      sortDirection = "asc";
    }
  }


  function flattenTree(
    nodes: TreeNode[],
    expanded: Set<string>,
    level: number = 0,
  ): Array<TreeNode & { level: number }> {
    const result: Array<TreeNode & { level: number }> = [];

    for (const node of nodes) {
      result.push({ ...node, level });

      if (node.children && (expanded.has(node.id) || searchTerm.trim())) {
        result.push(
          ...flattenTree(node.children as TreeNode[], expanded, level + 1),
        );
      }
    }

    return result;
  }

  // Find node by ID in the tree
  function findNodeById(nodes: TreeNode[], id: string): TreeNode | null {
    for (const node of nodes) {
      if (node.id === id) {
        return node;
      }
      if (node.children) {
        const found = findNodeById(node.children, id);
        if (found) return found;
      }
    }
    return null;
  }

  // Get all descendant node IDs
  function getDescendantIds(node: TreeNode): Set<string> {
    const descendants = new Set<string>();
    descendants.add(node.id);

    if (node.children) {
      for (const child of node.children) {
        const childDescendants = getDescendantIds(child);
        childDescendants.forEach((id) => descendants.add(id));
      }
    }

    return descendants;
  }
  // Get all ancestor node IDs using findParent
  function getAncestorIds(node: TreeNode): Set<string> {
    const ancestors = new Set<string>();
    let currentNode: TreeNode | null = node;
    while (currentNode) {
      ancestors.add(currentNode.id);
      currentNode = findParent(data, currentNode);
    }
    return ancestors;
  }

  function findParent(nodes: TreeNode[], child: TreeNode): TreeNode | null {
    for (const node of nodes) {
      if (node.children && node.children.some((c) => c.id === child.id)) {
        return node;
      }
      if (node.children) {
        const parent = findParent(node.children, child);
        if (parent) return parent;
      }
    }
    return null;
  }

  function isPartiallySelected(node: TreeNode): boolean {
    if (!node.children) {
      return false;
    }
    if (areAllChildrenSelected(node)) {
      return false; // All children are selected
    }
    const descendantIds = getDescendantIds(node);
    for (const descendantId of descendantIds) {
      if (selectedNodes.has(descendantId)) {
        return true; // At least one child is selected
      }
    }
    return false; // No children are selected
  }

  // Check if all children are selected
  function areAllChildrenSelected(node: TreeNode): boolean {
    if (!node.children) {
      return selectedNodes.has(node.id);
    }
    for (const child of node.children) {
      if (!selectedNodes.has(child.id)) {
        selectedNodes.delete(node.id);
        return false;
      }
      if (!areAllChildrenSelected(child)) {
        selectedNodes.delete(node.id);
        return false;
      }
    }
    selectedNodes.add(node.id);
    return true;
  }

  function toggleExpanded(nodeId: string) {
    if (expandedNodes.has(nodeId)) {
      expandedNodes.delete(nodeId);
    } else {
      expandedNodes.add(nodeId);
    }
    expandedNodes = new Set(expandedNodes);
  }

  function toggleSelected(nodeId: string) {
    const node = findNodeById(data, nodeId);
    if (!node) return;

    const descendantIds = getDescendantIds(node);

    if (selectedNodes.has(nodeId) || isPartiallySelected(node)) {
      // Deselect node and all descendants
      selectedNodes.delete(nodeId);
      descendantIds.forEach((id) => selectedNodes.delete(id));
    } else {
      // Select node and all descendants
      selectedNodes.add(nodeId);
      descendantIds.forEach((id) => selectedNodes.add(id));
    }

    const ancestorIds = getAncestorIds(node);
    // Ensure all ancestors are selected if all descendants are selected
    ancestorIds.forEach((ancestorId) => {
      if (areAllChildrenSelected(findNodeById(data, ancestorId) || node)) {
        selectedNodes.add(ancestorId);
      } else {
        selectedNodes.delete(ancestorId);
      }
    });

    selectedNodes = new Set(selectedNodes);
  }

  function handleNodeClick(node: TreeNode) {
    if (node.children) {
      toggleExpanded(node.id);
    } else {
      toggleSelected(node.id);
    }
  }

  function formatFileSize(bytes?: number): string {
    if (!bytes) return "";
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${sizes[i]}`;
  }
  function clearSelection() {
    selectedNodes = new Set();
  }

  function selectAllFiltered() {
    const allNodeIds = new Set<string>();
    function collectNodeIds(nodes: TreeNode[]) {
      for (const node of nodes) {
        allNodeIds.add(node.id);
        if (node.children) {
          collectNodeIds(node.children);
        }
      }
    }
    collectNodeIds(filteredData);
    selectedNodes = allNodeIds;
  }

  function areAllFilteredSelected(): boolean {
    const allNodeIds = new Set<string>();
    function collectNodeIds(nodes: TreeNode[]) {
      for (const node of nodes) {
        allNodeIds.add(node.id);
        if (node.children) {
          collectNodeIds(node.children);
        }
      }
    }
    collectNodeIds(filteredData);
    return selectedNodes.size === allNodeIds.size;
  }
</script>
{#snippet sortIcon(field: SortField)}
{#if sortField === field}
  {#if sortDirection === "asc"}
    <ArrowUp class="h-3 w-3" />
  {:else}
    <ArrowDown class="h-3 w-3" />
  {/if}
{:else}
  <ArrowUpDown class="h-3 w-3" />
{/if}
{/snippet}

<div class="max-w-full m-4 relative">
  {#if searchable}
    <div class="flex items-center py-4">
      <div class="relative max-w-sm">
        <Search class="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
        <Input
          placeholder="Search files and folders..."
          bind:value={searchTerm}
          class="pl-8"
        />
      </div>
      <div class="ml-auto text-sm text-muted-foreground">
        {totalFilteredFilesAndFolders} item{totalFilteredFilesAndFolders === 1
          ? ""
          : "s"} found
      </div>
    </div>
  {/if}
  {#if selectable}
    <div class="flex items-center gap-2 p-4 bg-muted/50 rounded-md border mb-4">
      <div class="flex items-center gap-2 flex-1">
        <span class="text-sm font-medium">
          {selectedNodes.size} item{selectedNodes.size === 1 ? "" : "s"} selected
        </span>

        <div class="flex items-center gap-1 ml-4">
          {@render selectedItemsActions?.(selectedNodesList)}
        </div>
      </div>

      <Button
        variant="ghost"
        size="sm"
        onclick={clearSelection}
        disabled={selectedNodes.size === 0}
      >
        Clear Selection
      </Button>
    </div>
  {/if}

  <div class="rounded-md border overflow-auto">
    <!-- Header Row -->
    <div class="border-b bg-muted/30">
      <div class="flex items-center gap-2 p-2 font-medium text-sm">
        {#if selectable}
          <div class="w-6 mr-1">
            <Checkbox
              class="mr-1"
              id="select-all-checkbox"
              bind:checked={areAllFilteredSelected,
              (v) => {
                if (v) {
                  selectAllFiltered();
                } else {
                  clearSelection();
                }
              }}
            />
          </div>
        {/if}

        <div class="w-6"></div>
        <!-- Space for expand/collapse button -->
        <div class="w-4"></div>
        <!-- Space for folder/file icon -->

        <!-- Name Column -->
        <Button
          variant="ghost"
          size="sm"
          class="flex-1 justify-start h-8 px-2 hover:bg-muted"
          onclick={() => handleSort("name")}
        >
          <span class="mr-2">Name</span>
          {@render sortIcon("name")}
        </Button>

        <!-- Size Column -->
        <Button
          variant="ghost"
          size="sm"
          class="w-20 justify-center h-8 px-2 hover:bg-muted"
          onclick={() => handleSort("size")}
        >
          <span class="mr-2">Size</span>
          {@render sortIcon("size")}
        </Button>

        <!-- Modified Column -->
        <Button
          variant="ghost"
          size="sm"
          class="w-32 justify-center h-8 px-2 hover:bg-muted"
          onclick={() => handleSort("modified")}
        >
          <span class="mr-2">Modified</span>
          {@render sortIcon("modified")}
        </Button>
      </div>
    </div>

    <!-- Content Rows -->
    <div class="">
      {#each flattenedExpandedNodes as node (node.id)}
        {@const isExpanded = expandedNodes.has(node.id)}
        {@const isSelected =
          selectedNodes.has(node.id) || areAllChildrenSelected(node)}
        {@const isIndeterminate = isPartiallySelected(node)}
        {@const hasChildren = node.children && node.children.length > 0}

        <div
          class={cn(
            "flex items-center gap-2 p-2  hover:bg-accent cursor-pointer mb-2",
            (isSelected || isIndeterminate) && "bg-accent",
          )}
          style="padding-left: {node.level * 1.5 + 0.5}rem"
          onclick={() => handleNodeClick(node)}
        >
          {#if selectable}
            <Checkbox
              bind:checked={() => isSelected, (_) => {}}
              bind:indeterminate={() => isIndeterminate, (_) => {}}
              onclick={(e) => {
                toggleSelected(node.id);
                e.stopPropagation();
              }}
              class="mr-1"
              id={`checkbox-${node.id}`}
            />
          {/if}

          {#if node.children}
            <Button
              variant="ghost"
              size="sm"
              class="h-6 w-6 p-0"
              onclick={(e) => {
                e.stopPropagation();
                toggleExpanded(node.id);
              }}
              disabled={!hasChildren}
            >
              {#if hasChildren}
                {#if isExpanded}
                  <ChevronDown class="h-4 w-4" />
                {:else}
                  <ChevronRight class="h-4 w-4" />
                {/if}
              {:else}
                <div class="h-4 w-4"></div>
              {/if}
            </Button>

            {#if isExpanded}
              <FolderOpen class="h-4 w-4 text-blue-600" />
            {:else}
              <Folder class="h-4 w-4 text-blue-600" />
            {/if}
          {:else}
            <div class="h-6 w-6"></div>
            <File class="h-4 w-4 text-gray-600" />
          {/if}

          <span class="flex-1 truncate">{node.name}</span>

          <div class="w-20 text-center">
            {#if !node.children && node.size}
              <span class="text-xs text-muted-foreground">
                {formatFileSize(node.size)}
              </span>
            {/if}
          </div>

          <div class="w-32 text-center">
            {#if node.modified}
              <span class="text-xs text-muted-foreground">
                {node.modified.toLocaleDateString()}
              </span>
            {/if}
          </div>
        </div>
      {:else}
        <div class="p-8 text-center text-muted-foreground">
          {searchTerm ? "No items match your search." : "No items to display."}
        </div>
      {/each}
    </div>
  </div>
</div>
