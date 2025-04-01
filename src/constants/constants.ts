export const audioConfig = {
    SAMPLE_RATE: 16000,
    CHANNELS: 1,
    FRAME_DURATION: 60,
    FRAME_SIZE: 0
}
audioConfig.FRAME_SIZE = Math.floor(
    audioConfig.SAMPLE_RATE * (audioConfig.FRAME_DURATION / 1000));

export const eventType = {
    SCHEDULE_EVENT: "schedule_event",
    AUDIO_INPUT_READY_EVENT: "audio_input_ready_event",
    AUDIO_OUTPUT_READY_EVENT: "audio_output_ready_event"
}

export const deviceState = {
    IDLE: "idle",
    CONNECTING: "connecting",
    LISTENING: "listening",
    SPEAKING: "speaking"
}

export const abortReason = {
    NONE: "none",
    WAKE_WORD_DETECTED: "wake_word_detected"
}

export const websocketURL = "ws://10.13.19.91:8080/";

export const wsEvent = {
    test: "test",
    incomingAudio: "incomingAudio",
    incomingJson: "incomingJson",
    audioChannelOpened: "audioChannelOpened",
    audioChannelClosed: "audioChannelClosed",
    networkError: "networkError",
} as const;

export type wsEvenType = typeof wsEvent;

export const protocolHeader = {
    AUDIO_FRAME: 0x01,
    CONTROL_MESSAGE: 0x02
}