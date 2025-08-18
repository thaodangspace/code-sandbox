import { useEffect, useRef } from 'react';
import { Terminal as XTerm } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import 'xterm/css/xterm.css';

export default function Terminal() {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const term = new XTerm();
    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    if (ref.current) {
      term.open(ref.current);
      fitAddon.fit();
    }
    const ws = new WebSocket(`ws://${location.host}/terminal`);
    ws.onmessage = (e) => term.write(e.data);
    term.onData((d) => ws.send(d));
    return () => {
      ws.close();
      term.dispose();
    };
  }, []);

  return <div ref={ref} className="h-full w-full" />;
}
