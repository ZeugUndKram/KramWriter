import os
import sys
import time
from simplenote import Simplenote

DIR = "/home/kramwriter/folder/simplenote"
CREDS_FILE = "/home/kramwriter/.simplenote_creds"

def get_title(content):
    lines = content.split('\n')
    title = lines[0][:30].strip() if lines else "Untitled"
    return "".join([c for c in title if c.isalnum() or c in (' ', '_')]).rstrip()

def sync():
    if not os.path.exists(CREDS_FILE):
        print("No credentials found.")
        sys.exit(1)

    with open(CREDS_FILE, 'r') as f:
        email, password = f.read().splitlines()

    sn = Simplenote(email, password)
    remote_notes, res = sn.get_notes()
    
    if res != 0:
        print("Failed to authenticate with Simplenote.")
        sys.exit(1)

    if not os.path.exists(DIR):
        os.makedirs(DIR)

    # Build a map of local files and their modified times
    local_files = {}
    for filename in os.listdir(DIR):
        if filename.endswith(".txt"):
            filepath = os.path.join(DIR, filename)
            local_files[filename] = {
                'path': filepath,
                'mtime': os.path.getmtime(filepath),
                'content': open(filepath, 'r').read()
            }

    print("Syncing...")

    # Process Remote Notes
    for note in remote_notes:
        if note.get('deleted'):
            continue
            
        full_note, _ = sn.get_note(note['key'])
        content = full_note.get('content', '')
        title = get_title(content)
        filename = f"{title}.txt"
        remote_mtime = float(full_note.get('modificationDate', 0))

        filepath = os.path.join(DIR, filename)

        if filename in local_files:
            local_mtime = local_files[filename]['mtime']
            # If local file is newer by more than 5 seconds, UPLOAD
            if local_mtime > remote_mtime + 5:
                full_note['content'] = local_files[filename]['content']
                sn.update_note(full_note)
                print(f"Uploaded changes for: {title}")
            # If remote is newer, DOWNLOAD
            elif remote_mtime > local_mtime + 5:
                with open(filepath, 'w') as f:
                    f.write(content)
                # Match local file time to remote time so it doesn't re-upload next sync
                os.utime(filepath, (time.time(), remote_mtime))
                print(f"Downloaded changes for: {title}")
            
            # Remove from our tracking list so we know it's been handled
            del local_files[filename]
        else:
            # File doesn't exist locally at all, DOWNLOAD
            with open(filepath, 'w') as f:
                f.write(content)
            os.utime(filepath, (time.time(), remote_mtime))
            print(f"Downloaded new note: {title}")

    # Process remaining local files (New files created on the Writerdeck)
    for filename, data in local_files.items():
        new_note = {"content": data['content']}
        sn.add_note(new_note)
        print(f"Uploaded new local file: {filename}")

    print("Sync Complete.")

if __name__ == "__main__":
    sync()