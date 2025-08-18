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

export default function DiffView() {
  const [container] = useAtom(containerAtom);

  const { data, isLoading, error } = useQuery<ChangeResponse>({
    queryKey: ['diff', container],
    queryFn: async () => {
      const res = await fetch(`/api/changed/${container}`);
      if (!res.ok) throw new Error('failed');
      return res.json();
    },
    enabled: !!container,
    refetchInterval: 5000
  });

  if (!container) return <p className="p-4">No container specified.</p>;
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
