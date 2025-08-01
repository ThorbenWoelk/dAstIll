#!/usr/bin/env python3
import argparse
import sys
from src.dastill.youtube_loader import YouTubeTranscriptLoader


def main():
    parser = argparse.ArgumentParser(description='dAstIll - YouTube Transcript Loader')
    parser.add_argument('url', help='YouTube video URL or ID')
    parser.add_argument('-l', '--languages', nargs='+', default=['en'], 
                        help='Preferred languages for transcript (default: en)')
    parser.add_argument('-o', '--output', help='Output file path to save transcript')
    parser.add_argument('--raw', action='store_true', 
                        help='Output raw transcript instead of cleaned version')
    
    args = parser.parse_args()
    
    loader = YouTubeTranscriptLoader()
    
    try:
        print(f"Loading transcript for: {args.url}")
        transcript_data = loader.load_transcript(args.url, args.languages)
        
        print(f"Video ID: {transcript_data['video_id']}")
        print(f"Language: {transcript_data['language']}")
        print(f"Auto-generated: {transcript_data['is_generated']}")
        print("-" * 50)
        
        if args.output:
            loader.save_transcript(transcript_data, args.output)
            print(f"Transcript saved to: {args.output}")
        else:
            print("\nTranscript:")
            print("-" * 50)
            if args.raw:
                print(transcript_data['formatted_text'])
            else:
                print(transcript_data['cleaned_text'])
    
    except Exception as e:
        print(f"Error: {str(e)}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
