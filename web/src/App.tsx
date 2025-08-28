import { Routes, Route, useParams, useNavigate, useLocation } from 'react-router-dom';
import { useEffect } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './components/ui/tabs';
import Terminal from './components/terminal';
import DiffView from './components/diff';
import Explorer from './components/explorer';

function ContainerView() {
  const { containerName } = useParams<{ containerName: string }>();
  
  return (
    <Tabs defaultValue="terminal" className="h-full flex flex-col">
      <TabsList className="fixed bottom-0 w-full">
        <TabsTrigger value="terminal">Terminal</TabsTrigger>
        <TabsTrigger value="diff">Diff</TabsTrigger>
      </TabsList>
      <TabsContent value="terminal" className="flex-1 mb-12">
        <Terminal containerName={containerName} />
      </TabsContent>
      <TabsContent value="diff" className="flex-1 overflow-auto mb-12">
        <DiffView containerName={containerName} />
      </TabsContent>
    </Tabs>
  );
}

function DefaultView() {
  const navigate = useNavigate();
  const location = useLocation();

  // Back-compat: if someone uses legacy links like
  // /?container=NAME[&token=...&run_b64=...&cwd_b64=...]
  // redirect them to /container/NAME while preserving query params
  useEffect(() => {
    const params = new URLSearchParams(location.search);
    const legacyContainer = params.get('container');
    if (legacyContainer) {
      params.delete('container');
      const qs = params.toString();
      const target = qs
        ? `/container/${legacyContainer}?${qs}`
        : `/container/${legacyContainer}`;
      navigate(target, { replace: true });
    }
  }, [location.search, navigate]);
  
  if (new URLSearchParams(location.search).get('container')) {
    // Avoid flashing Explorer before redirect happens
    return null;
  }

  return <Explorer />;
}

export default function App() {
  return (
    <Routes>
      <Route path="/container/:containerName" element={<ContainerView />} />
      <Route path="/" element={<DefaultView />} />
    </Routes>
  );
}
