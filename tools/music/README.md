# Toka Music Playlist System

A comprehensive music playlist generation tool that integrates with Spotify and YouTube, supports natural language processing, and enables voice input for creating intelligent playlists.

## Features

- 🎵 **Multi-Platform Support**: Create playlists on Spotify and YouTube
- 🎤 **Voice Input**: Process voice commands and audio files for playlist generation
- 🧠 **Natural Language Processing**: Understand music requests in natural language
- 🤖 **Intelligent Recommendations**: Use AI to suggest songs based on mood, genre, and context
- 🔧 **Toka Integration**: Fully integrated with Toka's agent runtime system
- 🛡️ **Security**: Sandboxed execution with capability-based security
- 📊 **Analytics**: Track playlist generation metrics and user preferences

## Components

### 1. Music Playlist Orchestrator (`orchestrator/`)
The main entry point that coordinates all music services:
- Handles user requests and input processing
- Orchestrates Spotify and YouTube integrations
- Manages NLP and voice processing pipelines
- Provides unified API for agents

### 2. Spotify Integration (`spotify/`)
Complete Spotify Web API integration:
- OAuth 2.0 authentication
- Playlist creation and management
- Song search and recommendations
- Audio features analysis
- Rate limiting and error handling

### 3. YouTube Integration (`youtube/`)
Comprehensive YouTube Data API integration:
- YouTube playlist creation
- Music video search and filtering
- Voice activity detection
- Content categorization
- Trending music discovery

### 4. Natural Language Processor (`nlp/`)
Advanced NLP for music understanding:
- Extract genres, moods, tempo from text
- Understand music-related intent
- Support for multiple languages
- Context-aware processing
- Sentiment analysis

### 5. Voice Processor (`voice/`)
Sophisticated voice input handling:
- Speech-to-text using OpenAI Whisper
- Voice activity detection
- Audio preprocessing and enhancement
- Multiple audio format support
- Real-time and file-based processing

## Installation

### Prerequisites

- Python 3.8 or higher
- FFmpeg (for audio processing)
- System audio libraries (platform-specific)

### Quick Setup

1. **Run the setup script:**
   ```bash
   cd tools/music
   python3 setup.py
   ```

2. **Configure API credentials:**
   ```bash
   cp .env.template .env
   # Edit .env with your API keys
   ```

