#!/usr/bin/env python3
import argparse
import sys
from src.dastill.transcript_loader import YouTubeTranscriptLoader


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
    download_parser.add_argument('--channel', default='unknown',
                        help='Channel name for organizing processed files (default: unknown)')
    
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
    
    # Add command - Add video IDs to be downloaded later
    add_parser = subparsers.add_parser('add', help='Add video IDs to be downloaded later')
    add_parser.add_argument('video_ids', nargs='+', help='Video IDs or URLs to add')
    add_parser.add_argument('--title', help='Optional title for the video')
    add_parser.add_argument('--channel', default='unknown',
                           help='Channel name for organizing processed files (default: unknown)')
    
    # Status command - Update video status
    status_parser = subparsers.add_parser('status', help='Update video status')
    status_parser.add_argument('video_id', help='Video ID to update')
    status_parser.add_argument('new_status', choices=['to_be_downloaded', 'downloaded', 'processed'], 
                              help='New status for the video')
    
    # Process command - Move videos from downloaded to processed
    process_parser = subparsers.add_parser('process', help='Mark videos as processed')
    process_parser.add_argument('video_ids', nargs='+', help='Video IDs to mark as processed')
    process_parser.add_argument('--channel', help='Override channel name for processed files')
    
    # Queue command - Show videos in different statuses
    queue_parser = subparsers.add_parser('queue', help='Show videos by status')
    queue_parser.add_argument('--status', choices=['to_be_downloaded', 'downloaded', 'processed'], 
                             help='Filter by specific status')
    
    # Handle legacy usage (no subcommand)
    if len(sys.argv) > 1 and not any(sys.argv[1] == cmd for cmd in ['download', 'list', 'info', 'remove', 'config', 'add', 'status', 'process', 'queue']):
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
        elif args.command == 'add':
            handle_add(loader, args)
        elif args.command == 'status':
            handle_status(loader, args)
        elif args.command == 'process':
            handle_process(loader, args)
        elif args.command == 'queue':
            handle_queue(loader, args)
    
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
        save_markdown=save_markdown,
        channel=args.channel
    )
    
    if transcript_data.get('already_exists'):
        print(f"✓ Video already processed!")
        print(f"Video ID: {transcript_data.get('video_id', 'N/A')}")
        print(f"Language: {transcript_data.get('language', 'N/A')}")
        print(f"Auto-generated: {transcript_data.get('is_generated', 'N/A')}")
        print(f"File: {transcript_data.get('file_path', 'N/A')}")
        print(f"Status: {transcript_data.get('status', 'N/A')}")
        print(f"Channel: {transcript_data.get('channel', 'N/A')}")
        return
    
    print(f"✓ Transcript loaded successfully!")
    print(f"Video ID: {transcript_data.get('video_id', 'N/A')}")
    print(f"Language: {transcript_data.get('language', 'N/A')}")
    print(f"Auto-generated: {transcript_data.get('is_generated', 'N/A')}")
    
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
        print(f"Total videos: {stats['total']}")
        print(f"To be downloaded: {stats['to_be_downloaded']}")
        print(f"Downloaded: {stats['downloaded']}")
        print(f"Processed: {stats['processed']}")
        print(f"\nChannels: {stats['channels']}")
    else:
        videos = loader.list_processed_videos()
        if not videos:
            print("No videos found.")
            return
        
        print(f"All videos ({len(videos)}):")
        print("-" * 80)
        for video in videos:
            print(f"ID: {video['video_id']}")
            print(f"Status: {video['status']}")
            print(f"Channel: {video['channel']}")
            if video.get('file_path'):
                print(f"File: {video['file_path']}")
            print("-" * 40)


