import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';

interface DirEntry {
  name: string;
  path: string;
  is_dir: boolean;
}

export default function Explorer() {
  const [path, setPath] = useState('/');
  const navigate = useNavigate();

  const { data, isLoading, error } = useQuery<DirEntry[]>({
    queryKey: ['list', path],
    queryFn: async () => {
      const res = await fetch(`/api/list?path=${encodeURIComponent(path)}`);
      if (!res.ok) throw new Error('failed');
      return res.json();
    },
  });

  const goUp = () => {
    if (path === '/') return;
    const parent = path.replace(/\/?[^/]+$/, '') || '/';
    setPath(parent);
  };

  const startHere = async () => {
    const res = await fetch('/api/start', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ path, agent: 'claude' }),
    });
    if (res.ok) {
      const json = await res.json();
      navigate(`/container/${json.container}`);
    }
  };

  if (isLoading) return <p className="p-4">Loading...</p>;
  if (error) return <p className="p-4">Error loading directory.</p>;

  return (
    <div className="p-4 space-y-2">
      <div className="flex space-x-2">
        <button className="px-2 py-1 bg-gray-200" onClick={goUp}>Up</button>
        <button className="px-2 py-1 bg-blue-500 text-white" onClick={startHere}>Start Here</button>
        <span className="ml-4">{path}</span>
      </div>
      <ul className="space-y-1">
        {data?.filter(d => d.is_dir).map(d => (
          <li key={d.path}>
            <button className="text-blue-600 underline" onClick={() => setPath(d.path)}>
              {d.name}
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
