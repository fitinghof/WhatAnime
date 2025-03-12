import os


def clear_screen():
    # Check the OS and run the appropriate clear command
    if os.name == "nt":  # For Windows
        os.system("cls")
    else:  # For macOS and Linux
        os.system("clear")
