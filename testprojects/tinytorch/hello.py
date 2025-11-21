#!/usr/bin/env python3
"""Simple hello world program using colorama package."""

from colorama import Fore, Style, init
import pytorch

# Initialize colorama
init(autoreset=True)

def main():
    print(f"{Fore.GREEN}Hello World!{Style.RESET_ALL}")
    print("Pytorch version:", pytorch.__version__)

if __name__ == "__main__":
    main()
