# coding: utf-8
import argparse
import os
import shutil
import subprocess
import typing
import tempfile
from json import JSONDecodeError

import requests

TRACIM_API_URL = (
    "https://tracim.bux.fr"
    "/api/v2/workspaces/{workspace_id}/files/{content_id}/raw/{filename}"
)
TRACIM_LOGIN = "sevajol.bastien@gmail.com"

CONFIG = {
    "x86_64-unknown-linux-gnu": ("Rolling_Linux_x86-64", "5", "201"),
    "i686-unknown-linux-gnu": ("Rolling_Linux_i686", "5", "384"),
    "x86_64-pc-windows-msvc": ("Rolling_Windows_x86-64", "5", "200"),
    "i686-pc-windows-msvc": ("Rolling_windows_i686", "5", "218"),
}
TMP_DIR = tempfile.gettempdir()


def main(targets: typing.List[str], tracim_api_key: typing.Optional[str] = None, debug: bool = False, upload: bool = False) -> None:
    for target in targets:
        print(target)
        assert target in CONFIG
        assert (not upload and debug) or (upload and not debug)
        assert (tracim_api_key and upload) or (not tracim_api_key and not upload)
        file_name, tracim_workspace_id, tracim_content_id = CONFIG[target]
        release_str = " --release" if not debug else ""

        # compile
        subprocess.check_output(
            f"cargo build --target {target}{release_str}",
            shell=True,
        )

        # zip
        folder_str = "release" if not debug else "debug"
        exe_extension = ".exe" if "windows" in target else ""
        shutil.rmtree(f"{TMP_DIR}/rolling/{file_name}", ignore_errors=True)
        os.makedirs(f"{TMP_DIR}/rolling/{file_name}")
        os.makedirs(f"{TMP_DIR}/rolling/{file_name}/resources")
        shutil.copy(f"target/{target}/{folder_str}/rollgui{exe_extension}", f"{TMP_DIR}/rolling/{file_name}")
        shutil.copy(f"resources/tilesheet.png", f"{TMP_DIR}/rolling/{file_name}/resources/")
        shutil.copy(f"resources/ui.png", f"{TMP_DIR}/rolling/{file_name}/resources/")
        zip_command = f"cd {TMP_DIR}\rolling && zip -r {file_name}.zip {file_name}"
        if "windows" in target:
            shutil.copy(f"rollgui.bat", f"{TMP_DIR}/rolling/{file_name}/")
            zip_command = f"cd {TMP_DIR}\\rolling && tar.exe -a -c -f {file_name}.zip {file_name}"
        print(zip_command)
        subprocess.check_output(
            zip_command,
            shell=True,
        )
        print(f"zip available at {TMP_DIR}/rolling/{file_name}.zip")

        if not upload:
            return
        print("upload ... ", end="")

        # upload
        with open(f"{TMP_DIR}/rolling/{file_name}.zip", "rb") as zip_file:
            response = requests.put(
                url=TRACIM_API_URL.format(
                    workspace_id=tracim_workspace_id,
                    content_id=tracim_content_id,
                    filename=f"{file_name}.zip",
                ),
                files={"files": zip_file},
                headers={
                    "Tracim-Api-Key": tracim_api_key,
                    "Tracim-Api-Login": TRACIM_LOGIN,
                },
            )
            if response.status_code != 204:
                try:
                    print(response.json()["message"])
                except JSONDecodeError:
                    print(response.text)
                exit(1)

        print("OK")


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument("target", nargs="+")
    parser.add_argument("--tracim-api-key", default=None)
    parser.add_argument("--debug", action="store_true", default=False)
    parser.add_argument("--upload", action="store_true", default=False)
    args = parser.parse_args()
    main(args.target, args.tracim_api_key, debug=args.debug, upload=args.upload)