3. **Get API Credentials:**

   **Spotify API:**
   - Go to [Spotify Developer Dashboard](https://developer.spotify.com/)
   - Create a new app
   - Copy Client ID and Client Secret to `.env`

   **YouTube API:**
   - Go to [Google Cloud Console](https://console.cloud.google.com/)
   - Enable YouTube Data API v3
   - Create credentials (API Key + OAuth 2.0)
   - Download credentials JSON file

### Manual Installation

If the setup script doesn't work for your system:

```bash
# Install system dependencies (Linux/Ubuntu)
sudo apt-get update
sudo apt-get install -y ffmpeg portaudio19-dev python3-dev build-essential

# Install Python dependencies
pip install -r requirements.txt

# Download spaCy model
python -m spacy download en_core_web_sm
```

## Usage

### Command Line Interface

```bash
# Basic usage
python3 orchestrator/music_playlist_orchestrator.py \
  --description "Create a happy workout playlist" \
  --platform both \
  --max-songs 30

# Voice input
python3 orchestrator/music_playlist_orchestrator.py \
  --description "relaxing jazz music" \
  --input-type voice \
  --audio-file recording.wav

# Specific preferences
python3 orchestrator/music_playlist_orchestrator.py \
  --description "90s rock hits for driving" \
  --genres rock \
  --decade 1990s \
  --mood energetic \
  --platform spotify
```

### Toka Agent Integration

The music system is registered as a Toka tool and can be invoked by agents:

```python
# Example agent usage
await toka_runtime.execute_tool(
    "music-playlist-orchestrator",
    {
        "description": "Create a study playlist with focus music",
        "platform": "spotify",
        "mood": "calm",
        "max_songs": 25
    }
)
```

### Python API

```python
from orchestrator.music_playlist_orchestrator import MusicPlaylistOrchestrator, PlaylistRequest, PlatformType

# Create orchestrator
config = load_config()
orchestrator = MusicPlaylistOrchestrator(config)

# Create playlist request
request = PlaylistRequest(
    description="Energetic workout music",
    platform=PlatformType.BOTH,
    genres=["electronic", "hip-hop"],
    mood="energetic",
    max_songs=40
)

# Generate playlist
results = await orchestrator.generate_playlist(request)
```

## Configuration

The system uses a comprehensive configuration file (`config/music_config.json`):

```json
{
  "spotify": {
    "client_id": "${SPOTIFY_CLIENT_ID}",
    "client_secret": "${SPOTIFY_CLIENT_SECRET}"
  },
  "youtube": {
    "api_key": "${YOUTUBE_API_KEY}",
    "credentials_file": "${YOUTUBE_CREDENTIALS_FILE}"
  },
  "nlp": {
    "model_name": "en_core_web_sm",
    "use_gpu": false
  },
  "voice": {
    "model_name": "base",
    "language": "en"
  }
}
```

## Natural Language Examples

The NLP processor understands various natural language requests:

- "Create a happy workout playlist"
- "I need some relaxing jazz for studying"
- "Play energetic dance music for my party"
- "Give me sad songs from the 90s"
- "Chill hip-hop for driving"
- "Romantic ballads for date night"
- "Upbeat pop music to wake up to"

## Voice Input

Supports multiple audio formats and processing modes:

- **File-based**: Process pre-recorded audio files (MP3, WAV, M4A, OGG, FLAC)
- **Real-time**: Record directly from microphone
- **Streaming**: Process audio streams in real-time

Features:
- Noise reduction and audio enhancement
- Voice activity detection
- Multiple language support
- Confidence scoring

## API Integrations

### Spotify Web API
- **Authentication**: OAuth 2.0 with PKCE
- **Endpoints**: Search, Recommendations, Playlists, Audio Features
- **Rate Limiting**: Intelligent handling of API limits
- **Features**: Audio analysis, personalized recommendations

### YouTube Data API v3
- **Authentication**: API Key + OAuth 2.0
- **Endpoints**: Search, Playlists, Videos, Channels
- **Content Filtering**: Music-specific video filtering
- **Features**: Trending discovery, metadata extraction

## Security

- **Sandboxed Execution**: Runs in controlled environment
- **Capability-Based**: Only requests necessary permissions
- **Token Encryption**: Secure credential storage
- **Network Isolation**: Limited to approved domains
- **Input Validation**: Comprehensive input sanitization

## Monitoring and Analytics

The system provides comprehensive monitoring:

- **Performance Metrics**: Response times, API efficiency
- **Usage Analytics**: Popular genres, platform preferences
- **Error Tracking**: Detailed error logging and reporting
- **User Satisfaction**: Playlist success rates

## Troubleshooting

### Common Issues

1. **Audio Processing Errors**
   ```bash
   # Install additional audio codecs
   sudo apt-get install -y ubuntu-restricted-extras
   ```

2. **Spotify Authentication**
   - Ensure redirect URI matches exactly
   - Check client credentials
   - Verify scope permissions

3. **YouTube API Limits**
   - Monitor quota usage in Google Cloud Console
   - Implement request batching
   - Use caching for repeated requests

4. **Voice Recognition Issues**
   - Check microphone permissions
   - Verify audio format compatibility
   - Adjust noise reduction settings

### Logs and Debugging

```bash
# Enable verbose logging
export LOG_LEVEL=DEBUG

# Check logs
tail -f logs/music_playlist.log

# Test individual components
python3 spotify/spotify_client.py
python3 youtube/youtube_client.py
python3 nlp/language_processor.py
python3 voice/voice_processor.py
```

## Development

### Project Structure

```
tools/music/
├── orchestrator/           # Main orchestrator
│   └── music_playlist_orchestrator.py
├── spotify/               # Spotify integration
│   └── spotify_client.py
├── youtube/               # YouTube integration
│   └── youtube_client.py
├── nlp/                   # Natural language processing
│   └── language_processor.py
├── voice/                 # Voice input processing
│   └── voice_processor.py
├── config/                # Configuration files
├── tests/                 # Test suite
├── requirements.txt       # Python dependencies
├── setup.py              # Setup script
└── README.md             # This file
```

### Testing

```bash
# Install test dependencies
pip install pytest pytest-asyncio pytest-cov

# Run tests
pytest tests/ -v

# Run with coverage
pytest tests/ --cov=. --cov-report=html
```

### Contributing

1. Follow the Toka coding standards
2. Add comprehensive tests for new features
3. Update documentation
4. Ensure security compliance
5. Test with multiple platforms

## License

This project is part of the Toka system and follows the same licensing terms.

## Support

For issues and questions:
1. Check the troubleshooting section
2. Review logs for error details
3. Consult Toka documentation
4. Create an issue in the Toka repository

## Roadmap

Future enhancements:
- [ ] Apple Music integration
- [ ] Collaborative playlist creation
- [ ] Advanced ML recommendations
- [ ] Social sharing features
- [ ] Multi-language NLP support
- [ ] Real-time collaboration
- [ ] Playlist analytics dashboard
- [ ] Voice conversation mode 