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
