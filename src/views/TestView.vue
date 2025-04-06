<template>
  <button @click="click">设置</button>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

let click = async () => {
  console.log("click");

  // const appWebview = getCurrentWebviewWindow();
  // appWebview.emitTo("main", "open-settings-window");

  invoke("open_settings_window")
    .then((res) => {
      console.log("res", res);
    })
    .catch((err) => {
      console.log("err", err);
    });
};
</script>
