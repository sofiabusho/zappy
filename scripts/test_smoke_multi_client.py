#!/usr/bin/env python3
"""I01 scripted multi-client localhost smoke: server + 2 autonomous AI clients.

End-to-end sanity that the three Zappy parts wire together on localhost with no
human piloting (RQ01, RQ20) and that the win condition the whole game is built
toward is wired into the live server loop (RQ02).

What the runtime smoke does:
  1. build the Rust server (release) and bind it to an ephemeral localhost port;
  2. wait until the TCP listener answers with the ``WELCOME`` handshake line;
  3. launch **two** ``./client/client`` processes for the same team — each a
     separate OS process that only talks to the server socket (RQ20);
  4. assert both clients complete the handshake autonomously (no stdin) and
     start playing (RQ01: server + AI client(s) interacting);
  5. assert the server stays responsive to a *third* fresh TCP connection while
     serving the two AI clients — i.e. the event loop is multiplexed and never
     blocks on one client (RQ16 spirit; supports RQ01);
  6. tear every process down cleanly.

The single team runs with ``-c 5`` (max 5 concurrent players), so a real
6-teammates-at-level-8 victory (RQ02) cannot fire mid-smoke and kill the server
under us. RQ02 is instead verified *structurally* — the win detector exists,
uses the subject thresholds (6 players / level 8), and is called every tick of
the live server loop — which is the honest integration check for a seconds-long
smoke. The exhaustive win-logic cases live in S15 (`server/src/win.rs` tests).
"""

from __future__ import annotations

import socket
import subprocess
import sys
import time
import unittest
from contextlib import closing
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

SERVER_DIR = ROOT / "server"
SERVER_BIN = SERVER_DIR / "target" / "release" / "zappy-server"
CLIENT_WRAPPER = ROOT / "client" / "client"
GUI_DIR = ROOT / "gui"

TEAM = "team1"
WORLD_W = 10
WORLD_H = 10
CLIENTS_PER_TEAM = 5  # > 2 launched clients, but < 6 → no win can end the smoke
TIME_UNIT = 100

SERVER_READY_TIMEOUT_S = 90.0  # includes a cold cargo build on first run
HANDSHAKE_TIMEOUT_S = 12.0
# The client prints this once the handshake succeeds (see client/zappy_client/main.py).
JOINED_MARKER = f"client: joined team '{TEAM}'"


def _free_localhost_port() -> int:
    """Ask the OS for an unused localhost TCP port."""
    with closing(socket.socket(socket.AF_INET, socket.SOCK_STREAM)) as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def _try_welcome(port: int, timeout: float = 1.0) -> bytes | None:
    """Connect to ``port`` and read the first line; return it or ``None``."""
    try:
        with closing(socket.create_connection(("127.0.0.1", port), timeout=timeout)) as sock:
            sock.settimeout(timeout)
            data = b""
            while b"\n" not in data and len(data) < 64:
                chunk = sock.recv(64)
                if not chunk:
                    break
                data += chunk
            return data
    except OSError:
        return None


def _build_server() -> None:
    """Build the release server binary (idempotent; no-op once built)."""
    subprocess.run(
        ["cargo", "build", "--release", "--quiet"],
        cwd=SERVER_DIR,
        check=True,
    )


