#!/usr/bin/env python3
"""
Music Language Processor

A sophisticated natural language processing component that understands music-related
requests and extracts musical intent, genres, moods, tempo, and other attributes
from natural language descriptions.
"""

import asyncio
import json
import logging
import re
from typing import Dict, List, Optional, Any, Tuple, Set
from dataclasses import dataclass, asdict
from enum import Enum
import spacy
from collections import defaultdict

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class MusicAttribute(Enum):
    """Types of music attributes we can extract"""
    GENRE = "genre"
    MOOD = "mood"
    TEMPO = "tempo"
    DECADE = "decade"
    ARTIST = "artist"
    SONG = "song"
    ACTIVITY = "activity"
    INSTRUMENT = "instrument"
    ENERGY = "energy"
    LANGUAGE = "language"

@dataclass
class MusicIntent:
    """Represents extracted music intent from natural language"""
    genres: List[str]
    moods: List[str]
    tempo: Optional[str]
    decade: Optional[str]
    artists: List[str]
    songs: List[str]
    activities: List[str]
    instruments: List[str]
    energy_level: Optional[str]
    language: Optional[str]
    confidence: float
    raw_text: str
    extracted_entities: Dict[str, List[str]]

class MusicLanguageProcessor:
    """
    Advanced NLP processor for music-related natural language understanding
    """
    
    def __init__(self, model_name: str = "en_core_web_sm", use_gpu: bool = False):
        """
        Initialize the music language processor
        
        Args:
            model_name: SpaCy model name to use
            use_gpu: Whether to use GPU acceleration
        """
        self.model_name = model_name
        self.use_gpu = use_gpu
        self.nlp = None
        
        # Music vocabulary and patterns
        self.music_vocabularies = self._build_music_vocabularies()
        self.pattern_rules = self._build_pattern_rules()
        
        # Initialize NLP components
        self._initialize_nlp()
    
    def _initialize_nlp(self):
        """Initialize spaCy NLP pipeline"""
        try:
            self.nlp = spacy.load(self.model_name)
            if self.use_gpu:
                spacy.prefer_gpu()
            
            # Add custom components for music understanding
            self._add_music_components()
            
            logger.info(f"NLP pipeline initialized with model: {self.model_name}")
            
        except OSError:
            logger.warning(f"SpaCy model {self.model_name} not found, using basic processing")
            self.nlp = None
    
    def _add_music_components(self):
        """Add custom NLP components for music understanding"""
        # Add entity ruler for music-specific entities
        if "entity_ruler" not in self.nlp.pipe_names:
            ruler = self.nlp.add_pipe("entity_ruler", before="ner")
            patterns = self._create_entity_patterns()
            ruler.add_patterns(patterns)
    
    def _create_entity_patterns(self) -> List[Dict[str, Any]]:
        """Create patterns for music entity recognition"""
        patterns = []
        
        # Genre patterns
        for genre in self.music_vocabularies[MusicAttribute.GENRE]:
            patterns.append({
                "label": "MUSIC_GENRE",
                "pattern": [{"LOWER": genre.lower()}]
            })
        
        # Mood patterns
        for mood in self.music_vocabularies[MusicAttribute.MOOD]:
            patterns.append({
                "label": "MUSIC_MOOD",
                "pattern": [{"LOWER": mood.lower()}]
            })
        
        # Decade patterns
        for decade in self.music_vocabularies[MusicAttribute.DECADE]:
            patterns.append({
                "label": "MUSIC_DECADE",
                "pattern": [{"LOWER": decade.lower()}]
            })
        
        # Activity patterns
        for activity in self.music_vocabularies[MusicAttribute.ACTIVITY]:
            patterns.append({
                "label": "MUSIC_ACTIVITY",
                "pattern": [{"LOWER": activity.lower()}]
            })
        
        return patterns
    
    def _build_music_vocabularies(self) -> Dict[MusicAttribute, List[str]]:
        """Build comprehensive music vocabularies"""
        vocabularies = {
            MusicAttribute.GENRE: [
                # Main genres
                "pop", "rock", "hip-hop", "rap", "jazz", "classical", "blues", "country",
                "electronic", "edm", "house", "techno", "trance", "dubstep", "drum and bass",
                "reggae", "ska", "punk", "metal", "heavy metal", "death metal", "black metal",
                "indie", "alternative", "folk", "acoustic", "ambient", "new age", "world",
                "latin", "salsa", "reggaeton", "bachata", "merengue", "cumbia", "tango",
                "r&b", "soul", "funk", "disco", "motown", "gospel", "spiritual",
                "grunge", "emo", "hardcore", "post-rock", "shoegaze", "britpop",
                "trap", "drill", "mumble rap", "conscious rap", "old school hip hop",
                "progressive rock", "psychedelic rock", "classic rock", "hard rock",
                "experimental", "avant-garde", "minimalist", "contemporary classical",
                "bossa nova", "samba", "fado", "flamenco", "celtic", "traditional",
                "k-pop", "j-pop", "mandopop", "bollywood", "afrobeat", "highlife",
                "chillout", "lo-fi", "synthwave", "vaporwave", "downtempo",
                "garage", "grime", "uk drill", "afroswing", "dancehall", "soca"
            ],
            
            MusicAttribute.MOOD: [
                # Emotional moods
                "happy", "sad", "melancholic", "joyful", "depressed", "euphoric",
                "angry", "peaceful", "anxious", "calm", "serene", "contemplative",
                "nostalgic", "romantic", "passionate", "tender", "loving", "heartbroken",
                "triumphant", "victorious", "defeated", "hopeful", "hopeless",
                "mysterious", "dark", "bright", "light", "heavy", "uplifting",
                "inspiring", "motivational", "empowering", "vulnerable", "confident",
                "rebellious", "defiant", "submissive", "aggressive", "gentle",
                "playful", "serious", "humorous", "ironic", "sarcastic",
                "dreamy", "surreal", "realistic", "abstract", "concrete",
                "spiritual", "religious", "secular", "political", "social",
                "introspective", "extroverted", "introverted", "social", "antisocial"
            ],
            
            MusicAttribute.TEMPO: [
                "slow", "fast", "medium", "quick", "rapid", "sluggish", "moderate",
                "allegro", "andante", "adagio", "presto", "largo", "vivace",
                "ballad tempo", "dance tempo", "march tempo", "waltz tempo",
                "uptempo", "downtempo", "mid-tempo", "double-time", "half-time",
                "accelerating", "decelerating", "steady", "variable", "rubato"
            ],
            
            MusicAttribute.DECADE: [
                "1940s", "1950s", "1960s", "1970s", "1980s", "1990s", "2000s", "2010s", "2020s",
                "40s", "50s", "60s", "70s", "80s", "90s", "00s", "10s", "20s",
                "nineteen forties", "nineteen fifties", "nineteen sixties", "nineteen seventies",
                "nineteen eighties", "nineteen nineties", "two thousands", "twenty tens", "twenty twenties",
                "classic", "vintage", "retro", "modern", "contemporary", "current", "recent",
                "old school", "new school", "golden age", "silver age", "bronze age"
            ],
            
            MusicAttribute.ACTIVITY: [
                "workout", "exercise", "running", "jogging", "cycling", "swimming",
                "studying", "working", "concentration", "focus", "meditation",
                "relaxation", "sleeping", "waking up", "morning", "evening",
                "party", "dancing", "clubbing", "celebration", "wedding",
                "driving", "road trip", "travel", "commuting", "walking",
                "cooking", "cleaning", "housework", "gardening", "reading",
                "gaming", "movie watching", "background music", "ambient",
                "shower", "bath", "spa", "massage", "yoga", "stretching",
                "date night", "romantic dinner", "intimate", "chill", "hangout",
                "beach", "summer", "winter", "spring", "autumn", "seasonal",
                "rain", "sunny", "cloudy", "stormy", "weather"
            ],
            
            MusicAttribute.INSTRUMENT: [
                "guitar", "acoustic guitar", "electric guitar", "bass guitar", "bass",
                "piano", "keyboard", "synthesizer", "synth", "organ", "harpsichord",
                "drums", "percussion", "drum kit", "snare", "kick", "cymbals",
                "violin", "viola", "cello", "double bass", "string quartet",
                "trumpet", "trombone", "french horn", "tuba", "brass section",
                "saxophone", "clarinet", "flute", "oboe", "bassoon", "woodwinds",
                "harmonica", "accordion", "banjo", "mandolin", "ukulele",
                "harp", "xylophone", "marimba", "vibraphone", "glockenspiel",
                "voice", "vocals", "singing", "choir", "harmony", "acapella",
                "electronic", "computer", "sampler", "drum machine", "vocoder",
                "didgeridoo", "bagpipes", "sitar", "tabla", "gamelan", "ethnic"
            ],
            
            MusicAttribute.ENERGY: [
                "high energy", "low energy", "medium energy", "energetic", "mellow",
                "intense", "laid-back", "aggressive", "gentle", "powerful", "subtle",
                "explosive", "calm", "wild", "tame", "fierce", "peaceful",
                "driving", "flowing", "pulsing", "steady", "erratic", "smooth",
                "rough", "polished", "raw", "refined", "organic", "synthetic",
                "warm", "cold", "hot", "cool", "burning", "freezing", "tepid"
            ],
            
            MusicAttribute.LANGUAGE: [
                "english", "spanish", "french", "german", "italian", "portuguese",
                "chinese", "japanese", "korean", "arabic", "hindi", "russian",
                "dutch", "swedish", "norwegian", "danish", "finnish", "polish",
                "czech", "hungarian", "turkish", "greek", "hebrew", "thai",
                "vietnamese", "indonesian", "malay", "tagalog", "swahili",
                "instrumental", "wordless", "vocal", "sung", "spoken", "rapped"
            ]
        }
        
        return vocabularies
    
    def _build_pattern_rules(self) -> List[Dict[str, Any]]:
        """Build pattern matching rules for music understanding"""
        rules = [
            # Genre patterns
            {
                "pattern": r"(?:play|want|like|love|need|give me|find) (?:some )?(\w+) (?:music|songs|tracks)",
                "attribute": MusicAttribute.GENRE,
                "group": 1
            },
            {
                "pattern": r"(?:I'm in the mood for|feeling like|want to listen to) (\w+)",
                "attribute": MusicAttribute.GENRE,
                "group": 1
            },
            
            # Mood patterns
            {
                "pattern": r"(?:I'm feeling|I feel|feeling|mood is) (\w+)",
                "attribute": MusicAttribute.MOOD,
                "group": 1
            },
            {
                "pattern": r"(?:make me feel|want to feel|need to feel) (\w+)",
                "attribute": MusicAttribute.MOOD,
                "group": 1
            },
            
            # Decade patterns
            {
                "pattern": r"(?:from the|music from|songs from) (\d{4}s|\d{2}s)",
                "attribute": MusicAttribute.DECADE,
                "group": 1
            },
            {
                "pattern": r"(\d{4}s|\d{2}s) (?:music|songs|hits|classics)",
                "attribute": MusicAttribute.DECADE,
                "group": 1
            },
            
            # Activity patterns
            {
                "pattern": r"(?:for|while|when|during) (\w+)",
                "attribute": MusicAttribute.ACTIVITY,
                "group": 1
            },
            {
                "pattern": r"(?:to|for) (\w+) (?:to|with)",
                "attribute": MusicAttribute.ACTIVITY,
                "group": 1
            },
            
            # Tempo patterns
            {
                "pattern": r"(?:slow|fast|quick|rapid|uptempo|downtempo|moderate) (?:music|songs|tempo|beat)",
                "attribute": MusicAttribute.TEMPO,
                "group": 0
            },
            
            # Energy patterns
            {
                "pattern": r"(?:high|low|medium) energy (?:music|songs)",
                "attribute": MusicAttribute.ENERGY,
                "group": 0
            },
            
            # Artist patterns
            {
                "pattern": r"(?:by|from|artist) ([A-Z][\w\s]+)",
                "attribute": MusicAttribute.ARTIST,
                "group": 1
            },
            
            # Song patterns
            {
                "pattern": r"(?:song|track) (?:called|named|titled) [\"']([^\"']+)[\"']",
                "attribute": MusicAttribute.SONG,
                "group": 1
            }
        ]
        
        return rules
    
    async def process_request(self, text: str) -> Dict[str, Any]:
        """
        Process a natural language music request
        
        Args:
            text: Natural language text describing music preferences
            
        Returns:
            Dictionary containing extracted music intent
        """
        logger.info(f"Processing music request: {text}")
        
        # Clean and normalize text
        cleaned_text = self._clean_text(text)
        
        # Extract entities and attributes
        entities = {}
        confidence = 0.0
        
        if self.nlp:
            # Use spaCy for advanced processing
            entities, confidence = await self._extract_with_spacy(cleaned_text)
        else:
            # Fallback to pattern matching
            entities, confidence = await self._extract_with_patterns(cleaned_text)
        
        # Post-process and validate entities
        validated_entities = self._validate_entities(entities)
        
        # Build music intent
        intent = MusicIntent(
            genres=validated_entities.get(MusicAttribute.GENRE, []),
            moods=validated_entities.get(MusicAttribute.MOOD, []),
            tempo=validated_entities.get(MusicAttribute.TEMPO, [None])[0],
            decade=validated_entities.get(MusicAttribute.DECADE, [None])[0],
            artists=validated_entities.get(MusicAttribute.ARTIST, []),
            songs=validated_entities.get(MusicAttribute.SONG, []),
            activities=validated_entities.get(MusicAttribute.ACTIVITY, []),
            instruments=validated_entities.get(MusicAttribute.INSTRUMENT, []),
            energy_level=validated_entities.get(MusicAttribute.ENERGY, [None])[0],
            language=validated_entities.get(MusicAttribute.LANGUAGE, [None])[0],
            confidence=confidence,
            raw_text=text,
            extracted_entities=validated_entities
        )
        
        # Convert to dictionary for return
        result = asdict(intent)
        
        # Add some intelligent defaults if nothing was extracted
        if not any(result[key] for key in ['genres', 'moods', 'artists', 'songs'] if isinstance(result[key], list)):
            result.update(self._generate_default_intent(cleaned_text))
        
        logger.info(f"Extracted music intent: {result}")
        return result
    
    def _clean_text(self, text: str) -> str:
        """Clean and normalize input text"""
        # Remove extra whitespace
        text = re.sub(r'\s+', ' ', text.strip())
        
        # Handle common contractions
        contractions = {
            "i'm": "i am", "you're": "you are", "we're": "we are",
            "they're": "they are", "it's": "it is", "that's": "that is",
            "what's": "what is", "who's": "who is", "where's": "where is",
            "when's": "when is", "why's": "why is", "how's": "how is",
            "can't": "cannot", "won't": "will not", "don't": "do not",
            "doesn't": "does not", "didn't": "did not", "wouldn't": "would not",
            "shouldn't": "should not", "couldn't": "could not", "haven't": "have not",
            "hasn't": "has not", "hadn't": "had not", "isn't": "is not",
            "aren't": "are not", "wasn't": "was not", "weren't": "were not"
        }
        
        for contraction, expansion in contractions.items():
            text = re.sub(rf'\b{contraction}\b', expansion, text, flags=re.IGNORECASE)
        
        return text
    
    async def _extract_with_spacy(self, text: str) -> Tuple[Dict[str, List[str]], float]:
        """Extract entities using spaCy NLP"""
        doc = self.nlp(text)
        entities = defaultdict(list)
        confidence = 0.0
        entity_count = 0
        
        # Extract named entities
        for ent in doc.ents:
            if ent.label_ == "MUSIC_GENRE":
                entities[MusicAttribute.GENRE].append(ent.text)
                confidence += 0.9
                entity_count += 1
            elif ent.label_ == "MUSIC_MOOD":
                entities[MusicAttribute.MOOD].append(ent.text)
                confidence += 0.9
                entity_count += 1
            elif ent.label_ == "MUSIC_DECADE":
                entities[MusicAttribute.DECADE].append(ent.text)
                confidence += 0.9
                entity_count += 1
            elif ent.label_ == "MUSIC_ACTIVITY":
                entities[MusicAttribute.ACTIVITY].append(ent.text)
                confidence += 0.8
                entity_count += 1
            elif ent.label_ == "PERSON":
                # Might be an artist
                entities[MusicAttribute.ARTIST].append(ent.text)
                confidence += 0.6
                entity_count += 1
        
        # Extract using vocabulary matching
        text_lower = text.lower()
        for attribute, vocabulary in self.music_vocabularies.items():
            for term in vocabulary:
                if term.lower() in text_lower:
                    entities[attribute].append(term)
                    confidence += 0.7
                    entity_count += 1
        
        # Extract using pattern matching
        pattern_entities, pattern_confidence = await self._extract_with_patterns(text)
        for attribute, terms in pattern_entities.items():
            entities[attribute].extend(terms)
            confidence += pattern_confidence
            entity_count += len(terms)
        
        # Normalize confidence
        if entity_count > 0:
            confidence = min(confidence / entity_count, 1.0)
        
        return dict(entities), confidence
    
    async def _extract_with_patterns(self, text: str) -> Tuple[Dict[str, List[str]], float]:
        """Extract entities using pattern matching"""
        entities = defaultdict(list)
        confidence = 0.0
        match_count = 0
        
        for rule in self.pattern_rules:
            pattern = rule["pattern"]
            attribute = rule["attribute"]
            group = rule["group"]
            
            matches = re.finditer(pattern, text, re.IGNORECASE)
            for match in matches:
                if group == 0:
                    extracted = match.group(0)
                else:
                    extracted = match.group(group)
                
                entities[attribute].append(extracted.strip())
                confidence += 0.8
                match_count += 1
        
        # Normalize confidence
        if match_count > 0:
            confidence = min(confidence / match_count, 1.0)
        
        return dict(entities), confidence
    
    def _validate_entities(self, entities: Dict[str, List[str]]) -> Dict[str, List[str]]:
        """Validate and clean extracted entities"""
        validated = {}
        
        for attribute, terms in entities.items():
            # Remove duplicates and empty strings
            unique_terms = list(set(term.strip() for term in terms if term.strip()))
            
            # Validate against vocabulary
            if attribute in self.music_vocabularies:
                vocabulary = [term.lower() for term in self.music_vocabularies[attribute]]
                validated_terms = []
                
                for term in unique_terms:
                    if term.lower() in vocabulary:
                        validated_terms.append(term)
                    else:
                        # Try fuzzy matching
                        fuzzy_match = self._fuzzy_match(term, vocabulary)
                        if fuzzy_match:
                            validated_terms.append(fuzzy_match)
                
                validated[attribute] = validated_terms
            else:
                validated[attribute] = unique_terms
        
        return validated
    
    def _fuzzy_match(self, term: str, vocabulary: List[str], threshold: float = 0.8) -> Optional[str]:
        """Perform fuzzy matching against vocabulary"""
        term_lower = term.lower()
        
        # Simple fuzzy matching based on substring and character overlap
        for vocab_term in vocabulary:
            if term_lower in vocab_term or vocab_term in term_lower:
                return vocab_term
            
            # Check character overlap
            overlap = len(set(term_lower) & set(vocab_term))
            if overlap / len(term_lower) >= threshold:
                return vocab_term
        
        return None
    
    def _generate_default_intent(self, text: str) -> Dict[str, Any]:
        """Generate default intent when no specific entities are found"""
        defaults = {
            "genres": ["pop"],
            "moods": ["happy"],
            "tempo": "medium",
            "decade": "2020s",
            "artists": [],
            "songs": [],
            "activities": [],
            "instruments": [],
            "energy_level": "medium",
            "language": "english",
            "confidence": 0.1
        }
        
        # Try to infer from common keywords
        text_lower = text.lower()
        
        if any(word in text_lower for word in ["party", "dance", "club", "fun"]):
            defaults["genres"] = ["pop", "dance"]
            defaults["moods"] = ["energetic", "happy"]
            defaults["energy_level"] = "high"
        elif any(word in text_lower for word in ["relax", "chill", "calm", "peaceful"]):
            defaults["genres"] = ["ambient", "chill"]
            defaults["moods"] = ["calm", "peaceful"]
            defaults["energy_level"] = "low"
        elif any(word in text_lower for word in ["work", "study", "focus", "concentrate"]):
            defaults["genres"] = ["ambient", "classical"]
            defaults["moods"] = ["calm", "focused"]
            defaults["activities"] = ["studying", "working"]
        elif any(word in text_lower for word in ["workout", "exercise", "gym", "run"]):
            defaults["genres"] = ["electronic", "hip-hop"]
            defaults["moods"] = ["energetic", "motivational"]
            defaults["activities"] = ["workout", "exercise"]
            defaults["energy_level"] = "high"
        
        return defaults
    
    async def analyze_sentiment(self, text: str) -> Dict[str, Any]:
        """Analyze sentiment of music request"""
        if not self.nlp:
            return {"sentiment": "neutral", "confidence": 0.0}
        
        doc = self.nlp(text)
        
        # Simple sentiment analysis based on sentiment words
        positive_words = ["love", "like", "enjoy", "happy", "good", "great", "awesome", "amazing"]
        negative_words = ["hate", "dislike", "sad", "bad", "terrible", "awful", "horrible"]
        
        positive_count = sum(1 for token in doc if token.text.lower() in positive_words)
        negative_count = sum(1 for token in doc if token.text.lower() in negative_words)
        
        if positive_count > negative_count:
            sentiment = "positive"
            confidence = min(positive_count / len(doc), 1.0)
        elif negative_count > positive_count:
            sentiment = "negative"
            confidence = min(negative_count / len(doc), 1.0)
        else:
            sentiment = "neutral"
            confidence = 0.5
        
        return {"sentiment": sentiment, "confidence": confidence}
    
    async def extract_keywords(self, text: str) -> List[str]:
        """Extract key musical keywords from text"""
        if not self.nlp:
            # Fallback to simple word extraction
            words = text.lower().split()
            return [word for word in words if len(word) > 3]
        
        doc = self.nlp(text)
        keywords = []
        
        for token in doc:
            if (token.pos_ in ["NOUN", "ADJ"] and 
                not token.is_stop and 
                not token.is_punct and 
                len(token.text) > 2):
                keywords.append(token.text.lower())
        
        return keywords

# Example usage and testing
async def main():
    """Example usage of the music language processor"""
    processor = MusicLanguageProcessor()
    
    test_requests = [
        "I want some happy pop music for my workout",
        "Play some relaxing jazz from the 1970s",
        "I'm feeling sad, need some emotional ballads",
        "Give me upbeat dance music for my party",
        "I need focus music for studying",
        "Play some chill hip-hop for driving",
        "I want romantic songs for date night",
        "Some energetic rock music please",
        "Play classical music for meditation"
    ]
    
    for request in test_requests:
        print(f"\nRequest: {request}")
        result = await processor.process_request(request)
        print(f"Genres: {result['genres']}")
        print(f"Moods: {result['moods']}")
        print(f"Tempo: {result['tempo']}")
        print(f"Activities: {result['activities']}")
        print(f"Confidence: {result['confidence']:.2f}")

if __name__ == "__main__":
    asyncio.run(main()) 