// Audio processing service for Sonica
export interface AudioConfig {
  sampleRate: number;
  channels: number;
  bufferSize: number;
}

export interface RecognitionResult {
  songId: string;
  title: string;
  artist: string;
  album?: string;
  confidence: number;
  duration: number;
  timestamp: number;
}

export interface AudioProcessor {
  startRecording(): Promise<void>;
  stopRecording(): Promise<Float32Array>;
  processAudio(audioData: Float32Array): Promise<RecognitionResult | null>;
  generateFingerprint(audioData: Float32Array): Promise<string>;
}

class SonicaAudioProcessor implements AudioProcessor {
  private mediaRecorder: MediaRecorder | null = null;
  private audioContext: AudioContext | null = null;
  private analyser: AnalyserNode | null = null;
  private microphone: MediaStreamAudioSourceNode | null = null;
  private stream: MediaStream | null = null;
  private audioChunks: Blob[] = [];
  private wasmModule: any = null;

  constructor() {
    this.initializeWasm();
  }

  private async initializeWasm() {
    try {
      // Load WebAssembly module for audio processing
      // This will be implemented when the WASM build is ready
      console.log('WebAssembly module will be loaded here');
    } catch (error) {
      console.error('Failed to initialize WebAssembly:', error);
    }
  }

  async startRecording(): Promise<void> {
    try {
      this.stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          sampleRate: 44100,
          channelCount: 1,
          echoCancellation: true,
          noiseSuppression: true,
          autoGainControl: true,
        },
      });

      this.audioContext = new AudioContext({ sampleRate: 44100 });
      this.analyser = this.audioContext.createAnalyser();
      this.microphone = this.audioContext.createMediaStreamSource(this.stream);

      this.analyser.fftSize = 2048;
      this.analyser.smoothingTimeConstant = 0.8;
      this.microphone.connect(this.analyser);

      this.mediaRecorder = new MediaRecorder(this.stream, {
        mimeType: 'audio/webm;codecs=opus',
      });

      this.audioChunks = [];
      this.mediaRecorder.ondataavailable = (event) => {
        if (event.data.size > 0) {
          this.audioChunks.push(event.data);
        }
      };

      this.mediaRecorder.start(100); // Collect data every 100ms
    } catch (error) {
      console.error('Error starting recording:', error);
      throw new Error('Failed to start audio recording');
    }
  }

  async stopRecording(): Promise<Float32Array> {
    return new Promise((resolve, reject) => {
      if (!this.mediaRecorder || !this.audioContext) {
        reject(new Error('Recording not started'));
        return;
      }

      this.mediaRecorder.onstop = async () => {
        try {
          const audioBlob = new Blob(this.audioChunks, { type: 'audio/webm' });
          const arrayBuffer = await audioBlob.arrayBuffer();
          const audioBuffer = await this.audioContext!.decodeAudioData(arrayBuffer);
          
          // Convert to Float32Array
          const audioData = audioBuffer.getChannelData(0);
          
          // Cleanup
          this.cleanup();
          
          resolve(audioData);
        } catch (error) {
          console.error('Error processing recorded audio:', error);
          reject(error);
        }
      };

      this.mediaRecorder.stop();
    });
  }

  async processAudio(audioData: Float32Array): Promise<RecognitionResult | null> {
    try {
      // Generate fingerprint
      const fingerprint = await this.generateFingerprint(audioData);
      
      // Send to backend for recognition
      const response = await fetch('/api/v1/recognition/recognize', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          fingerprint,
          audioData: Array.from(audioData),
          sampleRate: 44100,
        }),
      });

      if (!response.ok) {
        throw new Error('Recognition request failed');
      }

      const result = await response.json();
      return result;
    } catch (error) {
      console.error('Error processing audio:', error);
      return null;
    }
  }

  async generateFingerprint(audioData: Float32Array): Promise<string> {
    try {
      // For now, create a simple hash-based fingerprint
      // This will be replaced with the actual WebAssembly implementation
      const hash = await this.simpleHash(audioData);
      return hash;
    } catch (error) {
      console.error('Error generating fingerprint:', error);
      throw error;
    }
  }

  private async simpleHash(audioData: Float32Array): Promise<string> {
    // Simple hash implementation - will be replaced with WASM
    let hash = 0;
    for (let i = 0; i < audioData.length; i += 100) {
      hash = ((hash << 5) - hash + audioData[i]) & 0xffffffff;
    }
    return hash.toString(16);
  }

  private cleanup() {
    if (this.stream) {
      this.stream.getTracks().forEach(track => track.stop());
    }
    if (this.audioContext) {
      this.audioContext.close();
    }
    this.mediaRecorder = null;
    this.audioContext = null;
    this.analyser = null;
    this.microphone = null;
    this.stream = null;
    this.audioChunks = [];
  }

  // Get real-time audio data for visualization
  getAudioData(): Uint8Array | null {
    if (!this.analyser) return null;
    
    const bufferLength = this.analyser.frequencyBinCount;
    const dataArray = new Uint8Array(bufferLength);
    this.analyser.getByteFrequencyData(dataArray);
    
    return dataArray;
  }

  // Check if recording is active
  isRecording(): boolean {
    return this.mediaRecorder?.state === 'recording';
  }
}

export const audioProcessor = new SonicaAudioProcessor();
