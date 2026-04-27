import os
import sys
import time
import json
from simplenote import Simplenote

DIR = "/home/kramwriter/folder/simplenote"
CREDS_FILE = "/home/kramwriter/.simplenote_creds"
INDEX_FILE = os.path.join(DIR, ".sync_index.json")

def get_title(content):
    if not content or not content.strip():
        return "Untitled"
    lines = content.strip().split('\n')
    title = lines[0][:30].strip()
    clean_title = "".join([c for c in title if c.isalnum() or c in (' ', '_')]).strip()
    return clean_title if clean_title else "Untitled"

def load_index():
    if os.path.exists(INDEX_FILE):
        try:
            with open(INDEX_FILE, 'r') as f:
                return json.load(f)
        except:
            return {}
    return {}

def save_index(index):
    with open(INDEX_FILE, 'w') as f:
        json.dump(index, f)

def sync():
    if not os.path.exists(CREDS_FILE):
        sys.exit(1)

    with open(CREDS_FILE, 'r') as f:
        lines = f.read().splitlines()
        email, password = lines[0], lines[1]

    sn = Simplenote(email, password)
    # This is one fast request for ALL note metadata
    remote_notes_list, res = sn.get_note_list()    
    if res != 0:
        sys.exit(1)

    if not os.path.exists(DIR):
        os.makedirs(DIR)

    sync_index = load_index() # Maps Key -> Filename
    
    # Map of filename -> key for local lookup
    reverse_index = {v: k for k, v in sync_index.items()}
    
    # Get current local file stats (don't read content yet - save memory/time)
    local_files = {}
    for filename in os.listdir(DIR):
        if filename.endswith(".txt"):
            filepath = os.path.join(DIR, filename)
            local_files[filename] = os.path.getmtime(filepath)

    print("Syncing...")
    seen_keys = set()

    for note_meta in remote_notes_list:
        key = note_meta['key']
        if note_meta.get('deleted'): continue
        seen_keys.add(key)

        remote_mtime = float(note_meta.get('modifydate', 0))
        filename = sync_index.get(key)
        
        filepath = os.path.join(DIR, filename) if filename else None
        
        # DECISION LOGIC: Do we actually need to fetch the full note?
        need_to_fetch = False
        
        if not filename or not os.path.exists(filepath):
            # We've never seen this note before
            need_to_fetch = True
        else:
            local_mtime = local_files.get(filename, 0)
            # Only fetch if timestamps are > 10s apart
            if abs(remote_mtime - local_mtime) > 10:
                need_to_fetch = True

        if need_to_fetch:
            full_note, _ = sn.get_note(key)
            content = full_note.get('content', '')
            
            # If it's a brand new note, determine the filename
            if not filename:
                title = get_title(content)
                filename = f"{title}.txt"
                # Ensure no collisions
                counter = 1
                while os.path.exists(os.path.join(DIR, filename)):
                    filename = f"{title}_{counter}.txt"
                    counter += 1
                filepath = os.path.join(DIR, filename)
                sync_index[key] = filename

            local_mtime = os.path.getmtime(filepath) if os.path.exists(filepath) else 0

            # Comparison
            if remote_mtime > (local_mtime + 10):
                # Remote is newer: Download
                with open(filepath, 'w', encoding='utf-8') as f:
                    f.write(content)
                os.utime(filepath, (time.time(), remote_mtime))
                print(f"Downloaded: {filename}")
            
            elif local_mtime > (remote_mtime + 10):
                # Local is newer: Upload
                with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
                    local_content = f.read()
                full_note['content'] = local_content
                sn.update_note(full_note)
                # Simplenote returns new mtime after update, but we'll use local to be safe
                os.utime(filepath, (time.time(), local_mtime))
                print(f"Uploaded: {filename}")

        # Remove from local_files to track what's left (new local files)
        if filename in local_files:
            del local_files[filename]

    # Process remaining local files (brand new files created on device)
    for filename, mtime in local_files.items():
        if filename == ".sync_index.json": continue
        filepath = os.path.join(DIR, filename)
        with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
            content = f.read()
        
        if content.strip():
            new_note = {"content": content}
            res_note = sn.add_note(new_note)
            if 'key' in res_note:
                sync_index[res_note['key']] = filename
                print(f"Uploaded New Local: {filename}")

    save_index(sync_index)
    print("Sync Complete.")

if __name__ == "__main__":
    sync()