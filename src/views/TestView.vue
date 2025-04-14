<template>
  <button @click="start">开始</button>
  <button @click="stop">停止</button>
  <textarea>{{ text }}</textarea>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

import { ref } from "vue";
const text = ref("");

let unlisten: UnlistenFn;

const start = async () => {
  console.log("start");

  if (unlisten) {
    unlisten();
  }

  unlisten = await listen("recv_text", (event) => {
    console.log("recv_text", event.payload);

    text.value += event.payload;
    text.value += "\n";
  });

  invoke("audio_start")
    .then((res) => {
      console.log("res", res);
    })
    .catch((err) => {
      console.log("err", err);
    });
};

const stop = async () => {
  console.log("start");

  if (unlisten) {
    unlisten();
  }

  invoke("audio_stop")
    .then((res) => {
      console.log("res", res);
    })
    .catch((err) => {
      console.log("err", err);
    });
};
</script>
