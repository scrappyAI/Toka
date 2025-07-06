#!/usr/bin/env python3
"""
Toka Music Playlist Orchestrator

A comprehensive music playlist generation tool that integrates with Spotify and YouTube,
supports natural language processing, and enables voice input for creating playlists.
This tool serves as the main entry point for all music-related operations in Toka.
"""

import asyncio
import json
import logging
import os
import sys
from pathlib import Path
from typing import Dict, List, Optional, Union, Any
from dataclasses import dataclass, asdict
from enum import Enum
import argparse

# Add the tools directory to the path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from spotify.spotify_client import SpotifyPlaylistClient
    from youtube.youtube_client import YouTubePlaylistClient
    from nlp.language_processor import MusicLanguageProcessor
    from voice.voice_processor import VoiceInputProcessor
except ImportError as e:
    logging.warning(f"Some components not available: {e}")
    # Create stub classes for development
    class SpotifyPlaylistClient:
        def __init__(self, *args, **kwargs): pass
        async def create_playlist(self, *args, **kwargs): return {"id": "mock_spotify", "url": "mock"}
    
    class YouTubePlaylistClient:
        def __init__(self, *args, **kwargs): pass
        async def create_playlist(self, *args, **kwargs): return {"id": "mock_youtube", "url": "mock"}
    
    class MusicLanguageProcessor:
        def __init__(self, *args, **kwargs): pass
        async def process_request(self, *args, **kwargs): return {"songs": [], "genre": "pop", "mood": "happy"}
    
    class VoiceInputProcessor:
        def __init__(self, *args, **kwargs): pass
        async def process_audio(self, *args, **kwargs): return "Create a happy playlist"

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class PlatformType(Enum):
    """Supported music platforms"""
    SPOTIFY = "spotify"
    YOUTUBE = "youtube"
    BOTH = "both"

class InputType(Enum):
    """Supported input types"""
    TEXT = "text"
    VOICE = "voice"
    AUTO = "auto"

@dataclass
class PlaylistRequest:
    """Represents a playlist generation request"""
    description: str
    platform: PlatformType
    input_type: InputType
    name: Optional[str] = None
    public: bool = True
    max_songs: int = 50
    audio_file: Optional[str] = None
    genres: Optional[List[str]] = None
    mood: Optional[str] = None
    tempo: Optional[str] = None
    decade: Optional[str] = None

@dataclass
class PlaylistResult:
    """Represents a playlist generation result"""
    success: bool
    platform: str
    playlist_id: str
    playlist_url: str
    playlist_name: str
    songs_added: int
    total_duration: Optional[str] = None
    error: Optional[str] = None
    metadata: Optional[Dict[str, Any]] = None