def handle_info(loader, args):
    info = loader.get_video_info(args.video_id)
    if not info:
        print(f"No information found for video ID: {args.video_id}")
        return
    
    print(f"Video Information:")
    print(f"ID: {info.get('video_id', 'N/A')}")
    print(f"Status: {info.get('status', 'unknown')}")
    print(f"Channel: {info.get('channel', 'unknown')}")
    print(f"File: {info.get('file_path', 'N/A')}")
    
    # Note: In stateless mode, language/generation info is only available 
    # when reading from actual transcript files, not from file system metadata
    print(f"Language: {info.get('language', 'Check file for details')}")
    print(f"Auto-generated: {info.get('is_generated', 'Check file for details')}")
    
    metadata = info.get('metadata', {})
    if metadata:
        print(f"Languages requested: {metadata.get('languages_requested', 'N/A')}")
        print(f"File size: {metadata.get('file_size', 'N/A')} bytes")


def handle_remove(loader, args):
    result = loader.remove_video(args.video_id, delete_file=args.delete_file)
    
    if result['found']:
        print(f"✓ Video {args.video_id} found (status: {result.get('previous_status', 'unknown')})")
        
        if args.delete_file:
            if result['file_deleted']:
                print("✓ Associated file deleted successfully")
            elif result['error']:
                print(f"⚠ Warning: {result['error']}")
            else:
                print("⚠ File deletion was requested but not performed")
        else:
            print("Note: File was not deleted (use --delete-file to remove file)")
    else:
        print(f"Video {args.video_id} not found")


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


def handle_add(loader, args):
    added_count = 0
    
    for video_input in args.video_ids:
        # Extract video ID from URL if needed
        video_id = loader._extract_video_id(video_input) if 'youtube.com' in video_input or 'youtu.be' in video_input else video_input
        
        if loader.add_to_be_downloaded(video_id, args.channel):
            print(f"✓ Added {video_id} to download queue (channel: {args.channel})")
            added_count += 1
        else:
            print(f"⚠ Video {video_id} already in system")
    
    print(f"\nAdded {added_count} new video(s) to download queue")


def handle_status(loader, args):
    video_id = loader._extract_video_id(args.video_id) if 'youtube.com' in args.video_id or 'youtu.be' in args.video_id else args.video_id
    
    current_status, file_path = loader.manager.get_video_status(video_id)
    
    if current_status == 'not_downloaded':
        print(f"Video {video_id} not found")
        return
    
    print(f"Current status of {video_id}: {current_status}")
    print(f"Note: In stateless mode, status changes are done by moving files between folders:")
    print(f"  - to_be_downloaded: /to_be_downloaded/")
    print(f"  - downloaded: /downloaded/") 
    print(f"  - processed: /[channel-name]/")
    print(f"Use the 'process' command to move from downloaded to processed.")
    
    if file_path:
        print(f"Current file: {file_path}")


def handle_process(loader, args):
    processed_count = 0
    
    for video_input in args.video_ids:
        video_id = loader._extract_video_id(video_input) if 'youtube.com' in video_input or 'youtu.be' in video_input else video_input
        
        try:
            success, result = loader.process_video(video_id, args.channel)
            if success:
                print(f"✓ Processed {video_id}")
                print(f"  File moved to: {result}")
                processed_count += 1
            else:
                print(f"⚠ {result}")
        except Exception as e:
            print(f"Error processing {video_id}: {str(e)}")
    
    print(f"\nProcessed {processed_count} video(s)")


def handle_queue(loader, args):
    if args.status:
        videos = loader.manager.list_videos_by_status(args.status)
        print(f"Videos with status '{args.status}' ({len(videos)}):")
    else:
        # Show all statuses
        stats = loader.get_stats()
        print("Video Queue Overview:")
        print(f"Total videos: {stats['total']}")
        print(f"  to_be_downloaded: {stats['to_be_downloaded']}")
        print(f"  downloaded: {stats['downloaded']}")
        print(f"  processed: {stats['processed']}")
        print(f"\nChannels: {stats['channels']}")
        print("\nAll videos:")
        videos = loader.list_processed_videos()
    
    if not videos:
        print("No videos found.")
        return
    
    print("-" * 80)
    for video in videos:
        print(f"ID: {video['video_id']}")
        print(f"Status: {video['status']}")
        print(f"Channel: {video['channel']}")
        if video.get('file_path'):
            print(f"File: {video['file_path']}")
        print("-" * 40)


if __name__ == "__main__":
    main()
