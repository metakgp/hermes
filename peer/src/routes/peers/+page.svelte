<script lang="ts">
  import { buttonVariants } from "$lib/components/ui/button/index.js";
  import * as Table from "$lib/components/ui/table/index.js";
  import Menu from "@lucide/svelte/icons/menu";
  import File from "@lucide/svelte/icons/file";
  import Handshake from "@lucide/svelte/icons/handshake";
  import MessageSquareText from "@lucide/svelte/icons/message-square-text";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
    import { get } from "svelte/store";

  interface Peer {
    username: string;
    node_id: string;
  }
  let peers: Peer[] = $state([ ]);
  $effect(() => {
    console.log("PeersTab component mounted");
    invoke("get_peers")
      .then((data) => {
        console.log("Fetched peers:", data);
        peers = data as Peer[];
      })
      .catch((error) => {
        console.error("Error fetching peers:", error);
      });
  });
  
  listen<Peer>('peer::added', () => {
    getPeers();
  });
  listen<[Peer, Peer]>('peer::username_changed', () => {
    getPeers();
  });
  listen<Peer>('peer::left', () => {
    getPeers();
  });
  function getPeers() {
    invoke("get_peers")
      .then((data) => {
        console.log("Fetched peers:", data);
        peers = data as Peer[];
      })
      .catch((error) => {
        console.error("Error fetching peers:", error);
      });
  }
</script>

<div class="space-y-4 m-8 max-w-full">
  <Table.Root>
    <Table.Caption>A list of all online peers</Table.Caption>
    <Table.Header>
      <Table.Row>
        <Table.Head>NickName</Table.Head>
        <Table.Head>NodeId</Table.Head>
        <Table.Head>Actions</Table.Head>
      </Table.Row>
    </Table.Header>
    <Table.Body>
      {#each peers as peer}
        <Table.Row>
          <Table.Cell>{peer.username}</Table.Cell>
          <Table.Cell class="font-medium">{peer.node_id}</Table.Cell>
          <Table.Cell class="text-right">
            <DropdownMenu.Root>
              <DropdownMenu.Trigger
                class={buttonVariants({ variant: "outline" })}
              >
                <Menu />
              </DropdownMenu.Trigger>
              <DropdownMenu.Content>
                <DropdownMenu.Group>
                  <DropdownMenu.Item
                    ><MessageSquareText /> Shared Files</DropdownMenu.Item
                  >
                  <DropdownMenu.Item
                    ><Handshake /> Add as Friend</DropdownMenu.Item
                  >
                  <DropdownMenu.Item
                    ><MessageSquareText /> Message</DropdownMenu.Item
                  >
                </DropdownMenu.Group>
              </DropdownMenu.Content>
            </DropdownMenu.Root>
          </Table.Cell>
        </Table.Row>
      {/each}
    </Table.Body>
  </Table.Root>
</div>