class MusicPlaylistOrchestrator:
    """
    Main orchestrator for music playlist generation across multiple platforms
    """
    
    def __init__(self, config: Optional[Dict[str, Any]] = None):
        """Initialize the orchestrator with configuration"""
        self.config = config or {}
        self.spotify_client = None
        self.youtube_client = None
        self.nlp_processor = None
        self.voice_processor = None
        
        # Initialize components
        self._initialize_components()
    
    def _initialize_components(self):
        """Initialize all music service components"""
        try:
            # Initialize Spotify client
            spotify_config = self.config.get('spotify', {})
            self.spotify_client = SpotifyPlaylistClient(
                client_id=spotify_config.get('client_id'),
                client_secret=spotify_config.get('client_secret'),
                redirect_uri=spotify_config.get('redirect_uri', 'http://localhost:8888/callback')
            )
            
            # Initialize YouTube client
            youtube_config = self.config.get('youtube', {})
            self.youtube_client = YouTubePlaylistClient(
                api_key=youtube_config.get('api_key'),
                credentials_file=youtube_config.get('credentials_file')
            )
            
            # Initialize NLP processor
            nlp_config = self.config.get('nlp', {})
            self.nlp_processor = MusicLanguageProcessor(
                model_name=nlp_config.get('model_name', 'distilbert-base-uncased'),
                use_gpu=nlp_config.get('use_gpu', False)
            )
            
            # Initialize voice processor
            voice_config = self.config.get('voice', {})
            self.voice_processor = VoiceInputProcessor(
                model_name=voice_config.get('model_name', 'openai/whisper-base'),
                language=voice_config.get('language', 'en')
            )
            
            logger.info("All music components initialized successfully")
            
        except Exception as e:
            logger.error(f"Error initializing components: {e}")
            raise
    
    async def generate_playlist(self, request: PlaylistRequest) -> List[PlaylistResult]:
        """
        Generate a playlist based on the request
        """
        results = []
        
        try:
            # Process input based on type
            processed_request = await self._process_input(request)
            
            # Generate playlist on requested platforms
            if request.platform == PlatformType.SPOTIFY or request.platform == PlatformType.BOTH:
                spotify_result = await self._create_spotify_playlist(processed_request)
                results.append(spotify_result)
            
            if request.platform == PlatformType.YOUTUBE or request.platform == PlatformType.BOTH:
                youtube_result = await self._create_youtube_playlist(processed_request)
                results.append(youtube_result)
            
            return results
            
        except Exception as e:
            logger.error(f"Error generating playlist: {e}")
            return [PlaylistResult(
                success=False,
                platform="unknown",
                playlist_id="",
                playlist_url="",
                playlist_name="",
                songs_added=0,
                error=str(e)
            )]
    
    async def _process_input(self, request: PlaylistRequest) -> PlaylistRequest:
        """Process the input based on type (text, voice, or auto)"""
        
        if request.input_type == InputType.VOICE and request.audio_file:
            # Process voice input
            logger.info("Processing voice input...")
            voice_text = await self.voice_processor.process_audio(request.audio_file)
            request.description = voice_text
            
        elif request.input_type == InputType.AUTO and request.audio_file:
            # Auto-detect: assume voice if audio file provided
            logger.info("Auto-detecting voice input...")
            voice_text = await self.voice_processor.process_audio(request.audio_file)
            request.description = voice_text
        
        # Process natural language description
        logger.info("Processing natural language description...")
        nlp_result = await self.nlp_processor.process_request(request.description)
        
        # Update request with NLP insights
        if not request.genres and nlp_result.get('genres'):
            request.genres = nlp_result['genres']
        if not request.mood and nlp_result.get('mood'):
            request.mood = nlp_result['mood']
        if not request.tempo and nlp_result.get('tempo'):
            request.tempo = nlp_result['tempo']
        if not request.decade and nlp_result.get('decade'):
            request.decade = nlp_result['decade']
        
        return request
    
    async def _create_spotify_playlist(self, request: PlaylistRequest) -> PlaylistResult:
        """Create a playlist on Spotify"""
        try:
            logger.info("Creating Spotify playlist...")
            
            result = await self.spotify_client.create_playlist(
                name=request.name or f"Toka Playlist - {request.description[:50]}",
                description=f"Generated by Toka: {request.description}",
                public=request.public,
                genres=request.genres,
                mood=request.mood,
                tempo=request.tempo,
                decade=request.decade,
                max_songs=request.max_songs
            )
            
            return PlaylistResult(
                success=True,
                platform="spotify",
                playlist_id=result['id'],
                playlist_url=result['url'],
                playlist_name=result['name'],
                songs_added=result.get('songs_added', 0),
                total_duration=result.get('total_duration'),
                metadata=result.get('metadata')
            )
            
        except Exception as e:
            logger.error(f"Error creating Spotify playlist: {e}")
            return PlaylistResult(
                success=False,
                platform="spotify",
                playlist_id="",
                playlist_url="",
                playlist_name="",
                songs_added=0,
                error=str(e)
            )
    
    async def _create_youtube_playlist(self, request: PlaylistRequest) -> PlaylistResult:
        """Create a playlist on YouTube"""
        try:
            logger.info("Creating YouTube playlist...")
            
            result = await self.youtube_client.create_playlist(
                name=request.name or f"Toka Playlist - {request.description[:50]}",
                description=f"Generated by Toka: {request.description}",
                public=request.public,
                genres=request.genres,
                mood=request.mood,
                tempo=request.tempo,
                decade=request.decade,
                max_songs=request.max_songs
            )
            
            return PlaylistResult(
                success=True,
                platform="youtube",
                playlist_id=result['id'],
                playlist_url=result['url'],
                playlist_name=result['name'],
                songs_added=result.get('songs_added', 0),
                total_duration=result.get('total_duration'),
                metadata=result.get('metadata')
            )
            
        except Exception as e:
            logger.error(f"Error creating YouTube playlist: {e}")
            return PlaylistResult(
                success=False,
                platform="youtube",
                playlist_id="",
                playlist_url="",
                playlist_name="",
                songs_added=0,
                error=str(e)
            )
    
    async def search_songs(self, query: str, platform: PlatformType = PlatformType.BOTH, limit: int = 10) -> Dict[str, List[Dict]]:
        """Search for songs across platforms"""
        results = {}
        
        if platform == PlatformType.SPOTIFY or platform == PlatformType.BOTH:
            try:
                spotify_results = await self.spotify_client.search_songs(query, limit)
                results['spotify'] = spotify_results
            except Exception as e:
                logger.error(f"Spotify search error: {e}")
                results['spotify'] = []
        
        if platform == PlatformType.YOUTUBE or platform == PlatformType.BOTH:
            try:
                youtube_results = await self.youtube_client.search_songs(query, limit)
                results['youtube'] = youtube_results
            except Exception as e:
                logger.error(f"YouTube search error: {e}")
                results['youtube'] = []
        
        return results
    
    async def get_playlist_info(self, playlist_id: str, platform: PlatformType) -> Dict[str, Any]:
        """Get information about an existing playlist"""
        try:
            if platform == PlatformType.SPOTIFY:
                return await self.spotify_client.get_playlist_info(playlist_id)
            elif platform == PlatformType.YOUTUBE:
                return await self.youtube_client.get_playlist_info(playlist_id)
            else:
                return {"error": "Platform not supported for single playlist lookup"}
        except Exception as e:
            logger.error(f"Error getting playlist info: {e}")
            return {"error": str(e)}

