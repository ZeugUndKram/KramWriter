import os
import sys
import time
from simplenote import Simplenote

DIR = "/home/kramwriter/folder/simplenote"
CREDS_FILE = "/home/kramwriter/.simplenote_creds"

def get_title(content):
    if not content or not content.strip():
        return None
    lines = content.strip().split('\n')
    title = lines[0][:30].strip()
    clean_title = "".join([c for c in title if c.isalnum() or c in (' ', '_')]).strip()
    return clean_title if clean_title else "Untitled"

def sync():
    if not os.path.exists(CREDS_FILE):
        sys.exit(1)

    with open(CREDS_FILE, 'r') as f:
        lines = f.read().splitlines()
        email, password = lines[0], lines[1]

    sn = Simplenote(email, password)
    remote_notes, res = sn.get_note_list()    
    
    if res != 0:
        sys.exit(1)

    if not os.path.exists(DIR):
        os.makedirs(DIR)

    local_files = {}
    for filename in os.listdir(DIR):
        if filename.endswith(".txt"):
            filepath = os.path.join(DIR, filename)
            try:
                with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
                    content = f.read()
                local_files[filename] = {
                    'mtime': os.path.getmtime(filepath),
                    'content': content
                }
            except:
                continue

    print("Syncing...")
    
    seen_filenames = set()

    for note in remote_notes:
        if note.get('deleted'): continue
            
        full_note, _ = sn.get_note(note['key'])
        content = full_note.get('content', '')
        title = get_title(content)
        if not title: continue
            
        # Handle Duplicate Titles (e.g., GTA Radio, GTA Radio_1)
        base_filename = f"{title}.txt"
        filename = base_filename
        counter = 1
        while filename in seen_filenames:
            filename = f"{title}_{counter}.txt"
            counter += 1
        seen_filenames.add(filename)

        remote_mtime = float(full_note.get('modificationDate', 0))
        filepath = os.path.join(DIR, filename)

        if filename in local_files:
            local_mtime = local_files[filename]['mtime']
            local_content = local_files[filename]['content']

            # CRITICAL FIX: Only sync if content is actually DIFFERENT
            if local_content != content:
                # If local is newer than remote by more than 10 seconds
                if local_mtime > (remote_mtime + 10):
                    full_note['content'] = local_content
                    sn.update_note(full_note)
                    print(f"Uploaded: {filename}")
                # If remote is newer than local by more than 10 seconds
                elif remote_mtime > (local_mtime + 10):
                    with open(filepath, 'w', encoding='utf-8') as f:
                        f.write(content)
                    os.utime(filepath, (time.time(), remote_mtime))
                    print(f"Downloaded: {filename}")
            
            del local_files[filename]
        else:
            # Download new notes from server
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            os.utime(filepath, (time.time(), remote_mtime))
            print(f"New Remote Note: {filename}")

    # Upload local files that don't exist on server
    for filename, data in local_files.items():
        if data['content'].strip():
            sn.add_note({"content": data['content']})
            print(f"Uploaded New Local: {filename}")

    print("Sync Complete.")

if __name__ == "__main__":
    sync()