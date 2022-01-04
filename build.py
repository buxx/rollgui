# coding: utf-8
import argparse
import os
import shutil
import subprocess
import typing
import tempfile
import uuid
from json import JSONDecodeError

import requests

TRACIM_API_URL = "https://tracim.bux.fr/api/workspaces/{workspace_id}/files/{content_id}/raw/{filename}"
TRACIM_LOGIN = "sevajol.bastien@gmail.com"

CONFIG = {
    "x86_64-unknown-linux-gnu": ("Rolling_Linux_x86-64", "5", "201"),
    "i686-unknown-linux-gnu": ("Rolling_Linux_i686", "5", "384"),
    "x86_64-pc-windows-gnu": ("Rolling_Windows_x86-64", "5", "200"),
    "i686-pc-windows-msvc": ("Rolling_windows_i686", "5", "218"),
}


def main(
    targets: typing.List[str],
    tracim_api_key: typing.Optional[str] = None,
    debug: bool = False,
    upload_tracim: bool = False,
    upload_release: bool = False,
    cross: bool = False,
) -> None:
    for target in targets:
        print(target)
        uuid_ = uuid.uuid4().hex
        TMP_DIR = tempfile.gettempdir() + "/rolling_" + uuid_
        assert target in CONFIG
        if upload_tracim:
            assert tracim_api_key
        file_name, tracim_workspace_id, tracim_content_id = CONFIG[target]
        release_str = " --release" if not debug else ""
        base_cmd = "cargo" if not cross else "cross"

        # compile
        command = f"{base_cmd} build --target {target}{release_str}"
        print(command)
        subprocess.check_output(
            command,
            shell=True,
        )

        # zip
        folder_str = "release" if not debug else "debug"
        exe_extension = ".exe" if "windows" in target else ""
        shutil.rmtree(f"{TMP_DIR}/rolling/{file_name}", ignore_errors=True)
        os.makedirs(f"{TMP_DIR}/rolling/{file_name}")
        os.makedirs(f"{TMP_DIR}/rolling/{file_name}/resources")
        shutil.copy(
            f"target/{target}/{folder_str}/rollgui{exe_extension}",
            f"{TMP_DIR}/rolling/{file_name}/Heritage{exe_extension}",
        )
        shutil.copy(
            f"resources/graphics.png", f"{TMP_DIR}/rolling/{file_name}/resources/"
        )
        shutil.copy(f"resources/intro.png", f"{TMP_DIR}/rolling/{file_name}/resources/")
        shutil.copy(
            f"resources/introb.png", f"{TMP_DIR}/rolling/{file_name}/resources/"
        )
        with open(f"{TMP_DIR}/rolling/{file_name}/config.ini", "w+") as config_file:
            config_file.write(
                """[debug]
enable_bug_report = true

[server]
name = Héritage
server_hostname = rolling-server.bux.fr
server_port = 4443
unsecure = false

[design]
title = H
home_image = resources/intro.png
home_image_background = resources/introb.png
"""
            )
        zip_command = f"cd {TMP_DIR}/rolling && zip -r {file_name}.zip {file_name}"
        if "windows" in target:
            shutil.copy(f"rollgui.bat", f"{TMP_DIR}/rolling/{file_name}/")
            if not cross:
                zip_command = f"cd {TMP_DIR}\\rolling && tar.exe -a -c -f {file_name}.zip {file_name}"
        print(zip_command)
        subprocess.check_output(
            zip_command,
            shell=True,
        )
        print(f"zip available at {TMP_DIR}/rolling/{file_name}.zip")

        file_to_upload_path = f"{TMP_DIR}/rolling/{file_name}.zip"
        if "windows" in target and not cross:
            file_to_upload_path = f"{TMP_DIR}\\rolling\\{file_name}.zip"

        if upload_tracim:
            print("upload to Tracim ... ")

            # upload
            with open(file_to_upload_path, "rb") as zip_file:
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

        if upload_release:

            print("Upload to rolling.bux.fr/release")
            with open("Cargo.toml", "r") as cargo_file:
                cargo_content = cargo_file.read()

            version = None
            for line in cargo_content.splitlines():
                line = line.strip()
                if line.startswith("version"):
                    version = line.split("=")[1].strip()[1:-1]
                    break

            assert version is not None

            subprocess.check_output(
                f"scp -P 3122 "
                f"{file_to_upload_path} "
                f"bux@s2.bux.fr:/srv/www/bux.fr/rolling/release/{file_name}_{version}.zip",
                shell=True,
            )
            subprocess.check_output(
                f'ssh -p 3122 bux@s2.bux.fr \'echo "{version}" '
                f">> /srv/www/bux.fr/rolling/release/index'",
                shell=True,
            )
            print("OK")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("target", nargs="+")
    parser.add_argument("--tracim-api-key", default=None)
    parser.add_argument("--debug", action="store_true", default=False)
    parser.add_argument("--upload-tracim", action="store_true", default=False)
    parser.add_argument("--upload-release", action="store_true", default=False)
    parser.add_argument("--cross", action="store_true", default=False)
    args = parser.parse_args()
    main(
        args.target,
        args.tracim_api_key,
        debug=args.debug,
        upload_tracim=args.upload_tracim,
        upload_release=args.upload_release,
        cross=args.cross,
    )
