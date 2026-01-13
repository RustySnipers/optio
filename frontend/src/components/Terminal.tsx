/**
 * Interactive Remote Terminal Component
 *
 * Uses XTerm.js to provide a real-time terminal interface connected
 * to remote agents via the Optio Hub.
 */

import { useEffect, useRef, useState, useCallback } from "react";
import { Terminal as XTerm } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import {
  startTerminalSession,
  terminalSendInput,
  terminalResize,
  closeTerminalSession,
} from "@/lib/commands";
import { X, Terminal as TerminalIcon, Maximize2, Minimize2 } from "lucide-react";
import "@xterm/xterm/css/xterm.css";

interface TerminalProps {
  /** Agent machine ID to connect to */
  agentId: string;
  /** Agent's gRPC address for direct connection */
  agentAddress: string;
  /** Callback when terminal is closed */
  onClose?: () => void;
  /** Shell type (powershell, bash, cmd, sh) */
  shellType?: string;
  /** Agent hostname for display */
  hostname?: string;
}

export function Terminal({
  agentId,
  agentAddress,
  onClose,
  shellType,
  hostname,
}: TerminalProps) {
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<XTerm | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const unlistenersRef = useRef<UnlistenFn[]>([]);

  // Base64 encoding helper
  const toBase64 = (data: string): string => {
    return btoa(data);
  };

  // Base64 decoding helper
  const fromBase64 = (data: string): string => {
    return atob(data);
  };

  // Initialize the terminal
  useEffect(() => {
    if (!terminalRef.current) return;

    // Create the XTerm instance
    const term = new XTerm({
      cursorBlink: true,
      cursorStyle: "block",
      fontFamily: '"Cascadia Code", "Fira Code", Consolas, monospace',
      fontSize: 14,
      lineHeight: 1.2,
      theme: {
        background: "#0f172a", // slate-900
        foreground: "#e2e8f0", // slate-200
        cursor: "#60a5fa", // blue-400
        cursorAccent: "#0f172a",
        selectionBackground: "#334155", // slate-700
        black: "#0f172a",
        red: "#f87171",
        green: "#4ade80",
        yellow: "#facc15",
        blue: "#60a5fa",
        magenta: "#c084fc",
        cyan: "#22d3ee",
        white: "#e2e8f0",
        brightBlack: "#475569",
        brightRed: "#fca5a5",
        brightGreen: "#86efac",
        brightYellow: "#fde047",
        brightBlue: "#93c5fd",
        brightMagenta: "#d8b4fe",
        brightCyan: "#67e8f9",
        brightWhite: "#f8fafc",
      },
    });

    // Add fit addon for auto-resizing
    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    fitAddonRef.current = fitAddon;

    // Add web links addon for clickable URLs
    const webLinksAddon = new WebLinksAddon();
    term.loadAddon(webLinksAddon);

    // Open terminal in the container
    term.open(terminalRef.current);
    xtermRef.current = term;

    // Initial fit
    setTimeout(() => {
      fitAddon.fit();
    }, 100);

    // Handle window resize
    const handleResize = () => {
      if (fitAddonRef.current) {
        fitAddonRef.current.fit();
      }
    };
    window.addEventListener("resize", handleResize);

    // Cleanup
    return () => {
      window.removeEventListener("resize", handleResize);
      term.dispose();
    };
  }, []);

  // Connect to the agent terminal
  const connect = useCallback(async () => {
    if (!xtermRef.current || !fitAddonRef.current) return;

    setError(null);
    xtermRef.current.writeln("\x1b[1;36mConnecting to agent...\x1b[0m");

    try {
      const dims = fitAddonRef.current.proposeDimensions();
      const response = await startTerminalSession({
        agentId,
        agentAddress,
        shellType,
        cols: dims?.cols || 80,
        rows: dims?.rows || 24,
      });

      if (!response.success) {
        setError(response.error || "Failed to start terminal session");
        xtermRef.current.writeln(
          `\x1b[1;31mConnection failed: ${response.error}\x1b[0m`
        );
        return;
      }

      setSessionId(response.sessionId);
      xtermRef.current.writeln(
        `\x1b[1;32mConnected! Session: ${response.sessionId}\x1b[0m\r\n`
      );
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      setError(errorMsg);
      xtermRef.current.writeln(`\x1b[1;31mError: ${errorMsg}\x1b[0m`);
    }
  }, [agentId, agentAddress, shellType]);

  // Set up event listeners when session is established
  useEffect(() => {
    if (!sessionId || !xtermRef.current) return;

    const setupListeners = async () => {
      // Listen for terminal data
      const dataUnlisten = await listen<{ session_id: string; data: string }>(
        `terminal:data:${sessionId}`,
        (event) => {
          if (xtermRef.current && event.payload.data) {
            const decoded = fromBase64(event.payload.data);
            xtermRef.current.write(decoded);
          }
        }
      );

      // Listen for session started
      const startedUnlisten = await listen<{
        session_id: string;
        shell_type: string;
        pid: number;
      }>(`terminal:started:${sessionId}`, (event) => {
        setConnected(true);
        if (xtermRef.current) {
          xtermRef.current.writeln(
            `\x1b[1;32mShell: ${event.payload.shell_type} (PID: ${event.payload.pid})\x1b[0m\r\n`
          );
        }
      });

      // Listen for session ended
      const endedUnlisten = await listen<{
        session_id: string;
        exit_code: number;
        reason: string;
      }>(`terminal:ended:${sessionId}`, (event) => {
        setConnected(false);
        if (xtermRef.current) {
          xtermRef.current.writeln(
            `\r\n\x1b[1;33mSession ended (exit code: ${event.payload.exit_code})\x1b[0m`
          );
        }
      });

      // Listen for errors
      const errorUnlisten = await listen<{
        session_id: string;
        code: string;
        message: string;
      }>(`terminal:error:${sessionId}`, (event) => {
        setError(event.payload.message);
        if (xtermRef.current) {
          xtermRef.current.writeln(
            `\r\n\x1b[1;31mError: ${event.payload.message}\x1b[0m`
          );
        }
      });

      unlistenersRef.current = [
        dataUnlisten,
        startedUnlisten,
        endedUnlisten,
        errorUnlisten,
      ];
    };

    setupListeners();

    return () => {
      unlistenersRef.current.forEach((unlisten) => unlisten());
      unlistenersRef.current = [];
    };
  }, [sessionId]);

  // Handle terminal input
  useEffect(() => {
    if (!sessionId || !xtermRef.current) return;

    const term = xtermRef.current;

    // Handle user input
    const onData = term.onData(async (data) => {
      try {
        await terminalSendInput({
          sessionId,
          data: toBase64(data),
        });
      } catch (e) {
        console.error("Failed to send input:", e);
      }
    });

    return () => {
      onData.dispose();
    };
  }, [sessionId]);

  // Handle terminal resize
  useEffect(() => {
    if (!sessionId || !fitAddonRef.current) return;

    const fitAddon = fitAddonRef.current;

    const resizeObserver = new ResizeObserver(() => {
      fitAddon.fit();
      const dims = fitAddon.proposeDimensions();
      if (dims) {
        terminalResize({
          sessionId,
          cols: dims.cols,
          rows: dims.rows,
        }).catch(console.error);
      }
    });

    if (terminalRef.current) {
      resizeObserver.observe(terminalRef.current);
    }

    return () => {
      resizeObserver.disconnect();
    };
  }, [sessionId]);

  // Auto-connect on mount
  useEffect(() => {
    connect();
  }, [connect]);

  // Handle close
  const handleClose = async () => {
    if (sessionId) {
      try {
        await closeTerminalSession(sessionId, "User closed");
      } catch (e) {
        console.error("Failed to close session:", e);
      }
    }
    onClose?.();
  };

  // Toggle fullscreen
  const toggleFullscreen = () => {
    setIsFullscreen(!isFullscreen);
    setTimeout(() => {
      fitAddonRef.current?.fit();
    }, 100);
  };

  const containerClass = isFullscreen
    ? "fixed inset-0 z-50 bg-slate-900"
    : "flex flex-col h-full min-h-[400px] bg-slate-900 rounded-lg border border-slate-700";

  return (
    <div className={containerClass}>
      {/* Terminal Header */}
      <div className="flex items-center justify-between px-4 py-2 bg-slate-800 border-b border-slate-700 rounded-t-lg">
        <div className="flex items-center gap-3">
          <TerminalIcon className="w-4 h-4 text-slate-400" />
          <span className="text-sm font-medium text-slate-200">
            Terminal - {hostname || agentId}
          </span>
          {connected && (
            <span className="flex items-center gap-1 text-xs text-green-400">
              <span className="w-2 h-2 bg-green-400 rounded-full animate-pulse" />
              Connected
            </span>
          )}
          {error && !connected && (
            <span className="text-xs text-red-400">Disconnected</span>
          )}
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={toggleFullscreen}
            className="p-1 hover:bg-slate-700 rounded transition-colors"
            title={isFullscreen ? "Exit fullscreen" : "Fullscreen"}
          >
            {isFullscreen ? (
              <Minimize2 className="w-4 h-4 text-slate-400" />
            ) : (
              <Maximize2 className="w-4 h-4 text-slate-400" />
            )}
          </button>
          <button
            onClick={handleClose}
            className="p-1 hover:bg-slate-700 rounded transition-colors"
            title="Close terminal"
          >
            <X className="w-4 h-4 text-slate-400" />
          </button>
        </div>
      </div>

      {/* Terminal Container */}
      <div
        ref={terminalRef}
        className="flex-1 overflow-hidden p-2"
        style={{ minHeight: "300px" }}
      />

      {/* Status Bar */}
      <div className="flex items-center justify-between px-4 py-1 bg-slate-800 border-t border-slate-700 text-xs text-slate-400 rounded-b-lg">
        <span>
          {shellType || "shell"} | {agentAddress}
        </span>
        {sessionId && <span>Session: {sessionId.slice(0, 8)}...</span>}
      </div>
    </div>
  );
}

export default Terminal;
