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

  return (
    <div className="p-2 space-y-4 text-sm">
      {data?.files.map((f) => (
        <div key={f.path}>
          <h3 className="font-medium">{f.path} ({f.status})</h3>
          <pre className="bg-gray-100 p-2 overflow-auto text-xs">
            {f.diff || 'No diff'}
          </pre>
        </div>
      ))}
    </div>
  );
}
