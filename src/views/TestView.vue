<template>
  <button @click="start">开始</button>
  <button @click="stop">停止</button>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";

const start = async () => {
  console.log("start");

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

  invoke("audio_stop")
    .then((res) => {
      console.log("res", res);
    })
    .catch((err) => {
      console.log("err", err);
    });
};
</script>
