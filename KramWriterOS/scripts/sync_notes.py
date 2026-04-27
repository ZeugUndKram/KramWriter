import os
import sys
import time
from simplenote import Simplenote

DIR = "/home/kramwriter/folder/simplenote"
CREDS_FILE = "/home/kramwriter/.simplenote_creds"

def get_title(content):
    lines = content.strip().split('\n')
    if not lines or not lines[0].strip():
        return None
    # Take first 30 chars and keep only safe filename characters
    title = lines[0][:30].strip()
    clean_title = "".join([c for c in title if c.isalnum() or c in (' ', '_')]).strip()
    return clean_title if clean_title else None

def sync():
    if not os.path.exists(CREDS_FILE):
        print("Error: No credentials found.")
        sys.exit(1)

    with open(CREDS_FILE, 'r') as f:
        creds = f.read().splitlines()
        if len(creds) < 2:
            sys.exit(1)
        email, password = creds[0], creds[1]

    sn = Simplenote(email, password)
    # Use the modern method name
    remote_notes, res = sn.get_note_list()
    
    if res != 0:
        print("Error: Failed to authenticate.")
        sys.exit(1)

    if not os.path.exists(DIR):
        os.makedirs(DIR)

    # Build local file map with UTF-8 safety
    local_files = {}
    for filename in os.listdir(DIR):
        if filename.endswith(".txt"):
            filepath = os.path.join(DIR, filename)
            try:
                with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
                    content = f.read()
                local_files[filename] = {
                    'path': filepath,
                    'mtime': os.path.getmtime(filepath),
                    'content': content
                }
            except:
                continue

    print("Syncing...")

    # Process Remote Notes
    for note in remote_notes:
        if note.get('deleted'):
            continue
            
        full_note, _ = sn.get_note(note['key'])
        content = full_note.get('content', '')
        title = get_title(content)
        
        if not title: # Skip empty/ghost notes
            continue
            
        filename = f"{title}.txt"
        remote_mtime = float(full_note.get('modificationDate', 0))
        filepath = os.path.join(DIR, filename)

        if filename in local_files:
            local_mtime = local_files[filename]['mtime']
            # UPLOAD: Local is newer (5s buffer)
            if local_mtime > remote_mtime + 5:
                full_note['content'] = local_files[filename]['content']
                sn.update_note(full_note)
                print(f"Uploaded: {title}")
            # DOWNLOAD: Remote is newer
            elif remote_mtime > local_mtime + 5:
                with open(filepath, 'w', encoding='utf-8') as f:
                    f.write(content)
                os.utime(filepath, (time.time(), remote_mtime))
                print(f"Downloaded changes: {title}")
            
            del local_files[filename]
        else:
            # New file from server
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            os.utime(filepath, (time.time(), remote_mtime))
            print(f"New note: {title}")

    # Process remaining local files (New files from Writerdeck)
    for filename, data in local_files.items():
        if not data['content'].strip(): continue
        new_note = {"content": data['content']}
        sn.add_note(new_note)
        print(f"Uploaded new: {filename}")

    print("Sync Complete.")

if __name__ == "__main__":
    sync()