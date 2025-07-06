#!/usr/bin/env python3
"""
Setup script for Toka Music Playlist System

This script installs all dependencies and sets up the environment for the
comprehensive music playlist generation system.
"""

import os
import sys
import subprocess
import platform
from pathlib import Path

def run_command(command, description=""):
    """Run a shell command and handle errors"""
    print(f"Running: {command}")
    if description:
        print(f"Description: {description}")
    
    try:
        result = subprocess.run(command, shell=True, check=True, capture_output=True, text=True)
        print(f"Success: {result.stdout}")
        return True
    except subprocess.CalledProcessError as e:
        print(f"Error: {e.stderr}")
        return False

def check_python_version():
    """Check if Python version is compatible"""
    if sys.version_info < (3, 8):
        print("Error: Python 3.8 or higher is required")
        sys.exit(1)
    print(f"Python version: {sys.version}")

def install_system_dependencies():
    """Install system-level dependencies"""
    print("\n=== Installing System Dependencies ===")
    
    system = platform.system().lower()
    
    if system == "linux":
        # Ubuntu/Debian
        commands = [
            "sudo apt-get update",
            "sudo apt-get install -y ffmpeg portaudio19-dev python3-dev build-essential",
            "sudo apt-get install -y libffi-dev libssl-dev",
            "sudo apt-get install -y libasound2-dev",  # For audio processing
        ]
        
        for cmd in commands:
            if not run_command(cmd, "Installing Linux dependencies"):
                print(f"Warning: Failed to run {cmd}")
    
    elif system == "darwin":  # macOS
        # Check if Homebrew is installed
        if not run_command("brew --version", "Checking Homebrew"):
            print("Please install Homebrew first: https://brew.sh/")
            return False
        
        commands = [
            "brew update",
            "brew install ffmpeg portaudio",
            "brew install python@3.8",  # Ensure Python 3.8+
        ]
        
        for cmd in commands:
            if not run_command(cmd, "Installing macOS dependencies"):
                print(f"Warning: Failed to run {cmd}")
    
    elif system == "windows":
        print("Windows detected. Please install the following manually:")
        print("1. FFmpeg: https://ffmpeg.org/download.html")
        print("2. Microsoft Visual C++ Build Tools")
        print("3. Add FFmpeg to your PATH")
        input("Press Enter when ready to continue...")
    
    return True

def install_python_dependencies():
    """Install Python dependencies"""
    print("\n=== Installing Python Dependencies ===")
    
    requirements_file = Path(__file__).parent / "requirements.txt"
    
    if not requirements_file.exists():
        print("Error: requirements.txt not found")
        return False
    
    # Upgrade pip first
    run_command(f"{sys.executable} -m pip install --upgrade pip", "Upgrading pip")
    
    # Install requirements
    if not run_command(f"{sys.executable} -m pip install -r {requirements_file}", "Installing Python packages"):
        print("Error: Failed to install Python dependencies")
        return False
    
    return True

def download_spacy_model():
    """Download spaCy language model"""
    print("\n=== Downloading spaCy Model ===")
    
    if not run_command(f"{sys.executable} -m spacy download en_core_web_sm", "Downloading English model"):
        print("Warning: Failed to download spaCy model")
        return False
    
    return True

def create_directories():
    """Create necessary directories"""
    print("\n=== Creating Directories ===")
    
    directories = [
        "output",
        "logs",
        "cache/music",
        "backups/music",
        "temp"
    ]
    
    for directory in directories:
        dir_path = Path(directory)
        dir_path.mkdir(parents=True, exist_ok=True)
        print(f"Created: {dir_path}")

def create_env_template():
    """Create environment template file"""
    print("\n=== Creating Environment Template ===")
    
    env_template = """# Toka Music Playlist System Environment Variables
# Copy this file to .env and fill in your API credentials

# Spotify API Credentials
# Get these from: https://developer.spotify.com/
SPOTIFY_CLIENT_ID=your_spotify_client_id_here
SPOTIFY_CLIENT_SECRET=your_spotify_client_secret_here
SPOTIFY_REDIRECT_URI=http://localhost:8888/callback

# YouTube API Credentials
# Get these from: https://console.cloud.google.com/
YOUTUBE_API_KEY=your_youtube_api_key_here
YOUTUBE_CREDENTIALS_FILE=youtube_credentials.json

# NLP Configuration
NLP_MODEL_NAME=en_core_web_sm
NLP_USE_GPU=false

# Voice Processing Configuration
VOICE_MODEL_NAME=base
VOICE_LANGUAGE=en

# Optional: OpenAI API Key (for enhanced NLP)
# OPENAI_API_KEY=your_openai_api_key_here

# Optional: Redis Cache (for better performance)
# REDIS_URL=redis://localhost:6379

# Logging Configuration
LOG_LEVEL=INFO
LOG_FILE=logs/music_playlist.log

# Security
ENCRYPT_TOKENS=true
SANDBOX_MODE=false
"""
    
    env_file = Path(".env.template")
    with open(env_file, "w") as f:
        f.write(env_template)
    
    print(f"Created: {env_file}")
    print("Please copy .env.template to .env and fill in your API credentials")

def test_installation():
    """Test the installation"""
    print("\n=== Testing Installation ===")
    
    try:
        # Test imports
        import spacy
        import whisper
        import librosa
        import spotipy
        import googleapiclient
        
        print("✓ All critical imports successful")
        
        # Test spaCy model
        try:
            nlp = spacy.load("en_core_web_sm")
            print("✓ spaCy model loaded successfully")
        except Exception as e:
            print(f"✗ spaCy model error: {e}")
        
        # Test Whisper model
        try:
            model = whisper.load_model("base")
            print("✓ Whisper model loaded successfully")
        except Exception as e:
            print(f"✗ Whisper model error: {e}")
        
        return True
        
    except ImportError as e:
        print(f"✗ Import error: {e}")
        return False

def main():
    """Main setup function"""
    print("=== Toka Music Playlist System Setup ===")
    
    # Check Python version
    check_python_version()
    
    # Install system dependencies
    if not install_system_dependencies():
        print("Warning: System dependencies installation had issues")
    
    # Install Python dependencies
    if not install_python_dependencies():
        print("Error: Python dependencies installation failed")
        sys.exit(1)
    
    # Download spaCy model
    if not download_spacy_model():
        print("Warning: spaCy model download failed")
    
    # Create directories
    create_directories()
    
    # Create environment template
    create_env_template()
    
    # Test installation
    if test_installation():
        print("\n✓ Setup completed successfully!")
        print("\nNext steps:")
        print("1. Copy .env.template to .env")
        print("2. Fill in your API credentials in .env")
        print("3. Run the music playlist orchestrator:")
        print("   python3 orchestrator/music_playlist_orchestrator.py --help")
    else:
        print("\n✗ Setup completed with errors")
        print("Please check the error messages above and resolve them")

if __name__ == "__main__":
    main() 