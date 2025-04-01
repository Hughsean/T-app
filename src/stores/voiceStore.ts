
import { defineStore } from 'pinia'

export const useVoiceStore = defineStore('voice', {
  state: () => ({
    records: [] as { text: string; result: string }[]
  }),
  actions: {
    addRecord(text: string, result: string) {
      this.records.push({ text, result })
    }
  }
})