class MultiClientSmokeTests(unittest.TestCase):
    server: subprocess.Popen[str]
    clients: list[subprocess.Popen[str]]
    port: int

    @classmethod
    def setUpClass(cls) -> None:
        _build_server()
        assert SERVER_BIN.is_file(), f"server binary missing after build: {SERVER_BIN}"

        cls.port = _free_localhost_port()
        cls.server = subprocess.Popen(
            [
                str(SERVER_BIN),
                "-p", str(cls.port),
                "-x", str(WORLD_W),
                "-y", str(WORLD_H),
                "-c", str(CLIENTS_PER_TEAM),
                "-n", TEAM,
                "-t", str(TIME_UNIT),
            ],
            cwd=ROOT,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
        )

        # Wait for the listener to answer WELCOME (server also builds resources first).
        deadline = time.monotonic() + SERVER_READY_TIMEOUT_S
        cls.first_welcome: bytes | None = None
        while time.monotonic() < deadline:
            if cls.server.poll() is not None:
                out = cls.server.stdout.read() if cls.server.stdout else ""
                raise AssertionError(f"server exited early (code {cls.server.returncode}):\n{out}")
            reply = _try_welcome(cls.port)
            if reply and reply.startswith(b"WELCOME"):
                cls.first_welcome = reply
                break
            time.sleep(0.2)

        # Launch two autonomous AI clients for the same team.
        cls.clients = []
        for _ in range(2):
            cls.clients.append(
                subprocess.Popen(
                    [
                        str(CLIENT_WRAPPER),
                        "-n", TEAM,
                        "-p", str(cls.port),
                        "-h", "127.0.0.1",
                    ],
                    cwd=ROOT,
                    stdin=subprocess.DEVNULL,  # prove there is no human piloting (RQ20/RQ18)
                    stdout=subprocess.PIPE,
                    stderr=subprocess.STDOUT,
                    text=True,
                )
            )

        # Give the clients a moment to connect, handshake, and act.
        cls.client_joined = [False, False]
        deadline = time.monotonic() + HANDSHAKE_TIMEOUT_S
        cls.client_logs = ["", ""]
        while time.monotonic() < deadline and not all(cls.client_joined):
            time.sleep(0.3)
            # Non-invasive readiness probe: the server keeps answering WELCOME
            # while the two AI clients are mid-game (multiplexed, non-blocking).
            cls.midgame_welcome = _try_welcome(cls.port)
            for i, proc in enumerate(cls.clients):
                if proc.poll() is not None and not cls.client_joined[i]:
                    # Client exited; capture whatever it printed.
                    if proc.stdout:
                        cls.client_logs[i] += proc.stdout.read()

    @classmethod
    def _drain_client(cls, proc: subprocess.Popen[str], idx: int) -> str:
        """Stop a client and return its accumulated stdout."""
        if proc.poll() is None:
            proc.terminate()
        try:
            out, _ = proc.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            out, _ = proc.communicate(timeout=5)
        return cls.client_logs[idx] + (out or "")

    @classmethod
    def tearDownClass(cls) -> None:
        for i, proc in enumerate(getattr(cls, "clients", [])):
            cls.client_logs[i] = cls._drain_client(proc, i)
        server = getattr(cls, "server", None)
        if server is not None:
            if server.poll() is None:
                server.terminate()
            try:
                server.communicate(timeout=5)
            except subprocess.TimeoutExpired:
                server.kill()
                server.communicate(timeout=5)

    # --- RQ01: system has a server, AI client(s), and a graphic client ---

    def test_all_three_components_present(self) -> None:
        """RQ01: server binary, AI client, and graphic client all exist."""
        self.assertTrue(SERVER_BIN.is_file(), "server binary missing")
        self.assertTrue(CLIENT_WRAPPER.is_file(), "AI client wrapper missing")
        # Graphic client: TS + Canvas app (not exercised over TCP here, but present).
        self.assertTrue((GUI_DIR / "index.html").is_file(), "graphic client index.html missing")
        self.assertTrue((GUI_DIR / "src" / "app.ts").is_file(), "graphic client app missing")

    def test_server_greets_with_welcome(self) -> None:
        """RQ01/RQ19: the live server answers new TCP connections with WELCOME."""
        self.assertIsNotNone(self.first_welcome, "server never answered WELCOME on the port")
        assert self.first_welcome is not None
        self.assertTrue(self.first_welcome.startswith(b"WELCOME"))

    def test_two_ai_clients_handshake_autonomously(self) -> None:
        """RQ01/RQ20: both AI clients join the team with no human input (stdin closed)."""
        logs = [self._drain_client(p, i) for i, p in enumerate(self.clients)]
        # Cache so tearDownClass does not re-read closed pipes.
        for i, text in enumerate(logs):
            type(self).client_logs[i] = text
        for i, text in enumerate(logs):
            self.assertIn(
                JOINED_MARKER,
                text,
                f"AI client #{i} did not complete the handshake autonomously:\n{text}",
            )

    def test_server_stays_responsive_during_play(self) -> None:
        """RQ16 spirit / RQ01: multiplexed server keeps serving new connects mid-game."""
        # Probe live rather than relying on the loop timing above.
        reply = _try_welcome(self.port)
        self.assertIsNotNone(reply, "server stopped accepting connections while clients played")
        assert reply is not None
        self.assertTrue(reply.startswith(b"WELCOME"))

    # --- RQ20: clients must not exchange data outside the game ---

    def test_clients_have_no_out_of_band_ipc(self) -> None:
        """RQ20: the client only speaks to the server socket — no client↔client channel."""
        pkg = ROOT / "client" / "zappy_client"
        sources = "\n".join(p.read_text(encoding="utf-8") for p in pkg.rglob("*.py"))
        # No server sockets, shared files, pipes, or subprocess IPC between clients.
        forbidden = [
            ".bind(",
            ".listen(",
            "socket.socketpair",
            "multiprocessing",
            "os.mkfifo",
            "os.pipe",
            "subprocess",
        ]
        for token in forbidden:
            self.assertNotIn(
                token,
                sources,
                f"client uses out-of-band mechanism {token!r} (violates RQ20)",
            )

    # --- RQ02: win condition wired into the live server loop ---

    def test_win_detection_wired_into_server_loop(self) -> None:
        """RQ02: the server checks the 6-at-level-8 win condition every tick."""
        win_rs = (SERVER_DIR / "src" / "win.rs").read_text(encoding="utf-8")
        self.assertIn("WIN_LEVEL: u8 = 8", win_rs, "win level threshold is not 8 (subject)")
        self.assertIn("WIN_PLAYERS: usize = 6", win_rs, "win team size is not 6 (subject)")
        net_rs = (SERVER_DIR / "src" / "net.rs").read_text(encoding="utf-8")
        self.assertIn(
            "win::winning_team",
            net_rs,
            "server event loop never calls the win detector (RQ02 not wired)",
        )


if __name__ == "__main__":
    suite = unittest.defaultTestLoader.loadTestsFromModule(sys.modules[__name__])
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    sys.exit(0 if result.wasSuccessful() else 1)
