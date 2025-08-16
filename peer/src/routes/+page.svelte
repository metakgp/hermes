<script lang="ts">
  import { Input } from "$lib/components/ui/input/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Card from "$lib/components/ui/card/index.js";
  import { invoke } from "@tauri-apps/api/core";
  import { toast } from "svelte-sonner";
  let username = $state("");
  let isUsernameValid = $state(false);
  invoke("get_username")
    .then((data) => {
      username = data as string;
      if (username.length >= 3 && username.length <= 20) {
        isUsernameValid = true;
      } else {
        isUsernameValid = false;
      }
    })
    .catch((error) => {
      toast.error("Error fetching username: " + error);
    });

  $effect(() => {
    if (username.length < 3 || username.length > 20) {
      isUsernameValid = false;
    } else {
      isUsernameValid = true;
    }
  });
  function submitUsername() {
    if (isUsernameValid) {
      invoke("set_username", { username })
        .then(() => {
          toast.success("Username set successfully!");
        })
        .catch((error) => {
          toast.error("Error setting username: " + error);
        });
    } else {
      toast.error("Username must be between 3 and 20 characters.");
    }
  }
</script>

<div class="flex h-full justify-center items-center">
  <Card.Root class="w-full max-w-sm">
    <Card.Header>
      <Card.Title>Username</Card.Title>
      <Card.Description>
        Enter your username. Between 3 and 20 characters.
      </Card.Description>
    </Card.Header>
    <Card.Content class="flex flex-col gap-2 mt-2">
      <Input
        type="text"
        placeholder="Username"
        bind:value={username}
        aria-invalid={!isUsernameValid}
      />
      <Button class="w-full mt-2 mb-2" disabled={!isUsernameValid} onclick={submitUsername}>
        Submit
      </Button>
    </Card.Content>
  </Card.Root>
</div>
