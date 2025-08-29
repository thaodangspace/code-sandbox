import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { Button } from './ui/button';
import { Card, CardContent, CardHeader } from './ui/card';

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
    <Card className="max-w-xl mx-auto">
      <CardHeader>
        <div className="space-x-2">
          <Button variant="secondary" onClick={goUp}>
            Up
          </Button>
          <Button onClick={startHere}>Start Here</Button>
        </div>
        <span className="text-sm text-gray-600">{path}</span>
      </CardHeader>
      <CardContent>
        <ul className="space-y-1">
          {data?.filter(d => d.is_dir).map(d => (
            <li key={d.path}>
              <Button
                variant="link"
                className="px-0"
                onClick={() => setPath(d.path)}
              >
                {d.name}
              </Button>
            </li>
          ))}
        </ul>
      </CardContent>
    </Card>
  );
}
