import argparse
import collections
import http.client
import inspect
import json
import logging
import re
import os
import platform
import shutil
import subprocess

# import tarfile
import urllib.parse
import zipfile


ALL = "all"
TARGET = "target"
RELEASE_DIR_NAME = "release"
CROSS_BUILD_DIR_NAME = "cross_build"

PLATFORM_SYSTEM_WINDOWS = "Windows"

RUST_TARGET_WINDOWS_PATTERN = "-windows-"
RUST_TARGET_WINDOWS_MSVC_PATTERN = "-windows-msvc"


def shell(args: list, silent=False):
    logging.warning(" ".join(args))

    if not silent:
        subprocess.run(args)
    else:
        subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE)


def is_located_china():
    logging.warning(inspect.currentframe().f_code.co_name)

    conn = http.client.HTTPSConnection("ipinfo.io", timeout=1)

    try:
        conn.request("GET", "/json")

        response = conn.getresponse()
        text = response.read().decode()
        data = json.loads(text)

        return data.get("country") == "CN"
    except Exception as _:
        return True
    finally:
        conn.close()


def test_default_proxy(host: str, port: int) -> bool:
    logging.warning(f'{inspect.currentframe().f_code.co_name}("{host}", {port})')

    try:
        conn = http.client.HTTPConnection(host, port, timeout=0.1)
        conn.request("HEAD", "/")
        _ = conn.getresponse()
        return True
    except Exception as _:
        return False
    finally:
        conn.close()


def is_on_windows_subsystem_linux():
    return "WSL_DISTRO_NAME" in os.environ


def get_default_proxy() -> tuple:
    logging.warning(inspect.currentframe().f_code.co_name)

    (schema, ip, port) = ("", "", 0)
    if not is_located_china():
        return (schema, ip, port)

    # windows
    if platform.system() == PLATFORM_SYSTEM_WINDOWS:
        (schema, ip, port) = ("http", "127.0.0.1", 10809)

    # windows subsystem linux
    if is_on_windows_subsystem_linux():
        for line in (
            subprocess.run(
                ["ip", "route"], stdout=subprocess.PIPE, universal_newlines=True
            )
            .stdout.strip()
            .splitlines()
        ):
            if line.startswith("default"):
                host_ip = re.search(r"\s(\d+\.\d+\.\d+\.\d)\s", line).group(1)
                (schema, ip, port) = ("http", host_ip, 10809)
                break

    # try to connect proxy
    if ip and port and not test_default_proxy(host=ip, port=port):
        (schema, ip, port) = ("", "", 0)

    return (schema, ip, port)


def download_file(url: str, path: str):
    logging.warning(f'{inspect.currentframe().f_code.co_name}("{url}", "{path}")')

    parsed_url = urllib.parse.urlparse(url)

    proxy = os.environ.get("HTTPS_PROXY")
    if not proxy:
        conn = http.client.HTTPSConnection(parsed_url.netloc, timeout=15)
    else:
        parsed_proxy = urllib.parse.urlparse(proxy)
        proxy_ip, _, proxy_port = parsed_proxy.netloc.partition(":")
        conn = http.client.HTTPSConnection(proxy_ip, int(proxy_port), timeout=15)
        conn.set_tunnel(parsed_url.netloc)

    try:
        conn.request(
            "GET",
            parsed_url.path + ("?" + parsed_url.query if parsed_url.query else ""),
        )
        response = conn.getresponse()

        check_size = 256 * 1024
        if response.status == 200:
            with open(path, "wb") as file:
                while True:
                    chunk = response.read(check_size)
                    if not chunk:
                        break
                    file.write(chunk)
        elif response.status == 302:
            new_url = response.getheader("Location")
            download_file(url=new_url, path=path)

    except Exception as _:
        pass
    finally:
        conn.close()


