import { Tabs, TabsContent, TabsList, TabsTrigger } from './components/ui/tabs';
import Terminal from './components/terminal';
import DiffView from './components/diff';

export default function App() {
  return (
    <Tabs defaultValue="terminal" className="h-full flex flex-col">
      <TabsList className="fixed bottom-0 w-full">
        <TabsTrigger value="terminal">Terminal</TabsTrigger>
        <TabsTrigger value="diff">Diff</TabsTrigger>
      </TabsList>
      <TabsContent value="terminal" className="flex-1 mb-12">
        <Terminal />
      </TabsContent>
      <TabsContent value="diff" className="flex-1 overflow-auto mb-12">
        <DiffView />
      </TabsContent>
    </Tabs>
  );
}
