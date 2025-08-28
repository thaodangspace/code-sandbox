import { useQuery } from '@tanstack/react-query';
import { useAtom } from 'jotai';
import { containerAtom } from '../state';

interface FileDiff {
  path: string;
  status: string;
  diff?: string;
}

interface ChangeResponse {
  files: FileDiff[];
}

interface DiffViewProps {
  containerName?: string;
}

export default function DiffView({ containerName }: DiffViewProps) {
  const [container] = useAtom(containerAtom);
  const activeContainer = containerName || container;

  const { data, isLoading, error } = useQuery<ChangeResponse>({
    queryKey: ['diff', activeContainer],
    queryFn: async () => {
      const res = await fetch(`/api/changed/${activeContainer}`);
      if (!res.ok) throw new Error('failed');
      return res.json();
    },
    enabled: !!activeContainer,
    refetchInterval: 5000
  });

  if (!activeContainer) return <p className="p-4">No container specified.</p>;
  if (isLoading) return <p className="p-4">Loading...</p>;
  if (error) return <p className="p-4">Error loading diff.</p>;

  function parseDiff(diff: string) {
    const lines = diff.split('\n');
    const result: {
      content: string;
      oldNumber?: number;
      newNumber?: number;
      type: 'add' | 'remove' | 'context' | 'hunk';
    }[] = [];
    let oldLine = 0;
    let newLine = 0;
    const hunkRe = /@@ -(\d+),?\d* \+(\d+),?\d* @@/;
    for (const line of lines) {
      if (line.startsWith('@@')) {
        const m = line.match(hunkRe);
        if (m) {
          oldLine = parseInt(m[1], 10);
          newLine = parseInt(m[2], 10);
        }
        result.push({ content: line, type: 'hunk' });
      } else if (line.startsWith('+') && !line.startsWith('+++')) {
        result.push({
          content: line,
          oldNumber: undefined,
          newNumber: newLine++,
          type: 'add'
        });
      } else if (line.startsWith('-') && !line.startsWith('---')) {
        result.push({
          content: line,
          oldNumber: oldLine++,
          newNumber: undefined,
          type: 'remove'
        });
      } else {
        result.push({
          content: line,
          oldNumber: oldLine++,
          newNumber: newLine++,
          type: 'context'
        });
      }
    }
    return result;
  }

  return (
    <div className="p-2 space-y-4 text-sm">
      {data?.files.map((f) => (
        <div key={f.path}>
          <h3 className="font-medium">{f.path} ({f.status})</h3>
          {f.diff ? (
            <div className="bg-gray-100 p-2 overflow-auto text-xs font-mono">
              {parseDiff(f.diff).map((line, i) => {
                let className = 'flex whitespace-pre';
                if (line.type === 'add') {
                  className += ' text-green-700 bg-green-100';
                } else if (line.type === 'remove') {
                  className += ' text-red-700 bg-red-100';
                } else if (line.type === 'hunk') {
                  className += ' text-purple-700';
                }
                return (
                  <div key={i} className={className}>
                    <span className="w-12 text-right pr-2 text-gray-500 select-none">
                      {line.oldNumber ?? ''}
                    </span>
                    <span className="w-12 text-right pr-2 text-gray-500 select-none">
                      {line.newNumber ?? ''}
                    </span>
                    <span className="flex-1">{line.content}</span>
                  </div>
                );
              })}
            </div>
          ) : (
            <div className="bg-gray-100 p-2 overflow-auto text-xs">No diff</div>
          )}
        </div>
      ))}
    </div>
  );
}
