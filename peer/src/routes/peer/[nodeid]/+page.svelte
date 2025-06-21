<script lang="ts">
  import DirectoryTree, {
    type TreeNode,
  } from "$lib/components/custom/directorytree.svelte";
    import { path } from "@tauri-apps/api";

  // Sample data structure
  const treeData: TreeNode[] = [
    {
      id: "1",
      name: "Documents",
      type: "folder",
      path: "/Documents",
      modified: new Date("2024-01-15"),
      children: [
        {
          id: "1-1",
          name: "Projects",
          type: "folder",
          path: "/Documents/Projects",
          modified: new Date("2024-01-20"),
          children: [
            {
              id: "1-1-1",
              name: "project1.md",
              type: "file",
              path: "/Documents/Projects/project1.md",
              size: 2048,
              modified: new Date("2024-01-21"),
            },
            {
              id: "1-1-2",
              name: "project2.md",
              type: "file",
              path: "/Documents/Projects/project2.md",
              size: 4096,
              modified: new Date("2024-01-22"),
            },
          ],
        },
        {
          id: "1-2",
          name: "readme.txt",
          type: "file",
          path: "/Documents/readme.txt",
          size: 1024,
          modified: new Date("2024-01-10"),
        },
      ],
    },
    {
      id: "2",
      name: "Images",
      type: "folder",
      path: "/Images",
      modified: new Date("2024-01-12"),
      children: [
        {
          id: "2-1",
          name: "vacation.jpg",
          type: "file",
          path: "/Images/vacation.jpg",
          size: 102400,
          modified: new Date("2024-01-13"),
        },
      ],
    },
  ];

  function handleSelect(selectedIds: string[]) {
    console.log("Selected items:", selectedIds);
  }

  function handleNodeClick(node: TreeNode) {
    console.log("Clicked node:", node);
  }

  function generateTreeData(depth: number, width: number): TreeNode[] {
    // for testing generate a very very large tree data each node having width children depth times
    const root_children: TreeNode[] = [];
    for (let i = 0; i < depth; i++) {
      const children: TreeNode[] = [];
      // generate random sizes and date, ensure date is random
      let size = Math.floor(Math.random() * 1000000);
      let date = new Date(Date.now() - Math.floor(Math.random() * 10000000000));
      for (let j = 0; j < width; j++) {
        children.push({
          id: `${i}-${j}`,
          name: `Node ${i}-${j}`,
          path: `/Node-${i}-${j}`,
          size: size,
          modified: date,
          
        });
      }
      root_children.push({
        id: `root-${i}`,
        name: `Root Node ${i}`,
        path: `/Root-Node-${i}`,
        size: Math.floor(Math.random() * 1000000),
        modified: new Date(Date.now() - Math.floor(Math.random() * 10000000000)),
        children,
      });
    }
    root_children.push({
      id: `file-232`,
      name: `Fileadf`,
      path: `/Fileadf`,
    });
    return root_children;
  }
</script>

<div class="container mx-auto py-8">
  <h1 class="text-2xl font-bold mb-6">Directory Tree</h1>

  <DirectoryTree
    data={treeData}
    selectable={true}
    searchable={true}
    onNodeClick={handleNodeClick}
  />
</div>

