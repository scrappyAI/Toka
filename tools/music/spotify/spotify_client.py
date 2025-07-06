#!/usr/bin/env python3
"""
Spotify Playlist Client

A comprehensive client for interacting with Spotify's Web API to create playlists,
search for music, and manage user's music library. This client handles authentication,
rate limiting, and provides intelligent playlist generation capabilities.
"""

import asyncio
import base64
import json
import logging
import os
import random
import time
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass
from urllib.parse import urlencode, parse_qs
import aiohttp
import webbrowser
from datetime import datetime, timedelta

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class SpotifyTrack:
    """Represents a Spotify track"""
    id: str
    name: str
    artists: List[str]
    album: str
    duration_ms: int
    popularity: int
    preview_url: Optional[str] = None
    external_urls: Optional[Dict[str, str]] = None
    audio_features: Optional[Dict[str, float]] = None

@dataclass
class SpotifyPlaylist:
    """Represents a Spotify playlist"""
    id: str
    name: str
    description: str
    public: bool
    collaborative: bool
    url: str
    track_count: int
    owner: str
    images: List[Dict[str, Any]]

class SpotifyAuthError(Exception):
    """Raised when authentication fails"""
    pass

class SpotifyAPIError(Exception):
    """Raised when API requests fail"""
    pass

