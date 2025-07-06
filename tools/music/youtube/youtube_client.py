#!/usr/bin/env python3
"""
YouTube Playlist Client

A comprehensive client for interacting with YouTube's Data API to create playlists,
search for music videos, and manage user's YouTube music library. This client handles
authentication, rate limiting, and provides intelligent playlist generation capabilities.
"""

import asyncio
import json
import logging
import os
import re
import time
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass
from datetime import datetime, timedelta
import aiohttp
import google.auth.transport.requests
import google.oauth2.credentials
from google_auth_oauthlib.flow import InstalledAppFlow
from googleapiclient.discovery import build
from googleapiclient.errors import HttpError

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class YouTubeVideo:
    """Represents a YouTube video"""
    id: str
    title: str
    channel: str
    duration: str
    view_count: int
    published_at: str
    thumbnail_url: str
    description: str
    tags: List[str] = None
    category_id: str = None

@dataclass
class YouTubePlaylist:
    """Represents a YouTube playlist"""
    id: str
    title: str
    description: str
    privacy_status: str
    url: str
    video_count: int
    channel_title: str
    thumbnails: Dict[str, Any]
    published_at: str

class YouTubeAuthError(Exception):
    """Raised when authentication fails"""
    pass

class YouTubeAPIError(Exception):
    """Raised when API requests fail"""
    pass

