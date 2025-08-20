import { useEffect, useRef, useState } from 'react';
import { useAtom } from 'jotai';
import { containerAtom } from '../state';
import { Terminal as XTerm } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import 'xterm/css/xterm.css';

interface TerminalProps {
    containerName?: string;
}

export default function Terminal({ containerName }: TerminalProps) {
    const ref = useRef<HTMLDivElement>(null);
    const termRef = useRef<XTerm | null>(null);
    const wsRef = useRef<WebSocket | null>(null);
    const [container] = useAtom(containerAtom);
    const [isConnecting, setIsConnecting] = useState(false);
    const activeContainer = containerName || container;

    useEffect(() => {
        if (!activeContainer || !ref.current) return;
        
        setIsConnecting(true);
        
        // Clean up previous terminal if it exists
        if (termRef.current) {
            termRef.current.dispose();
        }
        if (wsRef.current) {
            wsRef.current.close();
        }

        const term = new XTerm({
            cursorBlink: true,
            fontSize: 14,
            fontFamily: 'Menlo, Monaco, "Courier New", monospace',
        });
        
        const fitAddon = new FitAddon();
        term.loadAddon(fitAddon);
        
        termRef.current = term;

        try {
            term.open(ref.current);
            
            // Wait for the terminal to be fully rendered before fitting
            setTimeout(() => {
                try {
                    fitAddon.fit();
                } catch (err) {
                    console.warn('Failed to fit terminal:', err);
                }
            }, 100);

            const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
            const ws = new WebSocket(
                `${protocol}://${window.location.hostname}:6789/terminal/${activeContainer}`
            );
            wsRef.current = ws;

            ws.onopen = () => {
                setIsConnecting(false);
                term.write('Connected to container...\r\n');
            };

            ws.onmessage = (e) => {
                try {
                    term.write(e.data);
                } catch (err) {
                    console.error('Failed to write to terminal:', err);
                }
            };

            ws.onerror = (err) => {
                console.error('WebSocket error:', err);
                setIsConnecting(false);
                term.write('\r\nConnection error. Please try again.\r\n');
            };

            ws.onclose = () => {
                setIsConnecting(false);
                term.write('\r\nConnection closed.\r\n');
            };

            term.onData((data) => {
                if (ws.readyState === WebSocket.OPEN) {
                    ws.send(data);
                }
            });

            // Handle resize
            const handleResize = () => {
                try {
                    fitAddon.fit();
                } catch (err) {
                    console.warn('Failed to fit terminal on resize:', err);
                }
            };

            window.addEventListener('resize', handleResize);

            return () => {
                window.removeEventListener('resize', handleResize);
                if (ws.readyState === WebSocket.OPEN) {
                    ws.close();
                }
                term.dispose();
            };
        } catch (err) {
            console.error('Failed to initialize terminal:', err);
            setIsConnecting(false);
        }
    }, [activeContainer]);

    if (!activeContainer) {
        return (
            <div className="h-full w-full flex items-center justify-center text-gray-500">
                No container specified
            </div>
        );
    }

    return (
        <div className="h-full w-full relative">
            {isConnecting && (
                <div className="absolute inset-0 flex items-center justify-center bg-black bg-opacity-50 text-white z-10">
                    Connecting to terminal...
                </div>
            )}
            <div ref={ref} className="h-full w-full" />
        </div>
    );
}
