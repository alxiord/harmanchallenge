#!/usr/bin/env python3

import os
import subprocess
import sys
from pathlib import Path
import webbrowser

DOCKER_IMAGE = "aior/harmanchallenge:v3"
DOCKER_CTR_HOME = "/home/alexandra"
BUILD_CMD = "cargo build --release"
DOC_CMD = "cargo doc"

class Wrapper:
    def __init__(self):
        self.projdir = Path(__file__).parent

    def usage():
        print(
            """
Usage:
./harmanchallenge.py build
    Runs the docker container `{}`, mounts the project directory as `{}`, 
    and inside it runs `cargo build --release` to build the project.

./harmanchallenge.py run [args]
    Runs the built binary with the specified args.
    
    Example:
    ./harmanchallenge.py run --input=input/hello.mp4 --width=640 --height=480 --format=h264 --flip --invert

    Args:
        --input: path to the input video file (mp4 supported only)
        --width: width in px of the output video (optional)
        --height: height in px of the output video (optional)
        --format: format for encoding the video before passing it to the screen sink (h264 supported only) (optional)
        --flip: flip the output video horizontally (optional)
        --invert: invert colors on the output video (optional)

./harmanchallenge.py doc
    Runs the docker container `{}`, mounts the project directory as `{}`, 
    and inside it runs `cargo doc --open` to build the documentation and open
    it in the browser.
            """.format(
                DOCKER_IMAGE, DOCKER_CTR_HOME, DOCKER_IMAGE, DOCKER_CTR_HOME
            )
        )
    
    def build(self):
        """Build the project in the docker container."""
        command = [
            "docker",
            "run",
            "--rm",
            "-v",
            f"{self.projdir}:{DOCKER_CTR_HOME}",
            DOCKER_IMAGE
        ]
        command.extend(BUILD_CMD.split(" "))
        
        subprocess.run(command, check=True)

    def run(self, args):
        """Run the built binary with the specified args."""
        if not args:
            print("Error: No command specified for 'run'.")
            self.usage()
            sys.exit(1)

        command = [
            f"{self.projdir}/target/release/harmanchallenge"
        ]
        command.extend(args)

        subprocess.run(command, check=True)

    def doc(self):
        """Generate and display the documentation."""
        command = [
            "docker",
            "run",
            "--rm",
            "-v",
            f"{self.projdir}:{DOCKER_CTR_HOME}",
            DOCKER_IMAGE
        ]
        command.extend(DOC_CMD.split(" "))

        subprocess.run(command, check=True)

        if not webbrowser.open("file://" + os.path.realpath("target/doc/harmanchallenge/index.html")):
            print("Error: Could not open browser.")
            print("Please open the file manually: target/doc/harmanchallenge/index.html")


def main():
    if len(sys.argv) < 2:
        print("Error: Missing command")
        Wrapper.usage()
        sys.exit(1)

    command = sys.argv[1]
    args = sys.argv[2:]

    wrapper = Wrapper()

    if command == "build":
        wrapper.build()
    elif command == "run":
        wrapper.run(args)
    elif command == "doc":
        wrapper.doc()
    else:
        print(f"Error: Unknown command '{command}'")
        Wrapper.usage()
        sys.exit(1)

if __name__ == "__main__":
    main()