class YouTubePlaylistClient:
    """
    YouTube Data API client for playlist management and music discovery
    """
    
    API_SERVICE_NAME = "youtube"
    API_VERSION = "v3"
    SCOPES = [
        "https://www.googleapis.com/auth/youtube.force-ssl",
        "https://www.googleapis.com/auth/youtube"
    ]
    
    # YouTube Music categories
    MUSIC_CATEGORIES = {
        "10": "Music",
        "1": "Film & Animation",
        "2": "Autos & Vehicles",
        "23": "Comedy",
        "24": "Entertainment",
        "25": "News & Politics",
        "26": "Howto & Style",
        "27": "Education",
        "28": "Science & Technology",
        "29": "Nonprofits & Activism",
        "30": "Movies",
        "31": "Anime/Animation",
        "32": "Action/Adventure",
        "33": "Classics",
        "34": "Comedy",
        "35": "Documentary",
        "36": "Drama",
        "37": "Family",
        "38": "Foreign",
        "39": "Horror",
        "40": "Sci-Fi/Fantasy",
        "41": "Thriller",
        "42": "Shorts",
        "43": "Shows",
        "44": "Trailers"
    }
    
    def __init__(self, api_key: str = None, credentials_file: str = None):
        """
        Initialize the YouTube client
        
        Args:
            api_key: YouTube Data API key (for read-only operations)
            credentials_file: OAuth 2.0 credentials file path (for write operations)
        """
        self.api_key = api_key
        self.credentials_file = credentials_file
        self.credentials = None
        self.service = None
        self.session = None
        
        # Rate limiting (YouTube API has generous limits)
        self.rate_limit_remaining = 10000
        self.rate_limit_reset = time.time() + 3600  # Reset hourly
        
        # Video duration cache
        self.duration_cache = {}
        
        # Common music search terms for better results
        self.music_keywords = [
            "official music video", "official video", "official audio",
            "lyric video", "lyrics", "live", "acoustic", "cover",
            "remix", "instrumental", "karaoke", "full song"
        ]
    
    async def __aenter__(self):
        """Async context manager entry"""
        self.session = aiohttp.ClientSession()
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit"""
        if self.session:
            await self.session.close()
    
    async def authenticate(self) -> bool:
        """
        Authenticate with YouTube API
        
        Returns:
            True if authentication successful, False otherwise
        """
        try:
            if self.credentials_file and os.path.exists(self.credentials_file):
                # Use OAuth flow for full access
                flow = InstalledAppFlow.from_client_secrets_file(
                    self.credentials_file, self.SCOPES
                )
                
                # Try to use local server for callback
                self.credentials = flow.run_local_server(port=0)
                
                # Build service with authenticated credentials
                self.service = build(
                    self.API_SERVICE_NAME, self.API_VERSION,
                    credentials=self.credentials
                )
                
            elif self.api_key:
                # Use API key for read-only operations
                self.service = build(
                    self.API_SERVICE_NAME, self.API_VERSION,
                    developerKey=self.api_key
                )
                
            else:
                raise YouTubeAuthError("No API key or credentials file provided")
            
            logger.info("YouTube API authentication successful")
            return True
            
        except Exception as e:
            logger.error(f"Authentication failed: {e}")
            return False
    
    async def search_songs(self, query: str, limit: int = 20, **filters) -> List[YouTubeVideo]:
        """
        Search for music videos on YouTube
        
        Args:
            query: Search query
            limit: Maximum number of results
            **filters: Additional filters (duration, upload_date, etc.)
            
        Returns:
            List of YouTubeVideo objects
        """
        if not self.service:
            raise YouTubeAPIError("YouTube service not initialized")
        
        # Enhance query for better music results
        enhanced_query = self._enhance_music_query(query)
        
        # Build search parameters
        search_params = {
            "part": "snippet",
            "q": enhanced_query,
            "type": "video",
            "maxResults": min(limit, 50),
            "order": "relevance",
            "videoCategoryId": "10"  # Music category
        }
        
        # Add filters
        if filters.get("duration"):
            search_params["videoDuration"] = filters["duration"]  # short, medium, long
        
        if filters.get("upload_date"):
            search_params["publishedAfter"] = filters["upload_date"]
        
        if filters.get("region"):
            search_params["regionCode"] = filters["region"]
        
        try:
            # Execute search
            search_response = self.service.search().list(**search_params).execute()
            
            videos = []
            video_ids = []
            
            # Extract video IDs and basic info
            for item in search_response.get("items", []):
                if item["id"]["kind"] == "youtube#video":
                    video_id = item["id"]["videoId"]
                    video_ids.append(video_id)
                    
                    # Create basic video object
                    video = YouTubeVideo(
                        id=video_id,
                        title=item["snippet"]["title"],
                        channel=item["snippet"]["channelTitle"],
                        duration="",  # Will be filled later
                        view_count=0,  # Will be filled later
                        published_at=item["snippet"]["publishedAt"],
                        thumbnail_url=item["snippet"]["thumbnails"]["high"]["url"],
                        description=item["snippet"]["description"],
                        tags=item["snippet"].get("tags", [])
                    )
                    videos.append(video)
            
            # Get detailed statistics and duration
            if video_ids:
                await self._populate_video_details(videos, video_ids)
            
            # Filter out non-music videos based on title/description
            music_videos = self._filter_music_videos(videos)
            
            return music_videos
            
        except HttpError as e:
            logger.error(f"YouTube search error: {e}")
            return []
        except Exception as e:
            logger.error(f"Unexpected error during search: {e}")
            return []
    
    def _enhance_music_query(self, query: str) -> str:
        """Enhance search query for better music results"""
        # Add music-specific terms if not already present
        music_terms = ["song", "music", "audio", "video", "official"]
        
        query_lower = query.lower()
        has_music_term = any(term in query_lower for term in music_terms)
        
        if not has_music_term:
            # Add general music term
            query += " music"
        
        # Remove common non-music terms
        non_music_terms = ["tutorial", "how to", "review", "reaction", "gameplay"]
        for term in non_music_terms:
            query = re.sub(rf'\b{term}\b', '', query, flags=re.IGNORECASE)
        
        return query.strip()
    
    async def _populate_video_details(self, videos: List[YouTubeVideo], video_ids: List[str]):
        """Populate video details like duration and view count"""
        try:
            # Get video details in batches (YouTube API supports up to 50 IDs per request)
            for i in range(0, len(video_ids), 50):
                batch_ids = video_ids[i:i+50]
                
                video_response = self.service.videos().list(
                    part="statistics,contentDetails",
                    id=",".join(batch_ids)
                ).execute()
                
                # Map details to video objects
                for j, item in enumerate(video_response.get("items", [])):
                    video_index = i + j
                    if video_index < len(videos):
                        videos[video_index].duration = self._parse_duration(
                            item["contentDetails"]["duration"]
                        )
                        videos[video_index].view_count = int(
                            item["statistics"].get("viewCount", 0)
                        )
                        
        except Exception as e:
            logger.warning(f"Could not populate video details: {e}")
    
    def _parse_duration(self, duration_str: str) -> str:
        """Parse ISO 8601 duration to readable format"""
        if duration_str in self.duration_cache:
            return self.duration_cache[duration_str]
        
        # Parse PT1H2M3S format
        pattern = r'PT(?:(\d+)H)?(?:(\d+)M)?(?:(\d+)S)?'
        match = re.match(pattern, duration_str)
        
        if not match:
            return "Unknown"
        
        hours = int(match.group(1) or 0)
        minutes = int(match.group(2) or 0)
        seconds = int(match.group(3) or 0)
        
        if hours > 0:
            formatted = f"{hours}:{minutes:02d}:{seconds:02d}"
        else:
            formatted = f"{minutes}:{seconds:02d}"
        
        self.duration_cache[duration_str] = formatted
        return formatted
    
    def _filter_music_videos(self, videos: List[YouTubeVideo]) -> List[YouTubeVideo]:
        """Filter videos to prioritize music content"""
        music_videos = []
        
        for video in videos:
            # Score based on music relevance
            score = 0
            title_lower = video.title.lower()
            description_lower = video.description.lower()
            
            # Positive indicators
            music_indicators = [
                "official music video", "official video", "official audio",
                "lyric video", "lyrics", "full song", "single",
                "album", "track", "music video", "official", "audio"
            ]
            
            for indicator in music_indicators:
                if indicator in title_lower:
                    score += 3
                elif indicator in description_lower:
                    score += 1
            
            # Negative indicators
            negative_indicators = [
                "reaction", "review", "cover", "tutorial", "how to",
                "gameplay", "unboxing", "vlog", "compilation"
            ]
            
            for indicator in negative_indicators:
                if indicator in title_lower:
                    score -= 2
            
            # Channel name indicators
            if any(term in video.channel.lower() for term in ["vevo", "records", "music"]):
                score += 2
            
            # Duration filter (music videos are typically 2-8 minutes)
            duration_parts = video.duration.split(":")
            if len(duration_parts) == 2:  # MM:SS format
                minutes = int(duration_parts[0])
                if 2 <= minutes <= 8:
                    score += 1
                elif minutes > 10:
                    score -= 1
            
            # View count factor (higher views often indicate official content)
            if video.view_count > 1000000:  # 1M+ views
                score += 1
            
            # Only include videos with positive score
            if score > 0:
                music_videos.append(video)
        
        # Sort by score (implicit in filtering logic) and return
        return sorted(music_videos, key=lambda v: v.view_count, reverse=True)
    
    async def create_playlist(self, name: str, description: str = "", public: bool = True,
                             genres: List[str] = None, mood: str = None, tempo: str = None,
                             decade: str = None, max_songs: int = 50) -> Dict[str, Any]:
        """
        Create a new YouTube playlist with intelligent song selection
        
        Args:
            name: Playlist name
            description: Playlist description
            public: Whether playlist should be public
            genres: Preferred genres
            mood: Mood for the playlist
            tempo: Tempo preference
            decade: Decade preference
            max_songs: Maximum number of songs to add
            
        Returns:
            Dictionary with playlist information
        """
        if not self.service or not self.credentials:
            raise YouTubeAPIError("User authentication required to create playlists")
        
        # Determine privacy status
        privacy_status = "public" if public else "private"
        
        # Create playlist
        playlist_data = {
            "snippet": {
                "title": name,
                "description": description,
                "defaultLanguage": "en",
                "tags": ["music", "playlist", "toka-generated"]
            },
            "status": {
                "privacyStatus": privacy_status
            }
        }
        
        try:
            playlist_response = self.service.playlists().insert(
                part="snippet,status",
                body=playlist_data
            ).execute()
            
            playlist_id = playlist_response["id"]
            playlist_url = f"https://www.youtube.com/playlist?list={playlist_id}"
            
            # Generate intelligent video selection
            videos = await self._generate_intelligent_videos(
                genres=genres, mood=mood, tempo=tempo, decade=decade, max_songs=max_songs
            )
            
            # Add videos to playlist
            videos_added = 0
            if videos:
                videos_added = await self._add_videos_to_playlist(playlist_id, videos)
            
            return {
                "id": playlist_id,
                "name": name,
                "url": playlist_url,
                "songs_added": videos_added,
                "total_duration": self._calculate_total_duration(videos),
                "metadata": {
                    "genres": genres,
                    "mood": mood,
                    "tempo": tempo,
                    "decade": decade,
                    "video_count": videos_added,
                    "privacy_status": privacy_status
                }
            }
            
        except HttpError as e:
            logger.error(f"Error creating YouTube playlist: {e}")
            raise YouTubeAPIError(f"Failed to create playlist: {e}")
    
    async def _generate_intelligent_videos(self, genres: List[str] = None, mood: str = None,
                                          tempo: str = None, decade: str = None, max_songs: int = 50) -> List[YouTubeVideo]:
        """
        Generate intelligent video selection based on criteria
        """
        videos = []
        search_queries = []
        
        # Build search queries based on criteria
        base_terms = []
        
        if genres:
            base_terms.extend(genres)
        
        if mood:
            mood_terms = self._map_mood_to_terms(mood)
            base_terms.extend(mood_terms)
        
        if tempo:
            tempo_terms = self._map_tempo_to_terms(tempo)
            base_terms.extend(tempo_terms)
        
        if decade:
            decade_terms = self._map_decade_to_terms(decade)
            base_terms.extend(decade_terms)
        
        # If no specific criteria, use popular music terms
        if not base_terms:
            base_terms = ["popular music", "hit songs", "best songs"]
        
        # Generate search queries
        for term in base_terms[:5]:  # Limit to avoid too many API calls
            search_queries.append(f"{term} music official video")
        
        # Add some variety with different combinations
        if len(base_terms) > 1:
            search_queries.append(f"{base_terms[0]} {base_terms[1]} playlist")
        
        # Search for videos
        all_videos = []
        video_ids_seen = set()
        
        for query in search_queries:
            try:
                query_videos = await self.search_songs(
                    query, 
                    limit=max_songs // len(search_queries) + 5
                )
                
                # Avoid duplicates
                for video in query_videos:
                    if video.id not in video_ids_seen:
                        all_videos.append(video)
                        video_ids_seen.add(video.id)
                        
                        if len(all_videos) >= max_songs:
                            break
                            
            except Exception as e:
                logger.warning(f"Error searching for '{query}': {e}")
                continue
            
            if len(all_videos) >= max_songs:
                break
        
        # Sort by relevance (view count and recency)
        all_videos.sort(key=lambda v: (v.view_count, v.published_at), reverse=True)
        
        return all_videos[:max_songs]
    
    def _map_mood_to_terms(self, mood: str) -> List[str]:
        """Map mood to search terms"""
        mood_mappings = {
            "happy": ["happy", "upbeat", "positive", "cheerful", "joyful"],
            "sad": ["sad", "melancholic", "emotional", "heartbreak", "ballad"],
            "energetic": ["energetic", "pump up", "workout", "high energy", "dance"],
            "calm": ["calm", "relaxing", "peaceful", "chill", "ambient"],
            "upbeat": ["upbeat", "positive", "feel good", "party", "celebration"],
            "chill": ["chill", "laid back", "relaxed", "mellow", "smooth"],
            "intense": ["intense", "powerful", "epic", "dramatic", "strong"],
            "melancholic": ["melancholic", "nostalgic", "emotional", "deep", "reflective"],
            "party": ["party", "celebration", "dance", "club", "fun"]
        }
        
        return mood_mappings.get(mood.lower(), [mood]) if mood else []
    
    def _map_tempo_to_terms(self, tempo: str) -> List[str]:
        """Map tempo to search terms"""
        tempo_mappings = {
            "slow": ["slow", "ballad", "emotional", "acoustic"],
            "medium": ["medium tempo", "mid-tempo", "steady"],
            "fast": ["fast", "upbeat", "energetic", "dance", "electronic"]
        }
        
        return tempo_mappings.get(tempo.lower(), [tempo]) if tempo else []
    
    def _map_decade_to_terms(self, decade: str) -> List[str]:
        """Map decade to search terms"""
        decade_mappings = {
            "2020s": ["2020s", "2021", "2022", "2023", "2024", "recent"],
            "2010s": ["2010s", "2010", "2015", "2019", "hits"],
            "2000s": ["2000s", "2000", "2005", "2009", "early 2000s"],
            "1990s": ["1990s", "90s", "1990", "1995", "1999", "90s hits"],
            "1980s": ["1980s", "80s", "1980", "1985", "1989", "80s hits"],
            "1970s": ["1970s", "70s", "1970", "1975", "1979", "70s hits"],
            "1960s": ["1960s", "60s", "1960", "1965", "1969", "60s hits"]
        }
        
        # Handle various decade formats
        decade_clean = decade.lower().replace("'s", "s").replace("s", "s")
        
        return decade_mappings.get(decade_clean, [decade]) if decade else []
    
    async def _add_videos_to_playlist(self, playlist_id: str, videos: List[YouTubeVideo]) -> int:
        """Add videos to a playlist"""
        videos_added = 0
        
        for video in videos:
            try:
                playlist_item = {
                    "snippet": {
                        "playlistId": playlist_id,
                        "resourceId": {
                            "kind": "youtube#video",
                            "videoId": video.id
                        }
                    }
                }
                
                self.service.playlistItems().insert(
                    part="snippet",
                    body=playlist_item
                ).execute()
                
                videos_added += 1
                
                # Small delay to respect rate limits
                await asyncio.sleep(0.1)
                
            except HttpError as e:
                logger.warning(f"Could not add video {video.id} to playlist: {e}")
                continue
            except Exception as e:
                logger.warning(f"Unexpected error adding video {video.id}: {e}")
                continue
        
        return videos_added
    
    def _calculate_total_duration(self, videos: List[YouTubeVideo]) -> str:
        """Calculate total duration of videos"""
        total_seconds = 0
        
        for video in videos:
            if video.duration:
                # Parse MM:SS or H:MM:SS format
                duration_parts = video.duration.split(":")
                if len(duration_parts) == 2:
                    minutes, seconds = map(int, duration_parts)
                    total_seconds += minutes * 60 + seconds
                elif len(duration_parts) == 3:
                    hours, minutes, seconds = map(int, duration_parts)
                    total_seconds += hours * 3600 + minutes * 60 + seconds
        
        # Format total duration
        hours = total_seconds // 3600
        minutes = (total_seconds % 3600) // 60
        seconds = total_seconds % 60
        
        if hours > 0:
            return f"{hours}h {minutes}m {seconds}s"
        else:
            return f"{minutes}m {seconds}s"
    
    async def get_playlist_info(self, playlist_id: str) -> Dict[str, Any]:
        """Get information about a playlist"""
        try:
            playlist_response = self.service.playlists().list(
                part="snippet,status,contentDetails",
                id=playlist_id
            ).execute()
            
            if not playlist_response.get("items"):
                return {"error": "Playlist not found"}
            
            playlist = playlist_response["items"][0]
            
            return {
                "id": playlist["id"],
                "title": playlist["snippet"]["title"],
                "description": playlist["snippet"]["description"],
                "privacy_status": playlist["status"]["privacyStatus"],
                "url": f"https://www.youtube.com/playlist?list={playlist['id']}",
                "video_count": playlist["contentDetails"]["itemCount"],
                "channel_title": playlist["snippet"]["channelTitle"],
                "thumbnails": playlist["snippet"]["thumbnails"],
                "published_at": playlist["snippet"]["publishedAt"]
            }
            
        except HttpError as e:
            logger.error(f"Error getting playlist info: {e}")
            return {"error": str(e)}
    
    async def get_trending_music(self, region: str = "US", limit: int = 20) -> List[YouTubeVideo]:
        """Get trending music videos"""
        try:
            # Get trending videos in music category
            trending_response = self.service.videos().list(
                part="snippet,statistics,contentDetails",
                chart="mostPopular",
                regionCode=region,
                videoCategoryId="10",  # Music category
                maxResults=limit
            ).execute()
            
            videos = []
            for item in trending_response.get("items", []):
                video = YouTubeVideo(
                    id=item["id"],
                    title=item["snippet"]["title"],
                    channel=item["snippet"]["channelTitle"],
                    duration=self._parse_duration(item["contentDetails"]["duration"]),
                    view_count=int(item["statistics"].get("viewCount", 0)),
                    published_at=item["snippet"]["publishedAt"],
                    thumbnail_url=item["snippet"]["thumbnails"]["high"]["url"],
                    description=item["snippet"]["description"],
                    tags=item["snippet"].get("tags", []),
                    category_id=item["snippet"]["categoryId"]
                )
                videos.append(video)
            
            return videos
            
        except HttpError as e:
            logger.error(f"Error getting trending music: {e}")
            return []

# Example usage and testing
async def main():
    """Example usage of the YouTube client"""
    import os
    
    api_key = os.getenv("YOUTUBE_API_KEY")
    credentials_file = os.getenv("YOUTUBE_CREDENTIALS_FILE")
    
    if not api_key and not credentials_file:
        print("Please set YOUTUBE_API_KEY or YOUTUBE_CREDENTIALS_FILE environment variables")
        return
    
    async with YouTubePlaylistClient(api_key, credentials_file) as client:
        # Authenticate
        if await client.authenticate():
            print("Authentication successful!")
            
            # Search for videos
            videos = await client.search_songs("happy music", limit=10)
            print(f"Found {len(videos)} videos")
            
            # Get trending music
            trending = await client.get_trending_music(limit=10)
            print(f"Got {len(trending)} trending videos")
            
        else:
            print("Authentication failed")

if __name__ == "__main__":
    asyncio.run(main()) 