# coding: utf-8
import argparse
import os
import shutil
import subprocess
import typing
import tempfile
import uuid
import dataclasses
from json import JSONDecodeError

import requests

TRACIM_API_URL = "https://tracim.bux.fr/api/workspaces/{workspace_id}/files/{content_id}/raw/{filename}"
TRACIM_LOGIN = "sevajol.bastien@gmail.com"


@dataclasses.dataclass
class Config:
    file_name: str
    bin_name: str
    tracim_workspace_id: int
    tracim_content_id: int
    config_file_content: str
    resources: typing.List[str]
    release_folder: str


heritage_config_file_content = """[debug]
enable_bug_report = true

[server]
name = Héritage
server_hostname = rolling-server.bux.fr
server_port = 4443
unsecure = false
releases_url = http://rolling.bux.fr/release

[design]
title = Héritage
home_image = resources/intro.png
home_image_background = resources/introb.png
"""

heritage_resources = [
    "resources/graphics.png",
    "resources/intro.png",
    "resources/introb.png",
]

heritage_release_folder = "/srv/www/bux.fr/rolling/release/"

creatif_config_file_content = """[debug]
enable_bug_report = true

[server]
name = Rolling Créatif
server_hostname = rolling-server-creatif.bux.fr
server_port = 4443
unsecure = false
releases_url = http://rolling-creatif.bux.fr/release

[design]
title = Héritage
home_image = resources/intro_creatif.png
home_image_background = resources/introb.png
"""

creatif_resources = [
    "resources/graphics.png",
    "resources/intro_creatif.png",
    "resources/introb.png",
]

creatif_release_folder = "/srv/www/bux.fr/rolling_creatif/release/"

CONFIGS = {
    "x86_64-unknown-linux-gnu": [
        Config(
            file_name="Heritage_Linux_x86-64",
            bin_name="Heritage",
            tracim_workspace_id=5,
            tracim_content_id=201,
            config_file_content=heritage_config_file_content,
            resources=heritage_resources,
            release_folder=heritage_release_folder,
        ),
        Config(
            file_name="RollingCreatif_Linux_x86-64",
            bin_name="RollingCreatif",
            tracim_workspace_id=5,
            tracim_content_id=3035,
            config_file_content=creatif_config_file_content,
            resources=creatif_resources,
            release_folder=creatif_release_folder,
        ),
    ],
    # "i686-unknown-linux-gnu": [
    #     Config(
    #         file_name="Heritage_Linux_i686",
    #         bin_name="Heritage",
    #         tracim_workspace_id=5,
    #         tracim_content_id=384,
    #         config_file_content=heritage_config_file_content,
    #         resources=heritage_resources,
    #         release_folder=heritage_release_folder,
    #     ),
    # ],
    "x86_64-pc-windows-gnu": [
        Config(
            file_name="Heritage_Windows_x86-64",
            bin_name="Heritage",
            tracim_workspace_id=5,
            tracim_content_id=200,
            config_file_content=heritage_config_file_content,
            resources=heritage_resources,
            release_folder=heritage_release_folder,
        ),
        Config(
            file_name="RollingCreatif_Windows_x86-64",
            bin_name="RollingCreatif",
            tracim_workspace_id=5,
            tracim_content_id=3034,
            config_file_content=creatif_config_file_content,
            resources=creatif_resources,
            release_folder=creatif_release_folder,
        ),
    ],
    # "i686-pc-windows-msvc": [
    #     Config(
    #         file_name="Heritage_Windows_i686",
    #         bin_name="Heritage",
    #         tracim_workspace_id=5,
    #         tracim_content_id=218,
    #         config_file_content=heritage_config_file_content,
    #         resources=heritage_resources,
    #         release_folder=heritage_release_folder,
    #     ),
    # ],
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
        if upload_tracim:
            assert tracim_api_key
        assert target in CONFIGS

        for config in CONFIGS[target]:
            uuid_ = uuid.uuid4().hex
            TMP_DIR = tempfile.gettempdir() + "/rolling_" + uuid_
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
            shutil.rmtree(f"{TMP_DIR}/rolling/{config.file_name}", ignore_errors=True)
            os.makedirs(f"{TMP_DIR}/rolling/{config.file_name}")
            os.makedirs(f"{TMP_DIR}/rolling/{config.file_name}/resources")
            shutil.copy(
                f"target/{target}/{folder_str}/rollgui{exe_extension}",
                f"{TMP_DIR}/rolling/{config.file_name}/{config.bin_name}{exe_extension}",
            )
            for resource in config.resources:
                shutil.copy(
                    resource,
                    f"{TMP_DIR}/rolling/{config.file_name}/resources/",
                )
            with open(
                f"{TMP_DIR}/rolling/{config.file_name}/config.ini", "w+"
            ) as config_file:
                config_file.write(config.config_file_content)
            zip_command = f"cd {TMP_DIR}/rolling && zip -r {config.file_name}.zip {config.file_name}"
            if "windows" in target:
                with open(
                    f"{TMP_DIR}/rolling/{config.file_name}/with_debug.bat", "w+"
                ) as target_bat_file:
                    with open(f"rollgui.bat") as origin_bat_file:
                        bat_content = origin_bat_file.read()
                    bat_content = bat_content.replace("__BIN_NAME__", config.bin_name)
                    target_bat_file.write(bat_content)
                if not cross:
                    zip_command = f"cd {TMP_DIR}\\rolling && tar.exe -a -c -f {config.file_name}.zip {config.file_name}"
            print(zip_command)
            subprocess.check_output(
                zip_command,
                shell=True,
            )
            print(f"zip available at {TMP_DIR}/rolling/{config.file_name}.zip")

            file_to_upload_path = f"{TMP_DIR}/rolling/{config.file_name}.zip"
            if "windows" in target and not cross:
                file_to_upload_path = f"{TMP_DIR}\\rolling\\{config.file_name}.zip"

            if upload_tracim:
                print("upload to Tracim ... ")

                # upload
                with open(file_to_upload_path, "rb") as zip_file:
                    response = requests.put(
                        url=TRACIM_API_URL.format(
                            workspace_id=config.tracim_workspace_id,
                            content_id=config.tracim_content_id,
                            filename=f"{config.file_name}.zip",
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
                    f"bux@s2.bux.fr:{config.release_folder}{config.file_name}_{version}.zip",
                    shell=True,
                )
                subprocess.check_output(
                    f'ssh -p 3122 bux@s2.bux.fr \'echo "{version}" '
                    f">> {config.release_folder}index'",
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
