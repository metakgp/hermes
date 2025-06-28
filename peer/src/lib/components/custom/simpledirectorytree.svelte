<script lang="ts">
  import { Folder, FolderOpen, File, ChevronRight, ChevronDown, Search, X } from '@lucide/svelte';
  import { cn } from '$lib/utils';
  
  interface TreeNode {
    name: string;
    type: 'file' | 'folder';
    children?: TreeNode[];
    path?: string;
  }
  
  interface Props {
    data: TreeNode[];
    onSelect?: (node: TreeNode) => void;
    searchable?: boolean;
  }
  
  let { data, onSelect = () => {}, searchable = true }: Props = $props();
  
  let expandedFolders = $state(new Set<string>());
  let searchTerm = $state('');
  
  function toggleFolder(path: string) {
    if (expandedFolders.has(path)) {
      expandedFolders.delete(path);
    } else {
      expandedFolders.add(path);
    }
    expandedFolders = new Set(expandedFolders); // Trigger reactivity
  }
  
  function getNodePath(parentPath: string, nodeName: string): string {
    return parentPath ? `${parentPath}/${nodeName}` : nodeName;
  }
  
  function filterNodes(nodes: TreeNode[], term: string): TreeNode[] {
    if (!term) return nodes;
    
    return nodes.reduce((acc, node) => {
      if (node.name.toLowerCase().includes(term.toLowerCase())) {
        acc.push(node);
      } else if (node.children) {
        const filteredChildren = filterNodes(node.children, term);
        if (filteredChildren.length > 0) {
          acc.push({ ...node, children: filteredChildren });
        }
      }
      return acc;
    }, [] as TreeNode[]);
  }
  
  function clearSearch() {
    searchTerm = '';
  }
  
  // Use $derived instead of $: reactive statement
  const filteredData = $derived(filterNodes(data, searchTerm));
</script>

<!-- Tree Node Snippet -->
{#snippet treeNode(node: TreeNode, level: number = 0, parentPath: string = '')}
  {@const nodePath = getNodePath(parentPath, node.name)}
  {@const isExpanded = expandedFolders.has(nodePath)}
  {@const hasChildren = node.children && node.children.length > 0}
  
  <div class="select-none">
    <!-- Node Content -->
    <div 
      class={cn(
        "flex items-center gap-2 py-1.5 px-2 rounded-sm cursor-pointer transition-colors",
        "hover:bg-accent hover:text-accent-foreground",
        "focus-visible:bg-accent focus-visible:text-accent-foreground focus-visible:outline-none",
        "text-sm font-medium"
      )}
      style="padding-left: {level * 1.5 + 0.5}rem"
      onclick={() => {
        if (node.type === 'folder' && hasChildren) {
          toggleFolder(nodePath);
        }
        onSelect(node);
      }}
      onkeydown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          if (node.type === 'folder' && hasChildren) {
            toggleFolder(nodePath);
          }
          onSelect(node);
        }
      }}
      role="button"
      tabindex="0"
      aria-expanded={node.type === 'folder' ? isExpanded : undefined}
    >
      <!-- Expand/Collapse Icon -->
      {#if node.type === 'folder' && hasChildren}
        <div class="w-4 h-4 flex items-center justify-center">
          {#if isExpanded}
            <ChevronDown size={14} class="text-muted-foreground" />
          {:else}
            <ChevronRight size={14} class="text-muted-foreground" />
          {/if}
        </div>
      {:else}
        <div class="w-4"></div>
      {/if}
      
      <!-- File/Folder Icon -->
      <div class="w-4 h-4 flex items-center justify-center">
        {#if node.type === 'folder'}
          {#if isExpanded}
            <FolderOpen size={16} class="text-primary" />
          {:else}
            <Folder size={16} class="text-primary" />
          {/if}
        {:else}
          <File size={16} class="text-muted-foreground" />
        {/if}
      </div>
      
      <!-- Node Name -->
      <span class="truncate flex-1 text-foreground">
        {node.name}
      </span>
    </div>
    
    <!-- Children -->
    {#if node.type === 'folder' && hasChildren && isExpanded}
      <div class="border-l border-border ml-4 pl-2">
        {#each node.children as child}
          {@render treeNode(child, level + 1, nodePath)}
        {/each}
      </div>
    {/if}
  </div>
{/snippet}

<!-- Main Component -->
<div class="w-full max-w-md space-y-4">
  {#if searchable}
    <!-- Search Input -->
    <div class="relative">
      <Search size={16} class="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground" />
      <input
        type="text"
        placeholder="Search files and folders..."
        bind:value={searchTerm}
        class={cn(
          "flex h-10 w-full rounded-md border border-input bg-background px-10 py-2 text-sm",
          "ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium",
          "placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2",
          "focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
        )}
      />
      {#if searchTerm}
        <button
          onclick={clearSearch}
          class="absolute right-3 top-1/2 transform -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
          aria-label="Clear search"
        >
          <X size={16} />
        </button>
      {/if}
    </div>
  {/if}
  
  <!-- Directory Tree -->
  <div class="rounded-lg border border-border bg-card text-card-foreground shadow-sm">
    <div class="p-4">
      <div class="space-y-1">
        {#if filteredData.length === 0}
          <div class="text-center py-8 text-muted-foreground">
            {#if searchTerm}
              <p class="text-sm">No results found for "{searchTerm}"</p>
            {:else}
              <p class="text-sm">No files or folders to display</p>
            {/if}
          </div>
        {:else}
          {#each filteredData as node}
            {@render treeNode(node)}
          {/each}
        {/if}
      </div>
    </div>
  </div>
</div>

