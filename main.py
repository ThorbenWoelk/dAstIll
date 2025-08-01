#!/usr/bin/env python3
import argparse
import sys
from src.dastill.youtube_loader import YouTubeTranscriptLoader


def main():
    parser = argparse.ArgumentParser(description='dAstIll - YouTube Transcript Loader')
    
    # Add subcommands
    subparsers = parser.add_subparsers(dest='command', help='Available commands')
    
    # Download command (default behavior)
    download_parser = subparsers.add_parser('download', help='Download transcript for a video')
    download_parser.add_argument('url', help='YouTube video URL or ID')
    download_parser.add_argument('-l', '--languages', nargs='+', 
                        help='Preferred languages for transcript (default: from config)')
    download_parser.add_argument('-o', '--output', help='Output file path to save transcript')
    download_parser.add_argument('--raw', action='store_true', 
                        help='Output raw transcript instead of cleaned version')
    download_parser.add_argument('--force', action='store_true', 
                        help='Force download even if video already processed')
    download_parser.add_argument('--no-markdown', action='store_true', 
                        help='Disable markdown storage')
    
    # List command
    list_parser = subparsers.add_parser('list', help='List processed videos')
    list_parser.add_argument('--stats', action='store_true', help='Show statistics')
    
    # Info command
    info_parser = subparsers.add_parser('info', help='Show info for a specific video')
    info_parser.add_argument('video_id', help='Video ID to get info for')
    
    # Remove command
    remove_parser = subparsers.add_parser('remove', help='Remove a video from tracking')
    remove_parser.add_argument('video_id', help='Video ID to remove')
    remove_parser.add_argument('--delete-file', action='store_true', 
                        help='Also delete the transcript file')
    
    # Config command
    config_parser = subparsers.add_parser('config', help='Show current configuration')
    
    # Handle legacy usage (no subcommand)
    if len(sys.argv) > 1 and not any(sys.argv[1] == cmd for cmd in ['download', 'list', 'info', 'remove', 'config']):
        # Insert 'download' as the first argument for backward compatibility
        sys.argv.insert(1, 'download')
    
    args = parser.parse_args()
    
    # Set default command if none provided
    if args.command is None:
        args.command = 'download'
    
    loader = YouTubeTranscriptLoader()
    
    try:
        if args.command == 'download':
            handle_download(loader, args)
        elif args.command == 'list':
            handle_list(loader, args)
        elif args.command == 'info':
            handle_info(loader, args)
        elif args.command == 'remove':
            handle_remove(loader, args)
        elif args.command == 'config':
            handle_config(loader, args)
    
    except Exception as e:
        print(f"Error: {str(e)}", file=sys.stderr)
        sys.exit(1)


def handle_download(loader, args):
    print(f"Loading transcript for: {args.url}")
    
    save_markdown = not args.no_markdown
    transcript_data = loader.load_transcript(
        args.url, 
        args.languages, 
        force=args.force, 
        save_markdown=save_markdown
    )
    
    if transcript_data.get('already_exists'):
        print(f"✓ Video already processed!")
        print(f"Video ID: {transcript_data['video_id']}")
        print(f"Language: {transcript_data['language']}")
        print(f"Auto-generated: {transcript_data['is_generated']}")
        print(f"File: {transcript_data.get('file_path', 'N/A')}")
        print(f"Processed: {transcript_data.get('processed_at', 'N/A')}")
        return
    
    print(f"✓ Transcript loaded successfully!")
    print(f"Video ID: {transcript_data['video_id']}")
    print(f"Language: {transcript_data['language']}")
    print(f"Auto-generated: {transcript_data['is_generated']}")
    
    if transcript_data.get('file_path'):
        print(f"Saved to: {transcript_data['file_path']}")
    
    if args.output:
        loader.save_transcript(transcript_data, args.output)
        print(f"Also saved to: {args.output}")
    
    if not args.output and not transcript_data.get('file_path'):
        print("\nTranscript:")
        print("-" * 50)
        if args.raw:
            print(transcript_data['formatted_text'])
        else:
            print(transcript_data['cleaned_text'])


def handle_list(loader, args):
    if args.stats:
        stats = loader.get_stats()
        print("Statistics:")
        print(f"Total videos: {stats['total_videos']}")
        print(f"Auto-generated: {stats['auto_generated_count']}")
        print(f"Manual transcripts: {stats['manual_transcript_count']}")
        print("\nLanguages:")
        for lang, count in stats['languages'].items():
            print(f"  {lang}: {count}")
    else:
        videos = loader.list_processed_videos()
        if not videos:
            print("No videos processed yet.")
            return
        
        print(f"Processed videos ({len(videos)}):")
        print("-" * 80)
        for video in videos:
            print(f"ID: {video['video_id']}")
            print(f"Language: {video.get('language', 'unknown')}")
            print(f"Generated: {video.get('is_generated', 'unknown')}")
            print(f"Processed: {video.get('processed_at', 'unknown')}")
            print(f"File: {video.get('file_path', 'N/A')}")
            print("-" * 40)


def handle_info(loader, args):
    info = loader.get_video_info(args.video_id)
    if not info:
        print(f"No information found for video ID: {args.video_id}")
        return
    
    print(f"Video Information:")
    print(f"ID: {info['video_id']}")
    print(f"Language: {info.get('language', 'unknown')}")
    print(f"Auto-generated: {info.get('is_generated', 'unknown')}")
    print(f"Processed: {info.get('processed_at', 'unknown')}")
    print(f"File: {info.get('file_path', 'N/A')}")
    
    metadata = info.get('metadata', {})
    if metadata:
        print(f"Languages requested: {metadata.get('languages_requested', 'N/A')}")
        print(f"File size: {metadata.get('file_size', 'N/A')} bytes")


def handle_remove(loader, args):
    success = loader.remove_video(args.video_id, delete_file=args.delete_file)
    if success:
        action = "and file deleted" if args.delete_file else ""
        print(f"✓ Video {args.video_id} removed from tracking {action}")
    else:
        print(f"Video {args.video_id} not found in tracking database")


def handle_config(loader, args):
    config = loader.config.config
    print("Current Configuration:")
    print("=" * 50)
    
    def print_dict(d, indent=0):
        for key, value in d.items():
            if isinstance(value, dict):
                print("  " * indent + f"{key}:")
                print_dict(value, indent + 1)
            else:
                print("  " * indent + f"{key}: {value}")
    
    print_dict(config)


if __name__ == "__main__":
    main()
