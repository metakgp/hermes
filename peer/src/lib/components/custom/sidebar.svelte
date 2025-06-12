<script lang="ts">
  import Upload from "@lucide/svelte/icons/upload";
  import Users from "@lucide/svelte/icons/users";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import CircleUser from "@lucide/svelte/icons/circle-user";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";

  const nav = [
    {
      name: "Home",
      icon: CircleUser,
      route: "/",
      tooltip: "Your account settings",
    },
    {
      name: "Upload",
      icon: Upload,
      route: "/upload",
      tooltip: "Upload files to your account",
    },
    {
      name: "Peers",
      icon: Users,
      route: "/peers",
      tooltip: "View online peers",
    },
    {
      name: "Messages",
      icon: MessageSquare,
      route: "",
      tooltip: "Coming Soon",
      disabled: true,
    },
  ];
</script>
<div class="flex">

<Sidebar.Root>
  <Sidebar.Content>
    <Sidebar.Group>
      <Sidebar.GroupLabel>Application</Sidebar.GroupLabel>
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each nav as item (item.name)}
            <Sidebar.MenuItem>
              <Sidebar.MenuButton>
                {#snippet child({ props })}
                  <a href={item.route} {...props}>
                    <item.icon />
                    <span>{item.name}</span>
                  </a>
                {/snippet}
              </Sidebar.MenuButton>
            </Sidebar.MenuItem>
          {/each}
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>
  </Sidebar.Content>
</Sidebar.Root> 
</div>
<!--     <div class="px-3 py-2"> -->
<!--       <h2 class="mb-2 px-4 text-lg font-semibold tracking-tight">Folders</h2> -->
<!--       <ScrollArea class="h-[120px] px-1"> -->
<!--         <div class="space-y-1 p-2"> -->
<!--           {#if $foldersStore.length === 0} -->
<!--             <div class="text-gray-400 px-2">No folders</div> -->
<!--           {:else} -->
<!--             {#each $foldersStore as folder} -->
<!--               <Button variant="ghost" class="w-full justify-start font-normal"> -->
<!--                 <Folder class="mr-2 size-4" /> {folder} -->
<!--               </Button> -->
<!--             {/each} -->
<!--           {/if} -->
<!--         </div> -->
<!--       </ScrollArea> -->
<!--     </div> -->
<!---->
<!--     <div class="px-3 py-2"> -->
<!--       <h2 class="mb-2 px-4 text-lg font-semibold tracking-tight">Files</h2> -->
<!--       <ScrollArea class="h-[120px] px-1"> -->
<!--         <div class="space-y-1 p-2"> -->
<!--           {#if $filesStore.length === 0} -->
<!--             <div class="text-gray-400 px-2">No files</div> -->
<!--           {:else} -->
<!--             {#each $filesStore as file} -->
<!--               <Button variant="ghost" class="w-full justify-start font-normal"> -->
<!--                 <FileIcon class="mr-2 size-4" /> {file.name} -->
<!--               </Button> -->
<!--             {/each} -->
<!--           {/if} -->
<!--         </div> -->
<!--       </ScrollArea> -->
<!--     </div> -->
<!--   </div> -->
<!-- </div> -->
<!---->
