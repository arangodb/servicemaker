#!/usr/bin/env python3
"""Simple hello world program using colorama package."""

from colorama import Fore, Style, init
import cugraph

# Initialize colorama
init(autoreset=True)

def main():
    print("Cugraph version:", cugraph.__version__)
    print(f"{Fore.GREEN}Hello World!{Style.RESET_ALL}")

if __name__ == "__main__":
    main()