def extract_file(path: str, dir: str):
    logging.warning(f'{inspect.currentframe().f_code.co_name}("{path}", "{dir}")')

    # DeprecationWarning: Python 3.14 will, by default, filter extracted tar archives and reject files or modify their metadata. Use the filter argument to control this behavior.
    #   File "D:\Program Files\Python312\Lib\tarfile.py", line 2269, in extractall
    #     self._extract_one(tarinfo, path, set_attrs=not tarinfo.isdir(),
    #   File "D:\Program Files\Python312\Lib\tarfile.py", line 2336, in _extract_one
    #     self._handle_fatal_error(e)
    #   File "D:\Program Files\Python312\Lib\tarfile.py", line 2332, in _extract_one
    #     self._extract_member(tarinfo, os.path.join(path, tarinfo.name),
    #   File "D:\Program Files\Python312\Lib\tarfile.py", line 2415, in _extract_member
    #     self.makefile(tarinfo, targetpath)
    #   File "D:\Program Files\Python312\Lib\tarfile.py", line 2461, in makefile
    #     with bltn_open(targetpath, "wb") as target:
    #          ^^^^^^^^^^^^^^^^^^^^^^^^^^^
    # OSError: [Errno 22] Invalid argument: 'target\\MacOSX11.3.sdk\\usr\\share\\man\\mann\\ttk::scrollbar.ntcl'

    # if path.endswith('.tar.xz'):
    #     with tarfile.open(path, 'r:xz') as tar:
    #         tar.extractall(path=dir)
    # if path.endswith('.zip'):
    #     with zipfile.ZipFile(path, 'r') as zip_ref:
    #         zip_ref.extractall(dir)

    cwd = os.getcwd()
    os.chdir(dir)

    if platform.system() == PLATFORM_SYSTEM_WINDOWS:
        file1, ext1 = os.path.splitext(path)
        file2, ext2 = os.path.splitext(file1)
        if ext1 == ".zip":
            shell(args=["7za", "x", os.path.basename(path)], silent=True)
        else:
            file1 = os.path.basename(path)
            shell(args=["7za", "x", file1], silent=True)
            file2 = os.path.basename(f"{file2}{ext2}")
            shell(args=["7za", "x", file2], silent=True)
            os.remove(file2)
    else:
        if path.endswith(".zip"):
            shell(args=["unzip", os.path.basename(path)])
        if path.endswith(".tar.xz"):
            shell(args=["tar", "-Jxf", os.path.basename(path)])

    os.chdir(cwd)