def load_config() -> Dict[str, Any]:
    """Load configuration from environment variables and config files"""
    config = {}
    
    # Load from environment variables
    config['spotify'] = {
        'client_id': os.getenv('SPOTIFY_CLIENT_ID'),
        'client_secret': os.getenv('SPOTIFY_CLIENT_SECRET'),
        'redirect_uri': os.getenv('SPOTIFY_REDIRECT_URI', 'http://localhost:8888/callback')
    }
    
    config['youtube'] = {
        'api_key': os.getenv('YOUTUBE_API_KEY'),
        'credentials_file': os.getenv('YOUTUBE_CREDENTIALS_FILE', 'youtube_credentials.json')
    }
    
    config['nlp'] = {
        'model_name': os.getenv('NLP_MODEL_NAME', 'distilbert-base-uncased'),
        'use_gpu': os.getenv('NLP_USE_GPU', 'false').lower() == 'true'
    }
    
    config['voice'] = {
        'model_name': os.getenv('VOICE_MODEL_NAME', 'openai/whisper-base'),
        'language': os.getenv('VOICE_LANGUAGE', 'en')
    }
    
    # Try to load from config file
    config_file = Path('config/music_config.json')
    if config_file.exists():
        try:
            with open(config_file, 'r') as f:
                file_config = json.load(f)
                # Merge with environment config (env vars take precedence)
                for key, value in file_config.items():
                    if key not in config:
                        config[key] = value
                    else:
                        config[key].update(value)
        except Exception as e:
            logger.warning(f"Error loading config file: {e}")
    
    return config

async def main():
    """Main entry point for the music playlist orchestrator"""
    parser = argparse.ArgumentParser(description='Toka Music Playlist Orchestrator')
    parser.add_argument('--description', '-d', required=True, help='Playlist description or request')
    parser.add_argument('--platform', '-p', choices=['spotify', 'youtube', 'both'], default='both', 
                       help='Platform to create playlist on')
    parser.add_argument('--input-type', '-i', choices=['text', 'voice', 'auto'], default='auto',
                       help='Input type (text, voice, or auto-detect)')
    parser.add_argument('--name', '-n', help='Playlist name (optional)')
    parser.add_argument('--audio-file', '-a', help='Audio file for voice input')
    parser.add_argument('--max-songs', '-m', type=int, default=50, help='Maximum number of songs')
    parser.add_argument('--private', action='store_true', help='Make playlist private')
    parser.add_argument('--genres', nargs='+', help='Preferred genres')
    parser.add_argument('--mood', help='Mood for the playlist')
    parser.add_argument('--tempo', help='Tempo preference (slow, medium, fast)')
    parser.add_argument('--decade', help='Decade preference (e.g., 2000s, 90s)')
    parser.add_argument('--config', '-c', help='Config file path')
    parser.add_argument('--output', '-o', help='Output file for results')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    try:
        # Load configuration
        config = load_config()
        
        # Create orchestrator
        orchestrator = MusicPlaylistOrchestrator(config)
        
        # Create request
        request = PlaylistRequest(
            description=args.description,
            platform=PlatformType(args.platform),
            input_type=InputType(args.input_type),
            name=args.name,
            public=not args.private,
            max_songs=args.max_songs,
            audio_file=args.audio_file,
            genres=args.genres,
            mood=args.mood,
            tempo=args.tempo,
            decade=args.decade
        )
        
        # Generate playlist
        results = await orchestrator.generate_playlist(request)
        
        # Output results
        output_data = {
            'request': asdict(request),
            'results': [asdict(result) for result in results],
            'timestamp': str(asyncio.get_event_loop().time())
        }
        
        if args.output:
            with open(args.output, 'w') as f:
                json.dump(output_data, f, indent=2)
            print(f"Results saved to {args.output}")
        else:
            print(json.dumps(output_data, indent=2))
        
        # Print summary
        print("\n=== Playlist Generation Summary ===")
        for result in results:
            if result.success:
                print(f"✓ {result.platform.title()}: {result.playlist_name}")
                print(f"  URL: {result.playlist_url}")
                print(f"  Songs: {result.songs_added}")
                if result.total_duration:
                    print(f"  Duration: {result.total_duration}")
            else:
                print(f"✗ {result.platform.title()}: {result.error}")
        
    except Exception as e:
        logger.error(f"Error in main: {e}")
        sys.exit(1)

if __name__ == "__main__":
    asyncio.run(main()) 