class SpotifyPlaylistClient:
    """
    Spotify Web API client for playlist management and music discovery
    """
    
    BASE_URL = "https://api.spotify.com/v1"
    AUTH_URL = "https://accounts.spotify.com/api/token"
    AUTHORIZE_URL = "https://accounts.spotify.com/authorize"
    
    def __init__(self, client_id: str, client_secret: str, redirect_uri: str = "http://localhost:8888/callback"):
        """
        Initialize the Spotify client
        
        Args:
            client_id: Spotify application client ID
            client_secret: Spotify application client secret
            redirect_uri: OAuth redirect URI
        """
        self.client_id = client_id
        self.client_secret = client_secret
        self.redirect_uri = redirect_uri
        self.access_token = None
        self.refresh_token = None
        self.token_expires_at = None
        self.session = None
        self.user_id = None
        
        # Rate limiting
        self.rate_limit_remaining = 1000
        self.rate_limit_reset = time.time()
        
        # Audio features cache
        self.audio_features_cache = {}
        
        # Genre seeds (Spotify's available genre seeds)
        self.genre_seeds = [
            "acoustic", "afrobeat", "alt-rock", "alternative", "ambient",
            "blues", "bossanova", "brazil", "breakbeat", "british",
            "chill", "classical", "club", "country", "dance",
            "dancehall", "deep-house", "disco", "drum-and-bass", "dub",
            "dubstep", "electronic", "folk", "funk", "garage",
            "gospel", "groove", "grunge", "happy", "hard-rock",
            "hip-hop", "house", "indie", "indie-pop", "jazz",
            "latin", "metal", "pop", "punk", "r-n-b",
            "reggae", "rock", "soul", "techno", "trance"
        ]
    
    async def __aenter__(self):
        """Async context manager entry"""
        self.session = aiohttp.ClientSession()
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit"""
        if self.session:
            await self.session.close()
    
    async def authenticate(self, authorization_code: Optional[str] = None) -> bool:
        """
        Authenticate with Spotify using OAuth 2.0
        
        Args:
            authorization_code: OAuth authorization code (if available)
            
        Returns:
            True if authentication successful, False otherwise
        """
        if not self.session:
            self.session = aiohttp.ClientSession()
        
        try:
            if authorization_code:
                # Exchange authorization code for access token
                await self._exchange_code_for_token(authorization_code)
            else:
                # Try client credentials flow first (for non-user-specific operations)
                await self._client_credentials_flow()
            
            # Get user profile if we have user access
            if self.access_token and not self.user_id:
                try:
                    await self._get_user_profile()
                except Exception as e:
                    logger.warning(f"Could not get user profile: {e}")
            
            return True
            
        except Exception as e:
            logger.error(f"Authentication failed: {e}")
            return False
    
    async def _client_credentials_flow(self):
        """Use client credentials flow for app-only access"""
        auth_header = base64.b64encode(f"{self.client_id}:{self.client_secret}".encode()).decode()
        
        headers = {
            "Authorization": f"Basic {auth_header}",
            "Content-Type": "application/x-www-form-urlencoded"
        }
        
        data = {"grant_type": "client_credentials"}
        
        async with self.session.post(self.AUTH_URL, headers=headers, data=data) as response:
            if response.status == 200:
                token_data = await response.json()
                self.access_token = token_data["access_token"]
                self.token_expires_at = time.time() + token_data["expires_in"]
                logger.info("Client credentials authentication successful")
            else:
                error_data = await response.json()
                raise SpotifyAuthError(f"Authentication failed: {error_data}")
    
    async def _exchange_code_for_token(self, authorization_code: str):
        """Exchange authorization code for access token"""
        auth_header = base64.b64encode(f"{self.client_id}:{self.client_secret}".encode()).decode()
        
        headers = {
            "Authorization": f"Basic {auth_header}",
            "Content-Type": "application/x-www-form-urlencoded"
        }
        
        data = {
            "grant_type": "authorization_code",
            "code": authorization_code,
            "redirect_uri": self.redirect_uri
        }
        
        async with self.session.post(self.AUTH_URL, headers=headers, data=data) as response:
            if response.status == 200:
                token_data = await response.json()
                self.access_token = token_data["access_token"]
                self.refresh_token = token_data.get("refresh_token")
                self.token_expires_at = time.time() + token_data["expires_in"]
                logger.info("Authorization code exchange successful")
            else:
                error_data = await response.json()
                raise SpotifyAuthError(f"Token exchange failed: {error_data}")
    
    async def _get_user_profile(self):
        """Get current user's profile"""
        response = await self._make_request("GET", "/me")
        if response:
            self.user_id = response.get("id")
            logger.info(f"User profile loaded: {self.user_id}")
    
    async def _refresh_access_token(self):
        """Refresh the access token using refresh token"""
        if not self.refresh_token:
            # Fall back to client credentials
            await self._client_credentials_flow()
            return
        
        auth_header = base64.b64encode(f"{self.client_id}:{self.client_secret}".encode()).decode()
        
        headers = {
            "Authorization": f"Basic {auth_header}",
            "Content-Type": "application/x-www-form-urlencoded"
        }
        
        data = {
            "grant_type": "refresh_token",
            "refresh_token": self.refresh_token
        }
        
        async with self.session.post(self.AUTH_URL, headers=headers, data=data) as response:
            if response.status == 200:
                token_data = await response.json()
                self.access_token = token_data["access_token"]
                self.token_expires_at = time.time() + token_data["expires_in"]
                if "refresh_token" in token_data:
                    self.refresh_token = token_data["refresh_token"]
                logger.info("Access token refreshed")
            else:
                error_data = await response.json()
                raise SpotifyAuthError(f"Token refresh failed: {error_data}")
    
    async def _make_request(self, method: str, endpoint: str, **kwargs) -> Optional[Dict[str, Any]]:
        """
        Make an authenticated request to Spotify API
        
        Args:
            method: HTTP method
            endpoint: API endpoint (without base URL)
            **kwargs: Additional arguments for the request
            
        Returns:
            Response data or None if request failed
        """
        if not self.access_token or time.time() >= self.token_expires_at:
            await self._refresh_access_token()
        
        # Handle rate limiting
        if time.time() < self.rate_limit_reset:
            await asyncio.sleep(self.rate_limit_reset - time.time())
        
        url = f"{self.BASE_URL}{endpoint}"
        headers = {
            "Authorization": f"Bearer {self.access_token}",
            "Content-Type": "application/json"
        }
        
        try:
            async with self.session.request(method, url, headers=headers, **kwargs) as response:
                # Update rate limit info
                self.rate_limit_remaining = int(response.headers.get("X-RateLimit-Remaining", "0"))
                if "X-RateLimit-Reset" in response.headers:
                    self.rate_limit_reset = time.time() + int(response.headers["X-RateLimit-Reset"])
                
                if response.status == 200 or response.status == 201:
                    return await response.json()
                elif response.status == 429:
                    # Rate limited
                    retry_after = int(response.headers.get("Retry-After", "1"))
                    logger.warning(f"Rate limited, waiting {retry_after} seconds")
                    await asyncio.sleep(retry_after)
                    return await self._make_request(method, endpoint, **kwargs)
                else:
                    error_data = await response.json()
                    logger.error(f"API request failed: {response.status} - {error_data}")
                    return None
                    
        except Exception as e:
            logger.error(f"Request error: {e}")
            return None
    
    async def search_songs(self, query: str, limit: int = 20, **filters) -> List[SpotifyTrack]:
        """
        Search for songs on Spotify
        
        Args:
            query: Search query
            limit: Maximum number of results
            **filters: Additional filters (genre, artist, etc.)
            
        Returns:
            List of SpotifyTrack objects
        """
        # Build search query
        search_query = query
        if filters.get("artist"):
            search_query += f" artist:{filters['artist']}"
        if filters.get("genre"):
            search_query += f" genre:{filters['genre']}"
        if filters.get("year"):
            search_query += f" year:{filters['year']}"
        
        params = {
            "q": search_query,
            "type": "track",
            "limit": min(limit, 50)
        }
        
        response = await self._make_request("GET", "/search", params=params)
        if not response or "tracks" not in response:
            return []
        
        tracks = []
        for track_data in response["tracks"]["items"]:
            track = SpotifyTrack(
                id=track_data["id"],
                name=track_data["name"],
                artists=[artist["name"] for artist in track_data["artists"]],
                album=track_data["album"]["name"],
                duration_ms=track_data["duration_ms"],
                popularity=track_data["popularity"],
                preview_url=track_data.get("preview_url"),
                external_urls=track_data.get("external_urls")
            )
            tracks.append(track)
        
        return tracks
    
    async def get_recommendations(self, seed_tracks: List[str] = None, seed_artists: List[str] = None, 
                                 seed_genres: List[str] = None, limit: int = 20, **audio_features) -> List[SpotifyTrack]:
        """
        Get song recommendations from Spotify
        
        Args:
            seed_tracks: List of track IDs to base recommendations on
            seed_artists: List of artist IDs to base recommendations on
            seed_genres: List of genres to base recommendations on
            limit: Maximum number of recommendations
            **audio_features: Target audio features (energy, danceability, etc.)
            
        Returns:
            List of recommended SpotifyTrack objects
        """
        params = {"limit": min(limit, 100)}
        
        if seed_tracks:
            params["seed_tracks"] = ",".join(seed_tracks[:5])
        if seed_artists:
            params["seed_artists"] = ",".join(seed_artists[:5])
        if seed_genres:
            valid_genres = [g for g in seed_genres if g in self.genre_seeds]
            if valid_genres:
                params["seed_genres"] = ",".join(valid_genres[:5])
        
        # Add audio feature targets
        for feature, value in audio_features.items():
            if feature.startswith("target_"):
                params[feature] = value
        
        response = await self._make_request("GET", "/recommendations", params=params)
        if not response or "tracks" not in response:
            return []
        
        tracks = []
        for track_data in response["tracks"]:
            track = SpotifyTrack(
                id=track_data["id"],
                name=track_data["name"],
                artists=[artist["name"] for artist in track_data["artists"]],
                album=track_data["album"]["name"],
                duration_ms=track_data["duration_ms"],
                popularity=track_data["popularity"],
                preview_url=track_data.get("preview_url"),
                external_urls=track_data.get("external_urls")
            )
            tracks.append(track)
        
        return tracks
    
    async def get_audio_features(self, track_ids: List[str]) -> Dict[str, Dict[str, float]]:
        """
        Get audio features for tracks
        
        Args:
            track_ids: List of track IDs
            
        Returns:
            Dictionary mapping track IDs to audio features
        """
        # Check cache first
        cached_features = {}
        uncached_ids = []
        
        for track_id in track_ids:
            if track_id in self.audio_features_cache:
                cached_features[track_id] = self.audio_features_cache[track_id]
            else:
                uncached_ids.append(track_id)
        
        if not uncached_ids:
            return cached_features
        
        # Fetch uncached features (API supports up to 100 tracks per request)
        all_features = cached_features.copy()
        
        for i in range(0, len(uncached_ids), 100):
            batch = uncached_ids[i:i+100]
            params = {"ids": ",".join(batch)}
            
            response = await self._make_request("GET", "/audio-features", params=params)
            if response and "audio_features" in response:
                for features in response["audio_features"]:
                    if features:  # Can be None for some tracks
                        track_id = features["id"]
                        feature_data = {
                            "acousticness": features["acousticness"],
                            "danceability": features["danceability"],
                            "energy": features["energy"],
                            "instrumentalness": features["instrumentalness"],
                            "liveness": features["liveness"],
                            "speechiness": features["speechiness"],
                            "valence": features["valence"],
                            "tempo": features["tempo"],
                            "loudness": features["loudness"]
                        }
                        all_features[track_id] = feature_data
                        self.audio_features_cache[track_id] = feature_data
        
        return all_features
    
    async def create_playlist(self, name: str, description: str = "", public: bool = True,
                             genres: List[str] = None, mood: str = None, tempo: str = None,
                             decade: str = None, max_songs: int = 50) -> Dict[str, Any]:
        """
        Create a new playlist with intelligent song selection
        
        Args:
            name: Playlist name
            description: Playlist description
            public: Whether playlist should be public
            genres: Preferred genres
            mood: Mood for the playlist (happy, sad, energetic, etc.)
            tempo: Tempo preference (slow, medium, fast)
            decade: Decade preference (1990s, 2000s, etc.)
            max_songs: Maximum number of songs to add
            
        Returns:
            Dictionary with playlist information
        """
        if not self.user_id:
            raise SpotifyAPIError("User authentication required to create playlists")
        
        # Create empty playlist
        playlist_data = {
            "name": name,
            "description": description,
            "public": public
        }
        
        response = await self._make_request("POST", f"/users/{self.user_id}/playlists", json=playlist_data)
        if not response:
            raise SpotifyAPIError("Failed to create playlist")
        
        playlist_id = response["id"]
        playlist_url = response["external_urls"]["spotify"]
        
        # Generate intelligent song selection
        tracks = await self._generate_intelligent_tracks(
            genres=genres, mood=mood, tempo=tempo, decade=decade, max_songs=max_songs
        )
        
        if tracks:
            # Add tracks to playlist
            track_uris = [f"spotify:track:{track.id}" for track in tracks]
            await self._add_tracks_to_playlist(playlist_id, track_uris)
        
        return {
            "id": playlist_id,
            "name": name,
            "url": playlist_url,
            "songs_added": len(tracks),
            "total_duration": self._calculate_total_duration(tracks),
            "metadata": {
                "genres": genres,
                "mood": mood,
                "tempo": tempo,
                "decade": decade,
                "track_count": len(tracks)
            }
        }
    
    async def _generate_intelligent_tracks(self, genres: List[str] = None, mood: str = None,
                                          tempo: str = None, decade: str = None, max_songs: int = 50) -> List[SpotifyTrack]:
        """
        Generate intelligent track selection based on criteria
        """
        tracks = []
        
        # Map mood to audio features
        mood_features = self._map_mood_to_features(mood)
        
        # Map tempo to audio features
        tempo_features = self._map_tempo_to_features(tempo)
        
        # Combine features
        target_features = {**mood_features, **tempo_features}
        
        # Get seed genres (map to valid Spotify genres)
        seed_genres = self._map_to_spotify_genres(genres) if genres else ["pop", "rock", "hip-hop"]
        
        # Get recommendations
        recommendations = await self.get_recommendations(
            seed_genres=seed_genres[:5],
            limit=max_songs,
            **target_features
        )
        
        # Filter by decade if specified
        if decade and recommendations:
            decade_tracks = await self._filter_by_decade(recommendations, decade)
            tracks.extend(decade_tracks)
        else:
            tracks.extend(recommendations)
        
        # If we don't have enough tracks, get more with different seeds
        if len(tracks) < max_songs:
            additional_needed = max_songs - len(tracks)
            
            # Try with different genre combinations
            for genre_combo in self._get_genre_combinations(seed_genres):
                if len(tracks) >= max_songs:
                    break
                    
                additional_recs = await self.get_recommendations(
                    seed_genres=genre_combo,
                    limit=additional_needed,
                    **target_features
                )
                
                # Avoid duplicates
                existing_ids = {t.id for t in tracks}
                for track in additional_recs:
                    if track.id not in existing_ids and len(tracks) < max_songs:
                        tracks.append(track)
                        existing_ids.add(track.id)
        
        return tracks[:max_songs]
    
    def _map_mood_to_features(self, mood: str) -> Dict[str, float]:
        """Map mood to Spotify audio features"""
        mood_mappings = {
            "happy": {"target_valence": 0.8, "target_energy": 0.7},
            "sad": {"target_valence": 0.2, "target_energy": 0.3},
            "energetic": {"target_energy": 0.9, "target_danceability": 0.8},
            "calm": {"target_energy": 0.3, "target_valence": 0.5},
            "upbeat": {"target_valence": 0.7, "target_energy": 0.8, "target_danceability": 0.7},
            "chill": {"target_energy": 0.4, "target_valence": 0.6},
            "intense": {"target_energy": 0.9, "target_loudness": -5},
            "melancholic": {"target_valence": 0.3, "target_energy": 0.4},
            "party": {"target_danceability": 0.9, "target_energy": 0.8, "target_valence": 0.7}
        }
        
        return mood_mappings.get(mood.lower(), {}) if mood else {}
    
    def _map_tempo_to_features(self, tempo: str) -> Dict[str, float]:
        """Map tempo to Spotify audio features"""
        tempo_mappings = {
            "slow": {"target_tempo": 80},
            "medium": {"target_tempo": 120},
            "fast": {"target_tempo": 160}
        }
        
        return tempo_mappings.get(tempo.lower(), {}) if tempo else {}
    
    def _map_to_spotify_genres(self, genres: List[str]) -> List[str]:
        """Map generic genres to Spotify's genre seeds"""
        genre_mapping = {
            "pop": ["pop", "indie-pop"],
            "rock": ["rock", "alt-rock", "hard-rock"],
            "hip-hop": ["hip-hop", "rap"],
            "jazz": ["jazz"],
            "classical": ["classical"],
            "electronic": ["electronic", "techno", "house"],
            "country": ["country"],
            "folk": ["folk", "indie"],
            "blues": ["blues"],
            "r&b": ["r-n-b", "soul"],
            "metal": ["metal"],
            "punk": ["punk"],
            "reggae": ["reggae"],
            "funk": ["funk", "groove"],
            "dance": ["dance", "club"],
            "alternative": ["alternative", "indie"]
        }
        
        mapped_genres = []
        for genre in genres:
            if genre.lower() in genre_mapping:
                mapped_genres.extend(genre_mapping[genre.lower()])
            elif genre.lower() in self.genre_seeds:
                mapped_genres.append(genre.lower())
        
        return list(set(mapped_genres))
    
    def _get_genre_combinations(self, genres: List[str]) -> List[List[str]]:
        """Get different combinations of genres for variety"""
        combinations = []
        
        # Single genres
        for genre in genres:
            combinations.append([genre])
        
        # Pairs
        for i in range(len(genres)):
            for j in range(i + 1, len(genres)):
                combinations.append([genres[i], genres[j]])
        
        # All genres (up to 5 - Spotify's limit)
        if len(genres) <= 5:
            combinations.append(genres)
        
        return combinations
    
    async def _filter_by_decade(self, tracks: List[SpotifyTrack], decade: str) -> List[SpotifyTrack]:
        """Filter tracks by decade (placeholder - would need additional API calls)"""
        # This would require getting album information and release dates
        # For now, return all tracks - could be enhanced with additional API calls
        return tracks
    
    async def _add_tracks_to_playlist(self, playlist_id: str, track_uris: List[str]):
        """Add tracks to a playlist"""
        # Spotify allows up to 100 tracks per request
        for i in range(0, len(track_uris), 100):
            batch = track_uris[i:i+100]
            data = {"uris": batch}
            await self._make_request("POST", f"/playlists/{playlist_id}/tracks", json=data)
    
    def _calculate_total_duration(self, tracks: List[SpotifyTrack]) -> str:
        """Calculate total duration of tracks"""
        total_ms = sum(track.duration_ms for track in tracks)
        total_seconds = total_ms // 1000
        hours = total_seconds // 3600
        minutes = (total_seconds % 3600) // 60
        seconds = total_seconds % 60
        
        if hours > 0:
            return f"{hours}h {minutes}m {seconds}s"
        else:
            return f"{minutes}m {seconds}s"
    
    async def get_playlist_info(self, playlist_id: str) -> Dict[str, Any]:
        """Get information about a playlist"""
        response = await self._make_request("GET", f"/playlists/{playlist_id}")
        if not response:
            return {"error": "Playlist not found"}
        
        return {
            "id": response["id"],
            "name": response["name"],
            "description": response["description"],
            "public": response["public"],
            "collaborative": response["collaborative"],
            "url": response["external_urls"]["spotify"],
            "track_count": response["tracks"]["total"],
            "owner": response["owner"]["display_name"],
            "images": response["images"]
        }

# Example usage and testing
async def main():
    """Example usage of the Spotify client"""
    import os
    
    client_id = os.getenv("SPOTIFY_CLIENT_ID")
    client_secret = os.getenv("SPOTIFY_CLIENT_SECRET")
    
    if not client_id or not client_secret:
        print("Please set SPOTIFY_CLIENT_ID and SPOTIFY_CLIENT_SECRET environment variables")
        return
    
    async with SpotifyPlaylistClient(client_id, client_secret) as client:
        # Authenticate
        if await client.authenticate():
            print("Authentication successful!")
            
            # Search for songs
            tracks = await client.search_songs("happy songs", limit=10)
            print(f"Found {len(tracks)} tracks")
            
            # Get recommendations
            recommendations = await client.get_recommendations(
                seed_genres=["pop", "rock"],
                limit=20,
                target_valence=0.8
            )
            print(f"Got {len(recommendations)} recommendations")
            
        else:
            print("Authentication failed")

if __name__ == "__main__":
    asyncio.run(main()) 