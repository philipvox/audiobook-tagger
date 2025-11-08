import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { FileAudio, RefreshCw, Wrench, Settings, Upload, UploadCloud, Save, ChevronRight, ChevronDown, Folder, Book, Files, Clock, Zap, FileSearch, AlertCircle } from 'lucide-react';
import { RawTagInspector } from './components/RawTagInspector';

function App() {
  const [activeTab, setActiveTab] = useState('scanner');
  const [groups, setGroups] = useState([]);
  const [scanning, setScanning] = useState(false);
  const [writing, setWriting] = useState(false);
  const [pushing, setPushing] = useState(false);
  const [selectedFiles, setSelectedFiles] = useState(new Set());
  const [expandedGroups, setExpandedGroups] = useState(new Set());
  const [config, setConfig] = useState(null);
  const [showTagInspector, setShowTagInspector] = useState(false);
  const [fileStatuses, setFileStatuses] = useState({});

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    const cfg = await invoke('get_config');
    setConfig(cfg);
  };

  const handleScan = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
    });
    
    if (!selected) return;
    
    setScanning(true);
    setFileStatuses({});
    try {
      const result = await invoke('scan_library', { 
        window: window.__TAURI__?.window.getCurrent(),
        paths: [selected] 
      });
      setGroups(result.groups);
      const allExpanded = new Set(result.groups.map((_, idx) => idx));
      setExpandedGroups(allExpanded);
    } catch (error) {
      console.error('Scan failed:', error);
      alert('Scan failed: ' + error);
    } finally {
      setScanning(false);
    }
  };

  const handleWrite = async () => {
    if (selectedFiles.size === 0) {
      alert('Please select files to write');
      return;
    }

    setWriting(true);
    const fileMap = {};
    
    groups.forEach(group => {
      group.files.forEach(file => {
        if (selectedFiles.has(file.id)) {
          fileMap[file.id] = {
            path: file.path,
            changes: file.changes
          };
        }
      });
    });

    try {
      const result = await invoke('write_tags', {
        request: {
          file_ids: Array.from(selectedFiles),
          files: fileMap,
          backup: true
        }
      });

      const newStatuses = { ...fileStatuses };
      
      Array.from(selectedFiles).forEach(fileId => {
        const hasError = result.errors.some(e => e.file_id === fileId);
        newStatuses[fileId] = hasError ? 'failed' : 'success';
      });
      
      setFileStatuses(newStatuses);

      if (result.failed > 0) {
        const errorMsg = result.errors.map(e => `${e.path}: ${e.error}`).join('\n');
        alert(`Written: ${result.success} files\nFailed: ${result.failed} files\n\nErrors:\n${errorMsg}`);
      } else {
        alert(`Successfully wrote ${result.success} file${result.success !== 1 ? 's' : ''}`);
      }
    } catch (error) {
      console.error('Write failed:', error);
      alert('Write failed: ' + error);
    } finally {
      setWriting(false);
    }
  };

  const handlePush = async () => {
    const successfulFiles = Array.from(selectedFiles).filter(id => fileStatuses[id] === 'success');
    
    if (successfulFiles.length === 0) {
      alert('No successfully written files to push. Please write tags first.');
      return;
    }

    const skippedCount = selectedFiles.size - successfulFiles.length;
    if (skippedCount > 0) {
      if (!confirm(`Push ${successfulFiles.length} successfully written files?\n(Skipping ${skippedCount} failed/unwritten files)`)) {
        return;
      }
    }

    setPushing(true);
    const items = [];
    
    groups.forEach(group => {
      group.files.forEach(file => {
        if (successfulFiles.includes(file.id)) {
          items.push({
            path: file.path,
            metadata: group.metadata
          });
        }
      });
    });

    try {
      const result = await invoke('push_abs_updates', { request: { items } });
      
      let message = `Updated ${result.updated} item${result.updated !== 1 ? 's' : ''} in AudiobookShelf.`;

      if (result.unmatched && result.unmatched.length > 0) {
        message += `\nCould not find matches for: ${result.unmatched.slice(0, 5).join(', ')}${
          result.unmatched.length > 5 ? '…' : ''
        }`;
      }

      if (result.failed && result.failed.length > 0) {
        const failures = result.failed
          .slice(0, 5)
          .map((f) => `${f.path}${f.reason ? ` (${f.reason})` : ''}`)
          .join(', ');
        message += `\nFailed to update: ${failures}${result.failed.length > 5 ? '…' : ''}`;
      }

      alert(message);
    } catch (error) {
      console.error('Push to AudiobookShelf failed:', error);
      alert('Failed to push updates: ' + error);
    } finally {
      setPushing(false);
    }
  };

  const toggleGroup = (groupId) => {
    const newExpanded = new Set(expandedGroups);
    if (newExpanded.has(groupId)) {
      newExpanded.delete(groupId);
    } else {
      newExpanded.add(groupId);
    }
    setExpandedGroups(newExpanded);
  };

  const selectAllInGroup = (group, checked) => {
    const newSelected = new Set(selectedFiles);
    group.files.forEach(file => {
      if (checked) {
        newSelected.add(file.id);
      } else {
        newSelected.delete(file.id);
      }
    });
    setSelectedFiles(newSelected);
  };

  const getGroupIcon = (type) => {
    if (type === 'single') return <Book className="w-4 h-4" />;
    if (type === 'series') return <Folder className="w-4 h-4" />;
    return <Files className="w-4 h-4" />;
  };

  const getFileStatusIcon = (fileId) => {
    const status = fileStatuses[fileId];
    if (status === 'success') return <span className="text-green-600">✓</span>;
    if (status === 'failed') return <span className="text-red-600">✗</span>;
    return null;
  };

  const getSuccessCount = () => {
    return Array.from(selectedFiles).filter(id => fileStatuses[id] === 'success').length;
  };

  const getFailedCount = () => {
    return Array.from(selectedFiles).filter(id => fileStatuses[id] === 'failed').length;
  };

  if (!config) {
    return (
      <div className="h-screen flex items-center justify-center">
        <div className="text-gray-500">Loading...</div>
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      <header className="bg-white border-b border-gray-200 px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <FileAudio className="w-8 h-8 text-red-600" />
            <h1 className="text-2xl font-bold text-gray-900">Audiobook Tagger</h1>
          </div>
          <div className="flex items-center gap-4">
            <button onClick={handleScan} disabled={scanning} className="btn btn-primary flex items-center gap-2">
              <RefreshCw className={`w-4 h-4 ${scanning ? 'animate-spin' : ''}`} />
              {scanning ? 'Scanning...' : 'Scan Library'}
            </button>
            <button 
              onClick={handleWrite} 
              disabled={writing || selectedFiles.size === 0}
              className="btn btn-primary flex items-center gap-2"
            >
              <Save className="w-4 h-4" />
              Write {selectedFiles.size > 0 ? `${selectedFiles.size} Files` : 'Files'}
            </button>
            <button
              onClick={() => setShowTagInspector(true)}
              className="btn btn-secondary flex items-center gap-2"
            >
              <FileSearch className="w-4 h-4" />
              Inspect Tags
            </button>
          </div>
        </div>
      </header>

      <div className="flex-1 flex overflow-hidden">
        <aside className="w-64 bg-white border-r border-gray-200 p-4">
          <nav className="space-y-2">
            <button
              onClick={() => setActiveTab('scanner')}
              className={`nav-item ${activeTab === 'scanner' ? 'active' : ''}`}
            >
              <RefreshCw className="w-4 h-4" />
              Scanner
            </button>
            <button
              onClick={() => setActiveTab('maintenance')}
              className={`nav-item ${activeTab === 'maintenance' ? 'active' : ''}`}
            >
              <Wrench className="w-4 h-4" />
              Maintenance
            </button>
            <button
              onClick={() => setActiveTab('settings')}
              className={`nav-item ${activeTab === 'settings' ? 'active' : ''}`}
            >
              <Settings className="w-4 h-4" />
              Settings
            </button>
          </nav>
        </aside>

        <main className="flex-1 overflow-auto p-6">
          {activeTab === 'scanner' && (
            <div className="space-y-6">
              {groups.length > 0 && (
                <div className="bg-white rounded-lg shadow p-4">
                  <div className="flex items-center justify-between mb-4">
                    <div className="flex items-center gap-4">
                      <span className="text-sm text-gray-600">{groups.length} groups</span>
                      <span className="text-sm text-gray-600">
                        {groups.reduce((sum, g) => sum + g.total_changes, 0)} changes
                      </span>
                      {getSuccessCount() > 0 && (
                        <span className="text-sm text-green-600">
                          ✓ {getSuccessCount()} written
                        </span>
                      )}
                      {getFailedCount() > 0 && (
                        <span className="text-sm text-red-600">
                          ✗ {getFailedCount()} failed
                        </span>
                      )}
                    </div>
                    <div className="flex gap-2">
                      <button onClick={() => {
                        const all = new Set();
                        groups.forEach(g => g.files.forEach(f => all.add(f.id)));
                        setSelectedFiles(all);
                      }} className="text-sm text-blue-600 hover:underline">
                        Select All
                      </button>
                      <button onClick={() => setSelectedFiles(new Set())} className="text-sm text-blue-600 hover:underline">
                        Deselect All
                      </button>
                      <button
                        onClick={handlePush}
                        disabled={pushing || getSuccessCount() === 0}
                        className="btn btn-sm btn-primary flex items-center gap-2"
                      >
                        <UploadCloud className="w-4 h-4" />
                        Push {getSuccessCount() > 0 ? `${getSuccessCount()} to ABS` : 'to ABS'}
                      </button>
                    </div>
                  </div>

                  <div className="space-y-2">
                    {groups.map((group, idx) => (
                      <div key={group.id} className="border rounded-lg">
                        <div className="flex items-center gap-3 p-3 hover:bg-gray-50">
                          <input
                            type="checkbox"
                            checked={group.files.every(f => selectedFiles.has(f.id))}
                            onChange={(e) => selectAllInGroup(group, e.target.checked)}
                            className="checkbox"
                          />
                          <button
                            onClick={() => toggleGroup(idx)}
                            className="flex items-center gap-2 flex-1 text-left"
                          >
                            {expandedGroups.has(idx) ? <ChevronDown className="w-4 h-4" /> : <ChevronRight className="w-4 h-4" />}
                            {getGroupIcon(group.group_type)}
                            <span className="font-medium">{group.group_name}</span>
                            <span className="text-sm text-gray-500">
                              {group.files.length} files · {group.group_type}
                            </span>
                            {group.total_changes > 0 && (
                              <span className="text-sm text-orange-600">{group.total_changes} changes</span>
                            )}
                          </button>
                        </div>

                        {expandedGroups.has(idx) && (
                          <div className="border-t p-3 space-y-2">
                            {group.files.map(file => (
                              <div key={file.id} className="flex items-center gap-3 pl-8">
                                <input
                                  type="checkbox"
                                  checked={selectedFiles.has(file.id)}
                                  onChange={(e) => {
                                    const newSelected = new Set(selectedFiles);
                                    if (e.target.checked) {
                                      newSelected.add(file.id);
                                    } else {
                                      newSelected.delete(file.id);
                                    }
                                    setSelectedFiles(newSelected);
                                  }}
                                  className="checkbox"
                                />
                                {getFileStatusIcon(file.id)}
                                <span className="text-sm flex-1">{file.filename}</span>
                                {Object.keys(file.changes).length > 0 && (
                                  <span className="text-xs text-gray-500">
                                    {Object.keys(file.changes).length} changes
                                  </span>
                                )}
                              </div>
                            ))}
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}
        </main>
      </div>

      {showTagInspector && (
        <RawTagInspector 
          isOpen={showTagInspector} 
          onClose={() => setShowTagInspector(false)} 
        />
      )}
    </div>
  );
}

export default App;