def is_bin_exists(command: str):
    try:
        subprocess.run([command], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    except FileNotFoundError as _:
        return False
    else:
        return True


def install_7zip(dir: str, name: str):
    logging.warning(f'{inspect.currentframe().f_code.co_name}("{dir}", "{name}")')

    file_name = f"{name}.zip"
    dir_path = os.path.join(dir, name)
    file_path = os.path.join(dir, file_name)
    url = (
        f"https://github.com/ascpkg/asc/releases/download/7z-windows-24.09/{file_name}"
    )

    if platform.system() == PLATFORM_SYSTEM_WINDOWS:
        if os.path.exists(dir_path) and is_bin_exists("7za"):
            return
        if not os.path.exists(file_path):
            download_file(url=url, path=file_path)
        with zipfile.ZipFile(file_path, "r") as zip_ref:
            zip_ref.extractall(dir)


def install_zig(dir: str, name: str):
    logging.warning(f'{inspect.currentframe().f_code.co_name}("{dir}", "{name}")')

    if platform.system() == PLATFORM_SYSTEM_WINDOWS or is_on_windows_subsystem_linux():
        file_name = f'{name}.{"zip" if platform.system() == PLATFORM_SYSTEM_WINDOWS else "tar.xz"}'
        dir_path = os.path.join(dir, name)
        file_path = os.path.join(dir, file_name)
        url = f"https://github.com/ascpkg/asc/releases/download/zig-0.13.0-cf90dfd-20240607/{file_name}"

        if os.path.exists(dir_path) and is_bin_exists("zig"):
            return
        if not os.path.exists(file_path):
            download_file(url=url, path=file_path)
        if os.path.exists(file_path):
            extract_file(path=file_path, dir=dir)


def install_mac_os_sdk(dir: str, name: str):
    logging.warning(f'{inspect.currentframe().f_code.co_name}("{dir}", "{name}")')

    file_name = f"{name}.tar.xz"
    dir_path = os.path.join(dir, name)
    file_path = os.path.join(dir, file_name)
    url = f"https://github.com/ascpkg/asc/releases/download/MacOSX11.3.sdk/{file_name}"

    if os.path.exists(dir_path):
        return
    if not os.path.exists(file_path):
        download_file(url=url, path=file_path)
    if os.path.exists(file_path):
        extract_file(path=file_path, dir=dir)


def set_envs() -> tuple:
    logging.info(f"{inspect.currentframe().f_code.co_name}")

    # build paths
    target = os.path.join(os.getcwd(), TARGET)
    os.makedirs(target, exist_ok=True)
    mac_os_sdk_dir_name = "MacOSX11.3.sdk"
    mac_os_sdk_path = os.path.join(target, mac_os_sdk_dir_name)
    zig_dir_name = f"zig-{platform.system().lower()}-x86_64-0.13.0"
    zig_path = os.path.join(target, zig_dir_name)
    zig_lib_dir = os.path.join(zig_path, "lib")

    # set macOS sdk env
    os.environ["SDKROOT"] = os.environ.get("SDKROOT", mac_os_sdk_path)

    # set zig env
    if platform.system() == PLATFORM_SYSTEM_WINDOWS or is_on_windows_subsystem_linux():
        os.environ["ZIG"] = zig_path
        os.environ["ZIG_LIB_DIR"] = zig_lib_dir

    # set path env
    p7zip_dir_name = "7z-windows-24.09"
    os.environ["PATH"] = os.pathsep.join(
        [
            os.path.join(target, p7zip_dir_name, "x64"),
            os.environ["PATH"],
        ]
    )
    if platform.system() == PLATFORM_SYSTEM_WINDOWS or is_on_windows_subsystem_linux():
        os.environ["PATH"] = os.pathsep.join([os.environ["ZIG"], os.environ["PATH"]])

    # set proxy env
    proxy_schema, proxy_ip, proxy_port = get_default_proxy()
    if proxy_schema and proxy_port and proxy_ip:
        proxy_host_port = f"{proxy_ip}:{proxy_port}"
        proxy_schema_host_port = f"{proxy_schema}://{proxy_host_port}"
        logging.warning(f"set proxy {proxy_schema_host_port}")
        os.environ["HTTP_PROXY"] = os.environ.get("HTTP_PROXY", proxy_schema_host_port)
        os.environ["HTTPS_PROXY"] = os.environ.get(
            "HTTPS_PROXY", proxy_schema_host_port
        )
        os.environ["NO_PROXY"] = os.environ.get(
            "NO_PROXY", f"localhost,127.0.0.1,{proxy_host_port}"
        )

    return (p7zip_dir_name, zig_dir_name, mac_os_sdk_dir_name)


def install_requirements(
    p7zip_dir_name: str, zig_dir_name: str, mac_os_sdk_dir_name: str
):
    logging.info(f"{inspect.currentframe().f_code.co_name}")

    # download if not exists
    install_7zip(dir=TARGET, name=p7zip_dir_name)
    install_zig(dir=TARGET, name=zig_dir_name)
    install_mac_os_sdk(dir=TARGET, name=mac_os_sdk_dir_name)


def install_cargo_zig_build():
    logging.info(f"{inspect.currentframe().f_code.co_name}")

    if platform.system() == PLATFORM_SYSTEM_WINDOWS or is_on_windows_subsystem_linux():
        installed = [
            line.strip()
            for line in subprocess.run(
                ["cargo", "--list"], stdout=subprocess.PIPE, universal_newlines=True
            )
            .stdout.strip()
            .splitlines()
        ]
        if "zigbuild" not in installed:
            shell(args=["cargo", "install", "cargo-zigbuild"])
    else:
        shell(args=["apt", "install", "-y", "python3-pip"])
        shell(args=["python3", "-m", "pip", "install", "cargo-zigbuild"])


def get_rust_targets(glibc_version=""):
    return [
        (
            "x86_64-pc-windows-msvc"
            if platform.system() == PLATFORM_SYSTEM_WINDOWS
            else "x86_64-pc-windows-gnu"
        ),
        (
            "aarch64-pc-windows-msvc"
            if platform.system() == PLATFORM_SYSTEM_WINDOWS
            else "aarch64-pc-windows-gnullvm"
        ),
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        f"x86_64-unknown-linux-gnu{glibc_version}",
        f"aarch64-unknown-linux-gnu{glibc_version}",
    ]


def add_rust_targets(target_pattern: str):
    logging.info(f"{inspect.currentframe().f_code.co_name}")

    installed = (
        subprocess.run(
            ["rustup", TARGET, "list", "--installed"],
            stdout=subprocess.PIPE,
            universal_newlines=True,
        )
        .stdout.strip()
        .splitlines()
    )

    for target in get_rust_targets():
        if not target.startswith(target_pattern):
            continue
        if target not in installed:
            shell(args=["rustup", TARGET, "add", target])


def build_rust_targets(target_pattern: str):
    logging.info(f"{inspect.currentframe().f_code.co_name}")

    for target in get_rust_targets(".2.17"):
        if not target.startswith(target_pattern):
            continue
        shell(
            args=[
                "cargo",
                "build" if RUST_TARGET_WINDOWS_MSVC_PATTERN in target else "zigbuild",
                "--release",
                "--target",
                target,
            ]
        )


def get_package_version():
    logging.warning(inspect.currentframe().f_code.co_name)

    with open("asc_bin/Cargo.toml") as f:
        for line in f:
            if line.startswith("version"):
                return line.partition("=")[-1].strip().strip('"')


def package(target, version):
    # make dirs
    cross_build_dir = os.path.join(TARGET, CROSS_BUILD_DIR_NAME)
    dir_name = f"{target}-{version}"
    dir_path = os.path.join(cross_build_dir, dir_name)
    if os.path.exists(dir_path):
        shutil.rmtree(dir_path, ignore_errors=True)
    os.makedirs(dir_path, exist_ok=True)
    logging.warning(f"makedirs {dir_path}")

    # copy file
    src_file_path = os.path.join(
        TARGET,
        target,
        RELEASE_DIR_NAME,
        f'asc{".exe" if RUST_TARGET_WINDOWS_PATTERN in target else ""}',
    )
    shutil.copy(src_file_path, dir_path)
    logging.warning(f"copy {src_file_path} to {dir_path}")

    # compress
    cwd = os.getcwd()
    os.chdir(cross_build_dir)
    d = os.path.basename(dir_name)
    shutil.make_archive(
        base_name=d,
        base_dir=d,
        format="zip" if RUST_TARGET_WINDOWS_PATTERN in target else "xztar",
    )
    logging.warning(f"compress {dir_name}")
    shutil.rmtree(d)
    os.chdir(cwd)


def package_rust_targets(version: str, target_pattern: str):
    logging.info(f"{inspect.currentframe().f_code.co_name}")

    for target in get_rust_targets():
        if not target.startswith(target_pattern):
            continue
        package(target, version)


def clean_target():
    shutil.rmtree(TARGET, ignore_errors=True)


def check_build_results(version: str, target_pattern: str):
    not_exists = []
    for target in get_rust_targets():
        if not target.startswith(target_pattern):
            continue
        ext = "zip" if RUST_TARGET_WINDOWS_PATTERN in target else "tar.xz"
        path = os.path.join(TARGET, CROSS_BUILD_DIR_NAME, f"{target}-{version}.{ext}")
        if not os.path.exists(path):
            not_exists.append(path)
    if not_exists:
        raise FileNotFoundError(f'not exists: {", ".join(not_exists)}')


# logger ansi color
class Color:
    INFO = "\033[94m"  # Light Blue
    WARNING = "\033[93m"  # Yellow
    ERROR = "\033[91m"  # Red
    RESET = "\033[0m"  # Reset to default


# logger formatter
class ColoredFormatter(logging.Formatter):
    def format(self, record):
        if record.levelno >= logging.ERROR:
            record.msg = f"{Color.ERROR}{record.msg}{Color.RESET}"
        elif record.levelno >= logging.WARNING:
            record.msg = f"{Color.WARNING}{record.msg}{Color.RESET}"
        else:
            record.msg = f"{Color.INFO}{record.msg}{Color.RESET}"

        return super().format(record)


# configure stdout logger
def setup_logger():
    logger = logging.getLogger()
    logger.setLevel(logging.INFO)

    handler = logging.StreamHandler()
    handler.setLevel(logging.INFO)

    formatter = ColoredFormatter(
        "%(asctime)s - %(lineno)d - %(levelname)s - %(message)s"
    )
    handler.setFormatter(formatter)

    logger.addHandler(handler)


# parse command line arguments
command_lines = collections.namedtuple(
    "command_lines",
    (
        ALL,
        TARGET,
        # clean_target.__name__,
        install_requirements.__name__,
        install_cargo_zig_build.__name__,
        add_rust_targets.__name__,
        build_rust_targets.__name__,
        package_rust_targets.__name__,
        check_build_results.__name__,
    ),
)


def parse_command_lines() -> command_lines:
    logging.warning(inspect.currentframe().f_code.co_name)

    arg_parser = argparse.ArgumentParser(description="cross build")

    arg_parser.add_argument(
        f"--{ALL}",
        type=bool,
        default=False,
        help="run all tasks (default False)",
        choices=[True, False],
    )
    arg_parser.add_argument(
        f"--{TARGET}",
        type=str,
        default="",
        help='rust target name (default "")',
        choices=get_rust_targets(),
    )
    # arg_parser.add_argument(
    #     f"--{clean_target.__name__}",
    #     type=bool,
    #     default=False,
    #     help=f'{clean_target.__name__.replace("_", " ")} (default False)',
    #     choices=[True, False],
    # )
    arg_parser.add_argument(
        f"--{install_requirements.__name__}",
        type=bool,
        default=False,
        help=f'{install_requirements.__name__.replace("_", " ")} (default False)',
        choices=[True, False],
    )
    arg_parser.add_argument(
        f"--{install_cargo_zig_build.__name__}",
        type=bool,
        default=False,
        help=f'{install_cargo_zig_build.__name__.replace("_", " ")} (default False)',
        choices=[True, False],
    )
    arg_parser.add_argument(
        f"--{add_rust_targets.__name__}",
        type=bool,
        default=False,
        help=f'{add_rust_targets.__name__.replace("_", " ")} (default False)',
        choices=[True, False],
    )
    arg_parser.add_argument(
        f"--{build_rust_targets.__name__}",
        type=bool,
        default=False,
        help=f'{build_rust_targets.__name__.replace("_", " ")} (default False)',
        choices=[True, False],
    )
    arg_parser.add_argument(
        f"--{package_rust_targets.__name__}",
        type=bool,
        default=False,
        help=f'{package_rust_targets.__name__.replace("_", " ")} (default False)',
        choices=[True, False],
    )
    arg_parser.add_argument(
        f"--{check_build_results.__name__}",
        type=bool,
        default=False,
        help=f'{check_build_results.__name__.replace("_", " ")} (default False)',
        choices=[True, False],
    )

    args = arg_parser.parse_args()

    return command_lines(
        all=args.all,
        target=args.target,
        # clean_target=args.clean_target,
        install_requirements=args.install_requirements,
        install_cargo_zig_build=args.install_cargo_zig_build,
        add_rust_targets=args.add_rust_targets,
        build_rust_targets=args.build_rust_targets,
        package_rust_targets=args.package_rust_targets,
        check_build_results=args.check_build_results,
    )


if __name__ == "__main__":
    setup_logger()

    cli_args = parse_command_lines()

    version = get_package_version()
    (p7zip_dir_name, zig_dir_name, mac_os_sdk_dir_name) = set_envs()

    # if cli_args.clean_target:
    #     clean_target()

    if cli_args.all or cli_args.install_requirements:
        install_requirements(p7zip_dir_name, zig_dir_name, mac_os_sdk_dir_name)

    if cli_args.all or cli_args.install_cargo_zig_build:
        install_cargo_zig_build()

    if cli_args.all or cli_args.add_rust_targets:
        add_rust_targets(cli_args.target)

    if cli_args.all or cli_args.build_rust_targets:
        build_rust_targets(cli_args.target)

    if cli_args.all or cli_args.package_rust_targets:
        package_rust_targets(version, cli_args.target)

    if cli_args.all or cli_args.check_build_results:
        check_build_results(version, cli_args.target)

    # if cli_args.clean_target:
    #     clean_target(cli_args.clean_target)
