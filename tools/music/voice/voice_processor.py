#!/usr/bin/env python3
"""
Voice Input Processor

A sophisticated voice processing component that converts audio input to text
and understands voice commands for music requests. Supports multiple audio
formats and uses advanced speech recognition models.
"""

import asyncio
import io
import json
import logging
import os
import tempfile
import time
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass
from pathlib import Path
import numpy as np
import wave
import whisper
import webrtcvad
import pyaudio
import librosa
from scipy.io import wavfile

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class AudioSegment:
    """Represents a segment of audio"""
    data: np.ndarray
    sample_rate: int
    duration: float
    start_time: float
    end_time: float
    is_speech: bool = False
    confidence: float = 0.0

@dataclass
class TranscriptionResult:
    """Represents the result of speech-to-text conversion"""
    text: str
    confidence: float
    language: str
    segments: List[Dict[str, Any]]
    processing_time: float
    audio_duration: float

class VoiceInputProcessor:
    """
    Advanced voice input processor for music requests
    """
    
    def __init__(self, model_name: str = "base", language: str = "en", 
                 use_gpu: bool = False, vad_aggressiveness: int = 2):
        """
        Initialize the voice input processor
        
        Args:
            model_name: Whisper model name (tiny, base, small, medium, large)
            language: Language code for speech recognition
            use_gpu: Whether to use GPU acceleration
            vad_aggressiveness: Voice activity detection aggressiveness (0-3)
        """
        self.model_name = model_name
        self.language = language
        self.use_gpu = use_gpu
        self.vad_aggressiveness = vad_aggressiveness
        
        # Audio processing parameters
        self.sample_rate = 16000  # Whisper's preferred sample rate
        self.chunk_size = 1024
        self.channels = 1
        self.format = pyaudio.paInt16
        
        # Initialize components
        self.whisper_model = None
        self.vad = None
        self.audio_interface = None
        
        # Audio preprocessing pipeline
        self.enable_noise_reduction = True
        self.enable_normalization = True
        self.enable_silence_removal = True
        
        # Initialize the processor
        self._initialize_components()
    
    def _initialize_components(self):
        """Initialize speech recognition and audio processing components"""
        try:
            # Initialize Whisper model
            logger.info(f"Loading Whisper model: {self.model_name}")
            self.whisper_model = whisper.load_model(self.model_name)
            
            # Initialize Voice Activity Detection
            self.vad = webrtcvad.Vad(self.vad_aggressiveness)
            
            # Initialize audio interface
            self.audio_interface = pyaudio.PyAudio()
            
            logger.info("Voice processing components initialized successfully")
            
        except Exception as e:
            logger.error(f"Error initializing voice components: {e}")
            raise
    
    def __del__(self):
        """Cleanup resources"""
        if self.audio_interface:
            self.audio_interface.terminate()
    
    async def process_audio_file(self, file_path: str) -> TranscriptionResult:
        """
        Process an audio file and convert to text
        
        Args:
            file_path: Path to the audio file
            
        Returns:
            TranscriptionResult object with transcribed text and metadata
        """
        if not os.path.exists(file_path):
            raise FileNotFoundError(f"Audio file not found: {file_path}")
        
        start_time = time.time()
        
        try:
            # Load and preprocess audio
            audio_data, duration = await self._load_audio_file(file_path)
            
            # Preprocess audio
            processed_audio = await self._preprocess_audio(audio_data)
            
            # Transcribe using Whisper
            result = await self._transcribe_audio(processed_audio)
            
            processing_time = time.time() - start_time
            
            return TranscriptionResult(
                text=result["text"].strip(),
                confidence=self._calculate_confidence(result),
                language=result.get("language", self.language),
                segments=result.get("segments", []),
                processing_time=processing_time,
                audio_duration=duration
            )
            
        except Exception as e:
            logger.error(f"Error processing audio file: {e}")
            raise
    
    async def process_audio_stream(self, duration: float = 5.0) -> TranscriptionResult:
        """
        Process real-time audio stream
        
        Args:
            duration: Duration to record in seconds
            
        Returns:
            TranscriptionResult object with transcribed text and metadata
        """
        start_time = time.time()
        
        try:
            # Record audio from microphone
            audio_data = await self._record_audio(duration)
            
            # Preprocess audio
            processed_audio = await self._preprocess_audio(audio_data)
            
            # Transcribe using Whisper
            result = await self._transcribe_audio(processed_audio)
            
            processing_time = time.time() - start_time
            
            return TranscriptionResult(
                text=result["text"].strip(),
                confidence=self._calculate_confidence(result),
                language=result.get("language", self.language),
                segments=result.get("segments", []),
                processing_time=processing_time,
                audio_duration=duration
            )
            
        except Exception as e:
            logger.error(f"Error processing audio stream: {e}")
            raise
    
    async def _load_audio_file(self, file_path: str) -> Tuple[np.ndarray, float]:
        """Load audio file and convert to numpy array"""
        try:
            # Use librosa for robust audio loading
            audio_data, sample_rate = librosa.load(
                file_path, 
                sr=self.sample_rate, 
                mono=True
            )
            
            duration = len(audio_data) / sample_rate
            
            logger.info(f"Loaded audio file: {file_path} ({duration:.2f}s)")
            return audio_data, duration
            
        except Exception as e:
            logger.error(f"Error loading audio file: {e}")
            raise
    
    async def _record_audio(self, duration: float) -> np.ndarray:
        """Record audio from microphone"""
        try:
            # Open audio stream
            stream = self.audio_interface.open(
                format=self.format,
                channels=self.channels,
                rate=self.sample_rate,
                input=True,
                frames_per_buffer=self.chunk_size
            )
            
            logger.info(f"Recording audio for {duration} seconds...")
            
            frames = []
            num_chunks = int(self.sample_rate * duration / self.chunk_size)
            
            for _ in range(num_chunks):
                data = stream.read(self.chunk_size)
                frames.append(data)
            
            stream.stop_stream()
            stream.close()
            
            # Convert to numpy array
            audio_data = np.frombuffer(b''.join(frames), dtype=np.int16)
            audio_data = audio_data.astype(np.float32) / 32768.0
            
            logger.info("Audio recording completed")
            return audio_data
            
        except Exception as e:
            logger.error(f"Error recording audio: {e}")
            raise
    
    async def _preprocess_audio(self, audio_data: np.ndarray) -> np.ndarray:
        """Preprocess audio for better recognition"""
        processed_audio = audio_data.copy()
        
        try:
            # Noise reduction
            if self.enable_noise_reduction:
                processed_audio = await self._reduce_noise(processed_audio)
            
            # Normalization
            if self.enable_normalization:
                processed_audio = await self._normalize_audio(processed_audio)
            
            # Silence removal
            if self.enable_silence_removal:
                processed_audio = await self._remove_silence(processed_audio)
            
            return processed_audio
            
        except Exception as e:
            logger.warning(f"Error in audio preprocessing: {e}")
            return audio_data
    
    async def _reduce_noise(self, audio_data: np.ndarray) -> np.ndarray:
        """Apply noise reduction to audio"""
        try:
            # Simple spectral subtraction for noise reduction
            # This is a basic implementation - could be enhanced with more sophisticated algorithms
            
            # Apply a high-pass filter to remove low-frequency noise
            from scipy.signal import butter, filtfilt
            
            # Design high-pass filter
            nyquist = self.sample_rate / 2
            low_cutoff = 80  # Hz
            high_cutoff = 8000  # Hz
            
            low = low_cutoff / nyquist
            high = high_cutoff / nyquist
            
            b, a = butter(4, [low, high], btype='band')
            filtered_audio = filtfilt(b, a, audio_data)
            
            return filtered_audio
            
        except Exception as e:
            logger.warning(f"Error in noise reduction: {e}")
            return audio_data
    
    async def _normalize_audio(self, audio_data: np.ndarray) -> np.ndarray:
        """Normalize audio amplitude"""
        try:
            # Peak normalization
            max_val = np.max(np.abs(audio_data))
            if max_val > 0:
                normalized_audio = audio_data / max_val
            else:
                normalized_audio = audio_data
            
            # Apply gentle compression
            compressed_audio = np.tanh(normalized_audio * 1.5) * 0.9
            
            return compressed_audio
            
        except Exception as e:
            logger.warning(f"Error in audio normalization: {e}")
            return audio_data
    
    async def _remove_silence(self, audio_data: np.ndarray) -> np.ndarray:
        """Remove silence from audio"""
        try:
            # Use librosa for voice activity detection
            intervals = librosa.effects.split(
                audio_data, 
                top_db=20, 
                frame_length=2048, 
                hop_length=512
            )
            
            if len(intervals) > 0:
                # Concatenate non-silent segments
                non_silent_audio = np.concatenate([
                    audio_data[start:end] for start, end in intervals
                ])
                return non_silent_audio
            else:
                return audio_data
                
        except Exception as e:
            logger.warning(f"Error in silence removal: {e}")
            return audio_data
    
    async def _transcribe_audio(self, audio_data: np.ndarray) -> Dict[str, Any]:
        """Transcribe audio using Whisper"""
        try:
            # Whisper expects audio in the range [-1, 1]
            audio_data = np.clip(audio_data, -1.0, 1.0)
            
            # Transcribe with Whisper
            result = self.whisper_model.transcribe(
                audio_data,
                language=self.language if self.language != "auto" else None,
                task="transcribe",
                verbose=False
            )
            
            return result
            
        except Exception as e:
            logger.error(f"Error in audio transcription: {e}")
            raise
    
    def _calculate_confidence(self, whisper_result: Dict[str, Any]) -> float:
        """Calculate confidence score from Whisper result"""
        try:
            # Whisper doesn't provide direct confidence scores
            # We'll estimate based on various factors
            
            segments = whisper_result.get("segments", [])
            if not segments:
                return 0.5
            
            # Calculate average confidence from segments
            total_confidence = 0
            total_duration = 0
            
            for segment in segments:
                # Use the probability if available, otherwise estimate
                if "avg_logprob" in segment:
                    # Convert log probability to confidence
                    confidence = np.exp(segment["avg_logprob"])
                else:
                    # Estimate confidence based on segment characteristics
                    confidence = 0.8  # Default confidence
                
                duration = segment["end"] - segment["start"]
                total_confidence += confidence * duration
                total_duration += duration
            
            if total_duration > 0:
                avg_confidence = total_confidence / total_duration
            else:
                avg_confidence = 0.5
            
            # Adjust confidence based on text quality
            text = whisper_result.get("text", "")
            if len(text.strip()) == 0:
                avg_confidence *= 0.1
            elif len(text.strip()) < 5:
                avg_confidence *= 0.5
            
            return min(max(avg_confidence, 0.0), 1.0)
            
        except Exception as e:
            logger.warning(f"Error calculating confidence: {e}")
            return 0.5
    
    async def detect_voice_activity(self, audio_data: np.ndarray) -> List[AudioSegment]:
        """Detect voice activity in audio"""
        try:
            # Convert to the format expected by WebRTC VAD
            audio_int16 = (audio_data * 32767).astype(np.int16)
            
            # WebRTC VAD works with specific frame sizes
            frame_duration = 30  # milliseconds
            frame_size = int(self.sample_rate * frame_duration / 1000)
            
            segments = []
            current_segment = None
            
            for i in range(0, len(audio_int16), frame_size):
                frame = audio_int16[i:i + frame_size]
                
                if len(frame) < frame_size:
                    # Pad the last frame if necessary
                    frame = np.pad(frame, (0, frame_size - len(frame)), 'constant')
                
                # Check if frame contains speech
                is_speech = self.vad.is_speech(frame.tobytes(), self.sample_rate)
                
                start_time = i / self.sample_rate
                end_time = (i + frame_size) / self.sample_rate
                
                if is_speech:
                    if current_segment is None:
                        # Start new speech segment
                        current_segment = {
                            "start": start_time,
                            "end": end_time,
                            "data": frame
                        }
                    else:
                        # Extend current segment
                        current_segment["end"] = end_time
                        current_segment["data"] = np.concatenate([
                            current_segment["data"], frame
                        ])
                else:
                    if current_segment is not None:
                        # End current speech segment
                        duration = current_segment["end"] - current_segment["start"]
                        
                        segment = AudioSegment(
                            data=current_segment["data"],
                            sample_rate=self.sample_rate,
                            duration=duration,
                            start_time=current_segment["start"],
                            end_time=current_segment["end"],
                            is_speech=True,
                            confidence=0.8
                        )
                        segments.append(segment)
                        current_segment = None
            
            # Handle last segment if it was speech
            if current_segment is not None:
                duration = current_segment["end"] - current_segment["start"]
                segment = AudioSegment(
                    data=current_segment["data"],
                    sample_rate=self.sample_rate,
                    duration=duration,
                    start_time=current_segment["start"],
                    end_time=current_segment["end"],
                    is_speech=True,
                    confidence=0.8
                )
                segments.append(segment)
            
            return segments
            
        except Exception as e:
            logger.error(f"Error in voice activity detection: {e}")
            return []
    
    async def process_audio(self, audio_input: str) -> str:
        """
        Main method to process audio input (file or stream)
        
        Args:
            audio_input: Path to audio file or "stream" for real-time recording
            
        Returns:
            Transcribed text
        """
        try:
            if audio_input.lower() == "stream":
                # Record from microphone
                result = await self.process_audio_stream(duration=5.0)
            else:
                # Process audio file
                result = await self.process_audio_file(audio_input)
            
            logger.info(f"Transcription result: '{result.text}' (confidence: {result.confidence:.2f})")
            return result.text
            
        except Exception as e:
            logger.error(f"Error processing audio: {e}")
            return ""
    
    async def get_audio_info(self, file_path: str) -> Dict[str, Any]:
        """Get information about an audio file"""
        try:
            audio_data, sample_rate = librosa.load(file_path, sr=None)
            duration = len(audio_data) / sample_rate
            
            return {
                "duration": duration,
                "sample_rate": sample_rate,
                "channels": 1,  # librosa loads as mono by default
                "format": Path(file_path).suffix.lower(),
                "file_size": os.path.getsize(file_path)
            }
            
        except Exception as e:
            logger.error(f"Error getting audio info: {e}")
            return {}
    
    async def save_audio_segment(self, audio_data: np.ndarray, output_path: str):
        """Save audio segment to file"""
        try:
            # Convert to int16 for WAV format
            audio_int16 = (audio_data * 32767).astype(np.int16)
            
            # Save as WAV file
            wavfile.write(output_path, self.sample_rate, audio_int16)
            
            logger.info(f"Audio segment saved to: {output_path}")
            
        except Exception as e:
            logger.error(f"Error saving audio segment: {e}")

# Example usage and testing
async def main():
    """Example usage of the voice input processor"""
    processor = VoiceInputProcessor(model_name="base", language="en")
    
    # Test with a sample audio file (if available)
    test_file = "test_audio.wav"
    if os.path.exists(test_file):
        print(f"Processing audio file: {test_file}")
        result = await processor.process_audio_file(test_file)
        print(f"Transcription: {result.text}")
        print(f"Confidence: {result.confidence:.2f}")
        print(f"Language: {result.language}")
        print(f"Processing time: {result.processing_time:.2f}s")
    else:
        print("No test audio file found")
    
    # Test with microphone input (commented out for automated testing)
    # print("Recording from microphone...")
    # result = await processor.process_audio_stream(duration=3.0)
    # print(f"Transcription: {result.text}")

if __name__ == "__main__":
    asyncio.run(main()) 