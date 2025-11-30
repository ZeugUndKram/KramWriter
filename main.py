#!/usr/bin/env python3
import os
import sys
import subprocess


def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))

    while True:
        print("\n=== Application Launcher ===")
        print("1. Show Logo")
        print("2. Show Menu")
        print("3. Exit")

        choice = input("Select option (1-3): ").strip()

        if choice == '1':
            logo_path = os.path.join(script_dir, "logo.py")
            if os.path.exists(logo_path):
                print("Launching logo...")
                result = subprocess.run([sys.executable, logo_path], capture_output=True, text=True)
                print(result.stdout)
                if result.stderr:
                    print("Errors:", result.stderr)
            else:
                print(f"logo.py not found at {logo_path}")

        elif choice == '2':
            menu_path = os.path.join(script_dir, "menu.py")
            if os.path.exists(menu_path):
                print("Launching menu...")
                result = subprocess.run([sys.executable, menu_path], capture_output=True, text=True)
                print(result.stdout)
                if result.stderr:
                    print("Errors:", result.stderr)
            else:
                print(f"menu.py not found at {menu_path}")

        elif choice == '3':
            print("Goodbye!")
            break
        else:
            print("Invalid choice. Please select 1-3.")


if __name__ == "__main__":
    main()